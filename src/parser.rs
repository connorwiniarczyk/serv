/// Parses the routes file into a Route Table


use peg;

use crate::route_table::{ Route, RouteTable };

use crate::pattern::{Pattern, Node};
use crate::command::*;

use regex::Regex;
use lazy_static::lazy_static;

use std::path::Path;
use std::io::{BufRead, BufReader};
use std::fs::File;

use std::fs::read_to_string;

// A regular expression used to strip comments from the file before parsing
lazy_static! {
    static ref COMMENT: Regex = Regex::new(r"#.*").unwrap();
}

pub fn parse_route_file(path: &Path) -> Result<RouteTable, ()> {
    let file = read_to_string(path).expect("Cannot find a routes.conf file in this directory");
    let route_table = route_parser::route_table(&file).expect("Failed to parse routes.conf file");
    Ok(route_table)
}

peg::parser! {
    pub grammar route_parser() for str {

        pub rule route_table() -> RouteTable = $(['\n'])* lines:line() ** ['\n'] $(['\n'])* {
            // filter all lines for those that contain valid routes
            let routes = lines.iter()
                .filter_map(|route| route.as_ref())
                .map(|route| route.clone())
                .collect();

            RouteTable { table:routes }   
        }

        rule line() -> Option<Route> = whitespace()? route:route()? $(['#'] [^ '\n']*)? { route }

        rule route_seperator() = quiet!{['\n']+}

        pub rule route() -> Route = request:request() [':'] commands:(commands_multi_line() / commands()) {
            Route { request, commands }
        }

        pub rule request() -> Pattern = "/"? nodes:node() ** "/" {
            if nodes.len() == 0 {
                return Pattern::new(vec![Node::val("")])
            } else {
                return Pattern::new(nodes)
            }
        }

        // pub rule node() -> Node = is_var:$(['*'])? is_rest:$(['*'])? value:$([^ ':' | '/' | '\n' | '\t' | '#' | ' ']+) {
        pub rule node() -> Node = is_var:"*"? is_rest:"*"? value:$([^ ':' | '/' | '\n' | '\t' | '#' | ' ']+) {
            match (is_var, is_rest) {
                (Some(_), Some(_)) => Node::rest(value),
                (Some(_), None) => Node::var(value),
                (None, None) => Node::val(value),
                _  => Node::val(value), 
            }
        }

        pub rule commands() -> Vec<Command> = commands:command() ** ";" ";"? whitespace()? {
            commands
        }
        pub rule commands_multi_line() -> Vec<Command> =
            whitespace()? "{" whitespace_with_line_breaks()?
            commands:command() ** (";" whitespace_with_line_breaks()?)
            ";"? whitespace_with_line_breaks()? "}" {
            commands
        }

        pub rule command() -> Command = whitespace()? name:word() args:args()? whitespace()? {
            match args {
                Some(args) => Command::new(name, args),
                None => Command::new(name, vec![]),
            }
        }

        pub rule args() -> Vec<Arg> = whitespace()? args:arg() ** whitespace() { args }

        pub rule arg() -> Arg = word:word() { Arg::new(None, word) }

        rule word() -> &'input str = word:$([^ ' ' | '\t' | '\n' | ';' | '#' | '}']+) { word }
        rule whitespace() = quiet!{[' ' | '\t']+}
        rule whitespace_with_line_breaks() = quiet!{[' ' | '\t' | '\n' | '\r']+}
        // rule whitespace() = quiet!{[' ' | '\t' | '\n']+}
    }
}


#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test() {
        let res = route_parser::route_table("/test : echo abcd;\n\n\n/hello: echo test; \n\n\n\n\n\n\n # comment \n").unwrap();
    }
}
