use std::fmt;
use std::path::Path;
use prettytable::{ Table, Row, Cell, row, cell };
use itertools::Itertools;
use crate::pattern;
use crate::pattern::{Pattern, Node };
use crate::parser;
use crate::request_state::{RequestState};
use crate::body::Body;
use crate::command::Command;
use hyper::{Request, Response};
use std::collections::HashMap;

use std::default::Default;

#[derive(Clone)]
pub struct Route {
    pub pattern: Pattern,
    pub commands: Vec<Command>,
}

impl Route {
    // pub async fn resolve<'request>(&'request self, request: &'request mut Request<hyper::Body>) -> Result<Response<hyper::Body>, &'request str> {

        // let body_bytes = hyper::body::to_bytes(request.body_mut()).await.unwrap();
        // let body = Body::Raw(body_bytes.to_vec());
        // let mut request_state = RequestState::new(&self, request);

        // match self.request.compare(request) {
        //     Ok(vars) => {
        //         for (key, value) in vars.into_iter(){
        //             request_state.variables.insert(key, value);
        //         }

        //         request_state.body = body;

        //         for command in &self.commands {
        //             command.run(&mut request_state);
        //         }
        //     }

        //     Err(_) => return Err("did not match")
        // }

        // Ok(request_state.into())
    // }
}

pub struct RouteTable {
    pub table: Vec<Route>,
    pub names: HashMap<String, usize>,
    // pub names: HashMap<String, &'route_table Route>
}

impl RouteTable {
    pub fn add(&mut self, route: Route ) {
        self.table.push( route ); 

        // if let Some(name) = route.pattern.name {
        //     self.names.insert(name, table.len() - 1);
        // }
    }

    pub fn new() -> Self {
        Self { table: Vec::new(), names: HashMap::new() }
    }

    pub async fn resolve(&self, mut req: Request<hyper::Body>) -> Result<Response<hyper::Body>, hyper::Error> {
        println!("incoming http request: {}", req.uri());
        for route in self.table.iter() {

            let vars = match route.pattern.compare(&req) {
                Ok(vars) => todo!(),
                Err(_) => continue,
            };

            // if the request matches, resolve it
            let mut state = RequestState::new(&route, &req);
            state.variables = vars;

            for command in route.commands {
                command.run(&mut state);
            }
            // state.body = 

            // if let Ok(result) = route.resolve(&mut req).await {
            //     return Ok(result)
            // }
        }

        return Ok(Response::new(hyper::Body::from("404 errer")))
    }
}

use parser::token::Token;

impl Default for RouteTable {
    fn default() -> Self {
        parser::parse_str("/**path: read $(path)").unwrap()
    }
}

impl fmt::Display for RouteTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>)  -> fmt::Result {

        let mut table = Table::new();
        table.add_row(row![ "ROUTE", "COMMANDS"]);
        for row in self.table.iter() {
            let commands = row.commands.iter().map(|command| command.to_string().replace("\n", "")).join("\n");
            table.add_row(row![row.pattern, commands]);
        }

        write!(f, "{}", table)
    }
}

impl fmt::Debug for Route {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:<8}", self.pattern)
    }
}

impl fmt::Display for Route {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:<8}", self.pattern)
    }
}
