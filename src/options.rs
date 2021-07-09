
use std::collections::HashMap;
use crate::route_table::*;
use tide::http::Url;
use std::process::Command;

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

    pub fn apply<'a>(&self, response: ResponseGenerator<'a>) -> ResponseGenerator<'a> {
        (self.func)(response, &self.args)
    }
}

macro_rules! option {
    ($name:ident($input:ident, $args:ident) => $func:block) => {
        pub fn $name<'a>(mut $input: ResponseGenerator<'a>, $args: &Vec<Arg>) -> ResponseGenerator<'a> {
            $func
        }
    };

    ($name:ident($input:ident) => $func:block) => {
        pub fn $name<'a>(mut $input: ResponseGenerator<'a>, _args: &Vec<Arg>) -> ResponseGenerator<'a> {
            $func
        }
    };
}

pub mod access_types { 
    use super::*;
    use std::fs;

    option!( exec(input, args) => {
        let path = input.route.resource.get_path(input.request_match);

        // get url query from request
        let query = HttpQuery::from_url(input.request.url());

        let rendered_args: Vec<&str> = args.iter().map(| Arg{ name, value } | match (name.as_str(), value){
            ("query", Some(param)) => query.get(param).unwrap_or(""),
            ("wild", Some(index)) => &input.request_match.wildcards[index.parse::<usize>().unwrap()],
            ("text", Some(value)) => value,
            (_, _) => "",
        }).collect();

        let output_raw = Command::new(&path).args(rendered_args).output().unwrap();
        let output = output_raw.stdout;

        input.body = output;

        return input
    });

    option!( read(input) => {
        let path = input.route.resource.get_path(input.request_match);
        let body: Vec<u8> = fs::read(&path).unwrap_or_default();
        input.body = body;

        return input
    });


    option!( header(input, args) => {
        args.into_iter().fold(input, |response, arg| match arg {
            Arg { name, value: Some(value) } => response.with_header(name, value),
            Arg { name, value: None } => response,
        })     
    });


    pub fn get_func(input: &str) -> OptionFunc {
        match input {
            "exec" => exec,
            "read" => read,
            "header" => header,
            _ => panic!(),
        }
    }


    // ------------
    // HELPER TYPES
    // ------------
    pub struct HttpQuery {
        inner: HashMap<String, String>,
    }

    impl HttpQuery {
        pub fn from_url(url: &Url) -> Self {

            let mut output: HashMap<String, String> = HashMap::new();
            let pairs = url.query_pairs();
            for ( left, right ) in pairs {
                output.insert(left.into_owned(), right.into_owned());
            }
            Self { inner: output }
        }

        pub fn get(&self, key: &str) -> Option<&str> {
           self.inner.get(key).and_then(|x| Some(x.as_str()))
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
