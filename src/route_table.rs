/// parses the routes file

use super::*;

use std::fs::File;
use std::io::{BufRead, BufReader};

use regex::Regex;
use lazy_static::lazy_static;

use std::fmt;
use std::path::Path;

use crate::path_expression::PathExpr;
use crate::options::Access;
use crate::options::Options;

lazy_static! {
    static ref COMMENT: Regex = Regex::new(r"#.*").unwrap();
    static ref LINE: Regex = Regex::new(r"(?P<request>[^\s]+)\s+(?P<resource>[^\s]+)\s+(?P<options>.*?)$").unwrap();
}

#[derive(Clone)]
pub struct Route {
    pub request: PathExpr,
    pub resource: PathExpr,
    pub options: Options,
}

use crate::State;
use tide::Request;

use crate::options::Response;

impl Route {
    pub fn new(request: &str, resource: &str, options: &str) -> Self {
        Self {
            request: PathExpr::new(request),
            resource: PathExpr::new(resource),
            options: Options::from_str(options),
        }
    }

    fn from_line(line: String) -> Option<Self> {
        let stripped_line = COMMENT.replace(&line, "");  // remove comments
        let captures = LINE.captures(&stripped_line)?;
        Some(Self::new(
            captures.name("request")?.as_str(),
            captures.name("resource")?.as_str(),
            captures.name("options")?.as_str(),
        ))
    }

    pub async fn resolve(&self, request: &Request<State>) -> Option<Response> {

        let path_match = self.request.match_request(request.param("route").unwrap_or(""))?;
        let mut rendered_path = path_match.to_path(&self.resource);

        // prepend working directory if path is local
        if rendered_path.is_relative() {
            rendered_path = request.state().config.root.join(&rendered_path); 
        }

        // debug information
        println!("serving resource at path: {:?}", rendered_path);

        let mut output = match &self.options.access_type {
            Access::Read => std::fs::read_to_string(rendered_path).ok()?,
            Access::Exec(args) => {
                let query = HttpQuery::from_url(request.url());
                let rendered_args: Vec<&str> = args.iter().map(| Arg{ name, value } | match (name.as_str(), value){
                    ("query", Some(param)) => query.get(param).unwrap_or(""),
                    ("wild", Some(index)) => &path_match.wildcards[index.parse::<usize>().unwrap()],
                    (_, _) => "",
                }).collect();

                // debug information
                println!("executing with args: {:?}", rendered_args);

                let output_raw = Command::new(&rendered_path).args(rendered_args).output().await.ok()?;
                let output = std::str::from_utf8(&output_raw.stdout).ok()?;
                output.to_string()
            },
        };

        let response = self.options.post_processors.iter().fold(Response::new(&output), |acc, x| x.apply(acc));

        println!();

        Some(response)
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
        let table = reader.lines()
            .filter_map(|x| x.ok()) // remove Err values and unwrap Ok() values
            .filter_map(Route::from_line) // turn each line into a Route, and remove failures
            .collect();
        Self { table }
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


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_line() {
        ()
       // panic!(); 
    }
}

