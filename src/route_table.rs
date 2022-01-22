/// parses the routes file

use std::fmt;
use std::path::Path;
use tide::Request;
use prettytable::{ Table, Row, Cell, row, cell };
use itertools::Itertools;

use crate::route_patterns::{RequestPattern, Node };
use crate::parser;
use crate::State;

use crate::request_state::RequestState;

use crate::route_patterns;

use crate::command::Command;
use crate::command::command;

#[derive(Clone)]
pub struct Route {
    pub request: RequestPattern,
    pub commands: Vec<Command>,
}

impl Route {
    pub async fn resolve<'request>(&'request self, request: &'request Request<State>, body: &'request Option<String>) -> Result<tide::Response, &'request str> {

        let mut request_state = RequestState::new(&self, &request, body);
        let request_match = self.request.compare(request, &mut request_state);

        if !request_match { return Err("did not match"); }

        // println!("\t found a matching route: {}", &self.request);
        // println!("\t with wildcards: {:?}", &request_match.wildcards);


        for command in &self.commands {
            request_state = command.run(request_state);
        }

        Ok(request_state.into())
    }
}

pub struct RouteTable {
    pub table: Vec<Route>,
}

impl RouteTable {
    pub fn add(&mut self, route: Route ) {
       self.table.push( route ); 
    }

    // pub fn from_file(path: &Path) -> Self {
    //     parser::parse_route_file(path)
    //         .or_else(|e| {println!("failed to parse routes file, using a default instead: {}", e); Err(e)})
    //         .unwrap_or_default()
    // }

    pub fn iter(&self) -> std::slice::Iter<Route> {
        self.table.iter()
    }
}

// use parser::route_parser::route as parse;

use crate::command::Arg;

impl Default for RouteTable {
    fn default() -> Self {
        let mut output = Self { table: vec![] };
        let request = RequestPattern::new(vec![
            Node::val("test"),
            Node::var("abcd"),
        ]);

        let commands = vec![
            command!("set", "var", "hello"),
            command!("echo", "$(path:acbd)", "world"),
            // Command::new("set", vec!["var", "hello"]),
            // Command::new("echo", vec!["$(path:abcd) world"]),
        ];

        output.add(
            Route {
                request,
                commands,
                // request: route_pattern![*test],//RequestPattern{ path: vec![ node!("test") ] },
            });

        return output;
    }
    // fn default() -> Self {
    //     let mut output = Self { table: vec![] };    

    //     // serve index.html as the root
    //     output.add(parse("/ /index.html read filetype(html)").unwrap());

    //     // serve javascript and css files from their own folders, use custom headers to make
    //     // things easier
    //     output.add(parse("/scripts/* scripts/* filetype(js)").unwrap());
    //     output.add(parse("/styles/* styles/*  read filetype(css)").unwrap());

    //     // serve general files, two directories deep
    //     output.add(parse("/* *  read").unwrap());
    //     output.add(parse("/*/* */*  read").unwrap());

    //     return output;
    // }
}

impl fmt::Display for RouteTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>)  -> fmt::Result {

        let mut table = Table::new();
        table.add_row(row![ "REQUEST", "RESOURCE", "OPTIONS" ]);
        for row in self.iter() {
            // let options_str = row.options.iter().map(|x| x.to_string()).join(" ");
            let options_str = "test";
            table.add_row(row![row.request, options_str]);
        }

        write!(f, "{}", table)
    }
}

impl fmt::Debug for Route {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:<8}", self.request)
    }
}

impl fmt::Display for Route {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:<8}", self.request)
    }
}
