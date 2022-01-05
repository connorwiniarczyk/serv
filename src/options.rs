
use std::collections::HashMap;
use crate::route_table::*;
use tide::http::Url;
use std::process::Command;
use itertools::Itertools;

type OptionFunc = for<'a> fn(ResponseGenerator<'a>, &Vec<Arg>) -> ResponseGenerator<'a>;

#[derive(Clone)]
pub struct RouteOption {
    pub args: Vec<Arg>,
    pub func: OptionFunc,

    pub func_name: String,
}

impl RouteOption {
    pub fn new(func: &str, args: Vec<Arg>) -> Self {
        let func_name = func.to_string();
        let func = access_types::get_func(func);
        Self { func, args, func_name }
    }

    pub fn apply<'request>(&self, response: ResponseGenerator<'request>) -> ResponseGenerator<'request> {
        (self.func)(response, &self.args)
    }
}

/// Declare an option using a javascript style arrow function syntax.
/// Generates a function pointer that can be used in the `func` field of
/// RouteOptions in the RouteTable. This is mainly useful as a way of
/// abbreviating the very verbose function signature that `RouteOption` 
/// requires.
/// 
/// Takes as arguments the name of the function, followed by an arrow function
/// that performs some operation on a `ResponseGenerator`.
///
/// ### examples
/// ```
/// // Adds CORS headers to the response
/// option!(cors (input) => {
///     input.with_header("Access-Control-Allow-Origin", "*")
/// });
/// ```
macro_rules! option {
    ($name:ident, ($input:ident, $args:ident) => $func:block) => {
        pub fn $name<'a>(mut $input: ResponseGenerator<'a>, $args: &Vec<Arg>) -> ResponseGenerator<'a> {
            $func
        }
    };

    ($name:ident, ($input:ident) => $func:block) => {
        pub fn $name<'a>(mut $input: ResponseGenerator<'a>, _args: &Vec<Arg>) -> ResponseGenerator<'a> {
            $func
        }
    };
}


pub mod access_types { 
    use super::*;
    use std::fs;


    option!(exec, (response, args) => {
        let path = response.route.resource.get_path(response.request_match);
        let rendered_args: Vec<String> = args.iter()
            .filter_map(|x| response.extract_data(x))
            .collect();

        let result = Command::new(&path).args(rendered_args).output().unwrap().stdout;
        response.with_body(result)
    });

    option!(read, (input) => {
        let path = input.route.resource.get_path(input.request_match);
        input.with_body(fs::read(&path).unwrap_or_default()) 
    });

    option!(header, (input, args) => {
        args.into_iter().fold(input, |response, arg| match arg {
            Arg { name, value: Some(value) } => response.with_header(name, value),
            Arg { name, value: None } => response,
        })     
    });

    option!(cors, (input) => {
        input.with_header("Access-Control-Allow-Origin", "*")
    });


    pub fn get_func(input: &str) -> OptionFunc {
        match input {
            "exec" => exec,
            "read" => read,
            "header" => header,
            "cors" => cors,
            _ => panic!(),
        }
    }
}


use crate::path_expression::RequestMatch;

pub struct ResponseGenerator<'a> {
    pub route: &'a Route,
    pub request_match: &'a RequestMatch<'a>,
    pub request: &'a crate::Request,

    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub status: u16,
}

impl<'a> ResponseGenerator<'a> {
    pub fn new(request_match: &'a RequestMatch, route: &'a Route, request: &'a crate::Request) -> Self  {
        Self {
            request_match,
            route,
            request,
            headers: HashMap::new(),
            body: vec![],
            status: 200
        }
    }

    pub fn with_header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string()); self
    }

    pub fn with_body(mut self, body: Vec<u8>) -> Self {
        self.body = body; self
    }

    /// Sometimes, arguments reference data 
    fn extract_data(&'a self, arg: &Arg) -> Option<String> {
        let Arg { name, value } = arg;
        let output = match (name.as_str(), value) {
            ("query", Some(key)) => {
                self.request.url()
                    .query_pairs()
                    .find_map(|(k, v)| match &k == key {
                        true => Some(v),
                        false => None,
                    })?
                    .into_owned()
            },
            ("query", None) => self.request.url().query()?.to_string(),
            ("wild", Some(index)) => self.request_match.wildcards[index.parse::<usize>().ok()?].to_string(),
            ("wild", None) => self.request_match.wildcards.iter().join(" "),
            ("text", Some(text)) => text.to_string(),
            (text, None) => text.to_string(),

            _other => return None,
        };

        Some(output)
    }
}

impl Into<tide::Response> for ResponseGenerator<'_> {
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


#[derive(Debug, Clone)]
pub struct Arg {
    pub name: String,
    pub value: Option<String>,
}

impl Arg {
    pub fn new(name: &str, value: Option<&str>) -> Self {
        Self {
            name: name.to_string(),
            value: value.and_then(|x| Some(x.to_string())),
        }
    }
}
