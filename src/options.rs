/// A Resolver is a type that can be turned into a tide HTTP response by
/// calling its resolve() method. Resolver types are defined as an enum

use crate::config::Config;
use regex::Regex;
use lazy_static::lazy_static;
use std::iter::Peekable;

use std::collections::HashMap; use crate::route_table::*;

use tide::http::Url;

use std::process::Command;

type OptionFunc = for<'a> fn(ResponseGenerator<'a>, &Vec<Arg>) -> ResponseGenerator<'a>;

#[derive(Clone)]
pub struct RouteOption {
    args: Vec<Arg>,
    func: OptionFunc,
}

impl RouteOption {
    pub fn new(func: &str, args: Vec<Arg>) -> Self {
        let func = access_types::get_func(func);
        Self { func, args }
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

mod access_types { 
    use super::*;
    
    use std::fs;

    option!( exec(input, args) => {
        let path = input.path_match.to_path(&input.route.resource);

        // get url query from request
        let query = HttpQuery::from_url(input.request.url());

        let rendered_args: Vec<&str> = args.iter().map(| Arg{ name, value } | match (name.as_str(), value){
            ("query", Some(param)) => query.get(param).unwrap_or(""),
            ("wild", Some(index)) => &input.path_match.wildcards[index.parse::<usize>().unwrap()],
            ("text", Some(value)) => value,
            (_, _) => "",
        }).collect();

        let output_raw = Command::new(&path).args(rendered_args).output().unwrap();
        let output = output_raw.stdout;

        input.body = output;

        return input
    });

    option!( read(input) => {
        let path = input.path_match.to_path(&input.route.resource);
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

use crate::path_expression::PathMatch;


pub struct ResponseGenerator<'a> {
    pub route: &'a Route,
    pub path_match: &'a PathMatch<'a>,
    pub request: &'a crate::Request,

    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub status: u16,
}

impl<'a> ResponseGenerator<'a> {
    pub fn new(path_match: &'a PathMatch, route: &'a Route, request: &'a crate::Request) -> Self  {
        Self {
            path_match,
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


//#[cfg(test)]
// mod test_option_parsing {
//     use super::*;

//     #[test]
//     fn nominal_case() {
//         let value = Options::from_str("exec(wild:0 wild:1) header(content-type:text/html)");
//         let access_type = value.access_type;

//         // check that the access type is Exec, and not Read
//         let access_args = match access_type {
//             Access::Exec( args ) => args,
//             Access::Read => panic!("wrong access type!"),
//         };

//         // check that there are exactly two arguments
//         assert_eq!(access_args.len(), 2);

//         // Check that both args are of type "wild" and have a value
//         for arg in access_args {
//             assert_eq!(&arg.name, "wild");
//             arg.value.unwrap();
//         }

//         let processors = value.post_processors;
//         assert_eq!(processors.len(), 1);

//         let processor = processors.into_iter().next().expect("The first processor is empty");
//         let arg = processor.args.into_iter().next().expect("The first argument is empty");

//         assert_eq!(&arg.name, "content-type");
//         assert_eq!(&arg.value.expect("arg value is none"), "text/html");
//     }

//     #[test]
//     /// Empty option strings should parse into an access type of Read, and zero post processors
//     fn empty_case() {
//        let left = Options::from_str("");

//        match left.access_type {
//             Access::Exec(_) => panic!("access type should be Read"),
//             Access::Read => ()
//        };

//        assert_eq!(left.post_processors.len(), 0);
//     }

