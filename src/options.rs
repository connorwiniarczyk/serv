// use std::collections::HashMap;
// use std::process::Command;
// use itertools::Itertools;

// use crate::route_patterns::RequestMatch;
// use crate::route_table::{Route};

// use crate::processors::*;
// use crate::processors;


// #[derive(Clone)]
// pub struct RouteOption {
//     pub args: Vec<Arg>,
//     pub processor: Processor,
//     pub name: String, // the name of the processor
// }

// impl RouteOption {
//     pub fn new(processor: &str, args: Vec<Arg>) -> Self {
//         let name = processor.to_string();
//         let processor = processors::get(processor);
//         Self { processor, args, name }
//     }

//     pub fn apply<'request>(&self, response: ResponseGenerator<'request>) -> ResponseGenerator<'request> {
//         (self.processor)(response, &self.args)
//     }
// }


// pub struct ResponseGenerator<'a> {
//     pub route: &'a Route,
//     pub request_match: &'a RequestMatch<'a>,
//     pub request: &'a crate::Request,
//     pub request_body: &'a Option<String>,

//     pub headers: HashMap<String, String>,
//     pub body: Vec<u8>,
//     pub status: u16,
// }

// impl<'a> ResponseGenerator<'a> {
//     pub fn new(request_match: &'a RequestMatch, route: &'a Route, request: &'a crate::Request, request_body: &'a Option<String>) -> Self  {
//         Self {
//             request_match,
//             route,
//             request,
//             request_body,
//             headers: HashMap::new(),
//             body: vec![],
//             status: 200
//         }
//     }

//     pub fn with_header(mut self, key: &str, value: &str) -> Self {
//         println!("\t\t adding header: {}:{}", key, value);
//         self.headers.insert(key.to_string(), value.to_string()); self
//     }

//     pub fn append_to_body(mut self, mut body: Vec<u8>) -> Self {
//         self.body.append(&mut body); self
//     }

//     /// Sometimes, arguments reference data 
//     pub fn extract_data(&'a self, arg: &Arg) -> Option<String> {
//         let Arg { name, value } = arg;
//         let output = match (name.as_str(), value) {
//             ("query", Some(key)) => {
//                 self.request.url()
//                     .query_pairs()
//                     .find_map(|(k, v)| match &k == key {
//                         true => Some(v),
//                         false => None,
//                     })?
//                     .into_owned()
//             },
//             ("query", None) => self.request.url().query()?.to_string(),
//             ("wild", Some(index)) => self.request_match.wildcards[index.parse::<usize>().ok()?].to_string(),
//             ("wild", None) => self.request_match.wildcards.iter().join(" "),
//             ("text", Some(text)) => text.to_string(),
//             ("body", None) => self.request_body.as_ref().expect("test").as_str().to_string(),
//             (text, None) => text.to_string(),

//             _other => return None,
//         };

//         Some(output)
//     }
// }

// impl Into<tide::Response> for ResponseGenerator<'_> {
//     fn into(self) -> tide::Response {

//         let mut out = tide::Response::builder(self.status);
//         out = self.headers.iter().fold(out, |acc, (key, value)| acc.header(key.as_str(), value.as_str()));
//         out = out.body(self.body);

//         // set the MIME type to text/plain if none was set
//         if self.headers.keys().all(|x| x != "content-type") {
//             out = out.header("content-type", "text/plain");
//         }

//         out.build()
//     }
// }


// #[derive(Debug, Clone)]
// pub struct Arg {
//     pub name: String,
//     pub value: Option<String>,
// }

// impl Arg {
//     pub fn new(name: &str, value: Option<&str>) -> Self {
//         Self {
//             name: name.to_string(),
//             value: value.and_then(|x| Some(x.to_string())),
//         }
//     }
// }

// use std::fmt;

// impl fmt::Display for RouteOption {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         let args_str = &self.args.iter().map(|x| x.to_string()).join(" ");
//         match args_str.len() {
//             0 => write!(f, "{}", self.name),
//             _ => write!(f, "{}({})", self.name, args_str),
//         }
        
//     }
// }

// impl fmt::Display for Arg {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         match &self.value {
//             Some(value) => write!(f, "{}:{}", self.name, value),
//             None => write!(f, "{}", self.name),
//         }
//     }
// }
