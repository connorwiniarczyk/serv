/// parses the routes file

use super::*;

use std::fs::File;
use std::io::{BufRead, BufReader};

use regex::Regex;
use lazy_static::lazy_static;

use std::fmt;
use std::path::Path;

use crate::path_expression::PathExpr;
use crate::resolver::Access;
use crate::resolver::Options;


#[derive(Clone)]
pub struct Route {
    pub request: PathExpr,
    pub resource: PathExpr,
    pub options: Options,
}

impl Route {
    pub fn new(request: &str, resource: &str, options: &str) -> Self {
        Self {
            request: PathExpr::new(request),
            resource: PathExpr::new(resource),
            options: Options::from_str(options),
        }
    }

    fn from_line(line: &str) -> Option<Self> {

        lazy_static! {
           static ref COMMENT: Regex = Regex::new(r"#.*").unwrap();
           static ref LINE: Regex = Regex::new(r"([^\s]+)\s+([^\s]+)\s+(.*?)$").unwrap();
        }
         
        // remove comments
        let stripped_line = COMMENT.replace(line, "");

        let captures = LINE.captures(&stripped_line)?;

        println!("{:?}", &captures[3]);
        let out = Self::new(
            &captures[1],
            &captures[2],
            &captures[3],
        );

        Some(out)
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

        let file = File::open(path).expect("There is no `routes` file in this directory");    
        let reader = BufReader::new(file);

        let output = reader.lines()
            .map(|x| x.unwrap_or(String::new()))
            .map(|x| Route::from_line(&x))
            .filter_map(|x| x) // this is a concise way of stripping None values while unwrapping the Some values
            .collect();

        Self { table: output }
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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_line() {
       panic!(); 
    }

}

