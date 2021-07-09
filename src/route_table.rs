/// parses the routes file

use super::*;

use regex::Regex;
use lazy_static::lazy_static;

use std::fmt;
use std::path::Path;

use crate::path_expression::{ RequestPattern, ResourcePattern };
use crate::options::RouteOption;

use crate::parser;

lazy_static! {
    static ref COMMENT: Regex = Regex::new(r"#.*").unwrap();
}

#[derive(Clone)]
pub struct Route {
    pub request: RequestPattern,
    pub resource: ResourcePattern,
    pub options: Vec<RouteOption>,
}

use crate::State;
use tide::Request;

use crate::options::ResponseGenerator;

impl Route {
    pub fn resolve<'request>(&'request self, request: &'request Request<State>) -> Result<tide::Response, &'request str> {
        let request_match = self.request.compare(request)?;
        println!("\t found a matching route: {} --> {}", &self.request, &self.resource);
        println!("\t with wildcards: {:?}", &request_match.wildcards);

        let mut response = ResponseGenerator::new(&request_match, &self, &request);
        for option in &self.options {
            response = option.apply(response);
        }
        

        Ok(response.into())
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
