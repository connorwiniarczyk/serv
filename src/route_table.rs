/// parses the routes file

use super::*;

use std::fs::File;
use std::io::{BufRead, BufReader};

use regex::Regex;
use lazy_static::lazy_static;

use std::fmt;
use std::path::Path;

#[derive(Clone)]
pub struct Route{
    pub path: String,
    pub resolver: Resolver,
}

impl Route {
    pub fn new(path: &str, resolver: Resolver) -> Self {
        Self { path: path.to_string(), resolver }
    }
}

pub struct RouteTable {
    // pub root: PathBuf, // the root path
    pub table: Vec<Route>,
}

impl RouteTable {
    pub fn add(&mut self, route: Route ) {
       self.table.push( route ); 
    }

    fn parse_line(line: &str, index: usize) -> Option<(String, String, String)> {

        lazy_static! {
           static ref COMMENT: Regex = Regex::new(r"#.*").unwrap();
           static ref LINE: Regex = Regex::new(r"([^\s]+)\s+([^\s]+)\s+([x,f]?)").unwrap();
        }
         
        // remove comments
        let stripped_line = COMMENT.replace(line, "");

        if let Some(captures) = LINE.captures(&stripped_line) {
            return Some((captures[1].to_string(), captures[2].to_string(), captures[3].to_string())); 
        } else {
            return None
        }
    }

    pub fn from_file(path: &Path) -> Self {

        let file = File::open(path).unwrap();    
        let reader = BufReader::new(file);

        let mut output = Self { table: vec![] };

        for(index, line) in reader.lines().enumerate() {

            let line = line.unwrap_or(String::new());

            // TODO: make this match expression prettier, preferably a one liner
            let ( route, handler, flags ) = match Self::parse_line(&line, index){
                Some(value) => value,
                None => continue,
            };

            let resolver = match flags.as_str() {
                "f" => Resolver::file(&handler),
                "x" => Resolver::exec(&handler),
                 _  => Resolver::file(&handler),
            };

            let new_route = Route { path: route, resolver };
            output.add(new_route);
        }

        return output
    }
}


impl fmt::Debug for Route {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let resolver_desc = match &self.resolver {
            Resolver::File{ path } => format!("file: {}", path),
            Resolver::Exec{ path } => format!("exec: *{}", path),
            _ => "other".to_string(),
        };

        write!(f, "{:<8} {:^5} {}", self.path, "-->", resolver_desc)
    }
}

impl fmt::Display for Route {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let resolver_desc = match &self.resolver {
            Resolver::File{ path } => format!("file: {}", path),
            Resolver::Exec{ path } => format!("exec: *{}", path),
            _ => "other".to_string(),
        };

        write!(f, "{:<8} {:^5} {}", self.path, "-->", resolver_desc)
    }
}

