use std::collections::HashMap;
use std::process::Command;
use itertools::Itertools;

// use crate::route_patterns::RequestMatch;
use crate::route_table::Route;

use crate::Request;


/// A RequestState tracks the state of an incoming HTTP request across its entire lifetime.
pub struct RequestState<'request> {

    pub route: &'request Route,
    pub request: &'request Request,
    pub request_body: Option<&'request str>,
    // pub request_body: Option<String>,

    pub variables: HashMap<String, String>,

    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub status: u16,
}

impl<'request> RequestState<'request> {

    pub fn new(route: &'request Route, request: &'request Request, request_body: Option<&'request str>) -> Self  {

        // populate variables with key value pairs in the query string
        let mut variables = HashMap::new();
        for (key, value) in request.url().query_pairs() {
            variables.insert(format!("query:{}", key), value.to_string());
        }

        Self {
            route,
            request,
            request_body,
            variables,
            headers: HashMap::new(),
            body: vec![],
            status: 200
        }
    }

    pub fn set_variable(&'request mut self, key: &str, value: &str) {
        self.variables.insert(key.to_string(), value.to_string());
    }

    pub fn get_variable(&'request self, name: &str) -> &'request str {
        if name == "body" {
            return &self.request_body.unwrap_or("");
        }

        self.variables.get(name).and_then(|val| Some(val.as_str())).unwrap_or("")
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

use std::fmt::Debug;
use std::fmt;
impl<'request> Debug for RequestState<'request> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("State")
            .field("status", &self.status)
            .field("body", &std::str::from_utf8(&self.body).unwrap_or("<bin>"))
            .field("headers", &self.headers)
            .field("vars", &self.variables)
            .finish()
        
    }
}
