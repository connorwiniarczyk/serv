/// parses the routes file

use std::fmt;
use std::path::Path;
use prettytable::{ Table, Row, Cell, row, cell };
use itertools::Itertools;

use crate::pattern;
use crate::pattern::{Pattern, Node };

use crate::parser;
// use crate::State;

use crate::request_state::RequestState;


use crate::command::Command;

use hyper::{Request, Body, Response};

#[derive(Clone)]
pub struct Route {
    pub request: Pattern,
    pub commands: Vec<Command>,
}

impl Route {
    pub async fn resolve<'request>(&'request self, request: &'request mut Request<Body>) -> Result<Response<Body>, &'request str> {

        let body = hyper::body::to_bytes(request.body_mut()).await.unwrap().to_vec();
        let mut request_state = RequestState::new(&self, request);

        match self.request.compare(request) {
            Ok(vars) => {
                for (key, value) in vars.into_iter(){
                    request_state.variables.insert(key, value);
                }

                request_state.body = body;

                for command in &self.commands {
                    command.run(&mut request_state);
                }

            }

            Err(_) => return Err("did not match")
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

    pub fn from_file(path: &Path) -> Self {
        parser::parse_route_file(path).unwrap_or_default()
    }

    pub fn iter(&self) -> std::slice::Iter<Route> {
        self.table.iter()
    }

    pub async fn resolve(&self, mut req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
        println!("incoming http request: {}", req.uri());
        for route in self.iter() {
            if let Ok(result) = route.resolve(&mut req).await {
                return Ok(result)
            }
        }

        return Ok(Response::new(Body::from("404 errer")))
    }
}

use parser::token::Token;

impl Default for RouteTable {
    fn default() -> Self {
        let mut output = Self { table: vec![] };
        let request = Pattern::new(vec![
            Node::rest("path"),
        ]);

        let command = Command::new("read", Some("$(path)"));

        output.add(Route { request, commands: vec![command] });

        output
    }
}

impl fmt::Display for RouteTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>)  -> fmt::Result {

        let mut table = Table::new();
        table.add_row(row![ "ROUTE", "COMMANDS"]);
        for row in self.iter() {
            let commands = row.commands.iter().map(|command| command.to_string().replace("\n", "")).join("\n");
            table.add_row(row![row.request, commands]);
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
