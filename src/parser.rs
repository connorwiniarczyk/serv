/// Parses the routes file into a Route Table


use peg;
use crate::error;

use crate::route_table;
use crate::options;

use crate::options::{ Arg, RouteOption };

use crate::route_table::{ Route };


peg::parser! {
    grammar path_parser() for str {

        pub rule route() -> Route = request:path() " "+ resource:path() " "+ options:options() { todo!() }

        pub rule path() -> Vec<String> = root:"/"? node:path_node() ** "/"  { node }
        rule path_node() -> String = n:$(['a'..='z' | 'A'..='Z' | '0'..='9']+) { n.to_string() }


        // rules for parsing options. eg: exec(), header(), etc.
        pub rule options() -> Vec<RouteOption> = options:option() ** " " { options }

        // The class of character that can be included in an identifier.
        // an identifier is any option name, argument, or value
        rule ident() -> &'input str = n:$([^ '(' | ')' | ' ' | ':' | ',']+) { n }

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
        path_parser::options("exec(query:first wild:0) header(content-type:text/html)").unwrap();
        // let args = path_parser::arguments("(test:abcd test)").unwrap();
        // path_parser::arguments("test:123 abcd one two:three 10:10 10:one").unwrap();

    }
}
