/// parses the routes file

use super::*;

use std::fs::File;
use std::io::{BufRead, BufReader};

use regex::Regex;
use lazy_static::lazy_static;

use std::fmt;
use std::path::Path;

use crate::path_expression::PathExpr;
use crate::options::RouteOption;

use crate::error;

use crate::parser;

lazy_static! {
    static ref COMMENT: Regex = Regex::new(r"#.*").unwrap();
}

#[derive(Clone)]
pub struct Route {
    pub request: PathExpr,
    pub resource: PathExpr,
    pub options: Vec<RouteOption>,
}

use crate::State;
use tide::Request;

use crate::options::ResponseGenerator;

impl Route {

    pub fn new(request: &str, resource: &str, options: &str) -> Self {
        todo!()
    }

    pub fn resolve<'request>(&self, request: &'request Request<State>) -> Option<tide::Response> {

        let temp = self.request.match_request(request.param("route").unwrap_or(""));

        println!("");
        println!("{:?}", temp);
        println!("");

        let path_match = self.request.match_request(request.param("route").unwrap_or(""))?;

        // let response = self.options.iter().fold(ResponseGenerator::new(&path_match, self), |acc, x| x.apply(acc)); 
        let mut response = ResponseGenerator::new(&path_match, &self, &request);

        for option in &self.options {
            response = option.apply(response);
        }

        Some(response.into())
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
        parser::parse_route_file(path).unwrap()
    }

    pub fn iter(&self) -> std::slice::Iter<Route> {
        self.table.iter()
    }
}

impl fmt::Debug for Route {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:<8} {:^5} {}", self.request, "-->", self.resource)
    }
}

impl fmt::Display for Route {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:<8} {:^5} {}", self.request, "-->", self.resource)
    }
}
