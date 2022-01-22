use std::collections::HashMap;
use std::process::Command;
use itertools::Itertools;

// use crate::route_patterns::RequestMatch;
use crate::route_table::Route;

use crate::Request;


/// A RequestState tracks the state of an incoming HTTP request across its entire lifetime.
pub struct RequestState<'request> {

    pub route: &'request Route,
    
    // I don't think you need this if you use variables
    // pub request_match: &'request  RequestMatch<'request>,

    pub request: &'request Request,
    pub request_body: &'request  Option<String>,

    pub variables: HashMap<String, String>,

    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub status: u16,
}

impl<'request> RequestState<'request> {

    pub fn new(route: &'request Route, request: &'request Request, request_body: &'request Option<String>) -> Self  {
        Self {
            route,
            request,
            request_body,
            variables: HashMap::new(),
            headers: HashMap::new(),
            body: vec![],
            status: 200
        }
    }

    pub fn get_variable(&'request self, name: &str) -> &'request str {
        match self.variables.get(name){
            Some(var) => var.as_str(),
            None => panic!("that variable doesn't exist"), // TODO: this should fail more gracefully
        }
    }

}

impl Into<tide::Response> for RequestState<'_> {
    fn into(self) -> tide::Response {

        let mut out = tide::Response::builder(self.status);
        out = self.headers.iter().fold(out, |acc, (key, value)| acc.header(key.as_str(), value.as_str()));
        out = out.body(self.body);

        // set the MIME type to text/plain if none was set
        if self.headers.keys().all(|x| x != "content-type") {
            out = out.header("content-type", "text/plain");
        }

        out.build()
    }
}
