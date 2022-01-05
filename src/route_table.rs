/// parses the routes file

use std::fmt;
use std::path::Path;
use tide::Request;

use crate::route_patterns::{ RequestPattern, ResourcePattern };
use crate::options::RouteOption;
use crate::parser;
use crate::State;
use crate::options::ResponseGenerator;


#[derive(Clone)]
pub struct Route {
    pub request: RequestPattern,
    pub resource: ResourcePattern,
    pub options: Vec<RouteOption>,
}


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

    pub fn sanitize(mut self) -> Self {
        let first_option = self.options.iter().next();

        let needs_default = match first_option {
            Some( RouteOption { func, args, func_name } ) => {
                !(func_name == "exec" || func_name == "read")
            }
            None => true,
        };

        self.options.insert(0, RouteOption::new("read", vec![]));
        return self
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
        parser::parse_route_file(path)
            .or_else(|e| {println!("failed to parse routes file, using a default instead: {}", e); Err(e)})
            .unwrap_or_default()
    }

    pub fn iter(&self) -> std::slice::Iter<Route> {
        self.table.iter()
    }
}

use parser::route_parser::route as parse;

impl Default for RouteTable {
    fn default() -> Self {
        let mut output = Self { table: vec![] };    

        // serve index.html as the root
        output.add(parse("/ /index.html read header(content-type:text/html)").unwrap());

        // serve javascript and css files from their own folders, use custom headers to make
        // things easier
        output.add(parse("/scripts/* scripts/* read header(content-type:text/javascript)").unwrap());
        output.add(parse("/styles/* /styles/*  read header(content-type:text/css)").unwrap());

        // serve general files, two directories deep
        output.add(parse("/* /*  read").unwrap());
        output.add(parse("/*/* /*/*  read").unwrap());

        return output;
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
