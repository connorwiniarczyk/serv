/// Parses the routes file into a Route Table


use peg;

use crate::options::{ Arg, RouteOption };
use crate::route_table::{ Route, RouteTable };

use regex::Regex;
use lazy_static::lazy_static;

use std::path::Path;
use std::io::{BufRead, BufReader};
use std::fs::File;

use crate::path_expression::{ Node, RequestPattern, ResourcePattern };

lazy_static! {
    // A regular expression used to strip comments from the file before parsing
    static ref COMMENT: Regex = Regex::new(r"#.*").unwrap();
}

pub fn parse_route_file(path: &Path) -> Result<RouteTable, String>{
    let file = File::open(path).expect("There is no `routes` file in this directory");    
    let reader = BufReader::new(file);
    let table = reader.lines()
        .filter_map(|x| x.ok()) // remove Err values and unwrap Ok() values
        .map(|x| COMMENT.replace(&x, "").to_string()) // remove Comments
        .filter_map(|x| route_parser::route(&x).ok()) // turn each line into a Route, and remove failures
        .collect();

    Ok(RouteTable { table })
}

peg::parser! {
    pub grammar route_parser() for str {

        pub rule route() -> Route = request:request() [' ' | '\t']+ resource:resource() [' ' | '\t']+ options:options() {
            Route { request, resource, options }    
        }

        rule path_node() -> Node = node:$([^ ' ' | '\t' | ':' | '/' | '\\']*) { Node::from_str(node) }

        pub rule request() -> RequestPattern = root:"/"? nodes:path_node() ** "/" {
            RequestPattern{ path:nodes }
        }

        pub rule resource() -> ResourcePattern = root:"/"? nodes:path_node() ** "/" {
            ResourcePattern{ is_global: root.is_some(), path:nodes }
        }

        // rules for parsing options. eg: exec(), header(), etc.
        pub rule options() -> Vec<RouteOption> = options:option() ** " " { options }

        // The class of character that can be included in an identifier.
        // an identifier is any option name, argument, or value
        rule ident() -> &'input str = n:$([^ '(' | ')' | '\t' | ' ' | ':' | ',']+) { n }

        pub rule option() -> RouteOption = name:ident() args:arguments()? { RouteOption::new( name, args.unwrap_or(vec![]) ) }

        pub rule arguments() -> Vec<Arg> = "(" args:argument() ** ([ ',' | ' ' ]+) ")"   { args }
        pub rule argument() -> Arg = arg:ident() val:arg_value()? { Arg::new(arg, val) }
        rule arg_value() -> &'input str = ":" val:ident() { val }
    }
}




#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test() {
        let route = route_parser::route("/styles/*   css/*          	header(content-type:text/css)").unwrap();
    }

    #[test]
    fn file() {
        parse_route_file(Path::new("/home/connor/projects/serv/examples/cms/routes")).unwrap();
    }

    #[test]
    fn test2() {
        let path = "/home/connor/projects/serv/examples/cms/routes";
        let file = File::open(path).expect("There is no `routes` file in this directory");    
        let reader = BufReader::new(file);
    }
}

