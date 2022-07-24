use std::collections::HashMap;
use std::process::Command;
use itertools::Itertools;
use url::Url;

use hyper::{Request, Response, Body};

// use crate::route_patterns::RequestMatch;
use crate::route_table::Route;

use lazy_static::lazy_static;

// use crate::Request;

/// A RequestState tracks the state of an incoming HTTP request across its entire lifetime.
pub struct RequestState<'request> {

    pub route: &'request Route,
    pub request: &'request Request<Body>,

    pub variables: HashMap<String, String>,
    pub headers: HashMap<String, String>,

    pub body: Vec<u8>, 
    pub mime: Option<String>,

    pub status: u16,
}

impl<'request> RequestState<'request> {

    pub fn new(route: &'request Route, request: &'request Request<Body>) -> Self  {

        // populate variables with key value pairs in the query string
        let mut variables = HashMap::new();
        println!("{:?}", request);


        let mut url_string = "http://0.0.0.0".to_string();
        url_string.push_str(&request.uri().to_string());

        let url = Url::parse(&url_string).expect("Could not parse url");


        // let url = Url::parse(&request.uri().to_string()).unwrap();
        for (key, value) in url.query_pairs() {
            variables.insert(format!("query:{}", key), value.to_string());
        }

        Self {
            route,
            request,
            variables,
            headers: HashMap::new(),
            body: vec![],
            mime: None,
            status: 200
        }
    }

    pub fn set_variable(&'request mut self, key: &str, value: &str) {
        self.variables.insert(key.to_string(), value.to_string());
    }

    pub fn get_variable(&'request self, name: &str) -> &'request str {

        // TODO: add back a way to access the body as a variable.
        // Because the body is a stream now this gets more complicated since you need to await the
        // end of the stream in order to get all its data
        //
        // if name == "body" {
        //     return match &self.request.body() {
        //         Some(bytes) => std::str::from_utf8(bytes).unwrap_or(""),
        //         None => "",

        //     };
        //     // return std::str::from_utf8(&self.request_body.unwrap_or(Vec::new())).unwrap_or("");
        // }

        self.variables.get(name).and_then(|val| Some(val.as_str())).unwrap_or("")
    }

    // Automatically detect the mime type of the response
    pub fn set_mime_type(&mut self) {
        match &self.mime {
            Some(mime_type) => self.headers.insert("content-type".to_string(), mime_type.to_string()),
            None => self.headers.insert("content-type".to_string(), tree_magic::from_u8(&self.body)),
        };
    }

}

impl Into<Response<Body>> for RequestState<'_> {
    fn into(mut self) -> Response<Body> {

        self.set_mime_type();

        let mut out = hyper::Response::builder().status(self.status);

        // for (key, value) in self.headers.iter() {
        //     out.headers_mut().insert(hyper::header::HeaderName.from_lowercase())
        //     out.header(key.as_str(), value.as_str());
        // }


        out.body(self.body.into()).unwrap()
    }
}

use std::fmt::Debug;
use std::fmt;
impl<'request> Debug for RequestState<'request> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("State")
            .field("status", &self.status)
            .field("request_body", &self.request.body())
            .field("body", &std::str::from_utf8(&self.body).unwrap_or("<bin>"))
            .field("headers", &self.headers)
            .field("vars", &self.variables)
            .finish()
        
    }
}
