/// A Resolver is a type that can be turned into a tide HTTP response by
/// calling its resolve() method. Resolver types are defined as an enum

use async_process::Command;
use crate::config::Config;
use regex::Regex;
use lazy_static::lazy_static;
use std::iter::Peekable;

use std::collections::HashMap;


pub enum OptionKind {
    Access(fn(String, &Vec<Arg>) -> Response),
    Process(fn(Response, &Vec<Arg>) -> Response),
}

pub struct RouteOption {
    args: Vec<Arg>,
    func: OptionKind,
}

impl RouteOption {
    pub fn new(func: &str, args: Vec<Arg>) -> Self {
        let func = access_types::get_func(func);

        Self {
            func:OptionKind::Access(func), args
        }
    }
}

mod access_types { 
    use super::*;

    type Access = fn(String, &Vec<Arg>) -> Response;

    pub fn exec(input: String, args: &Vec<Arg>) -> Response { todo!(); }
    pub fn read(input: String, args: &Vec<Arg>) -> Response { todo!(); }

    pub fn get_func(input: &str) -> Access {
        match input {
            "exec" => exec,
            "read" => read,
            _ => panic!(),
        }
    }
}



// define regular expressions for the module
lazy_static! {
   static ref OPTS: Regex = Regex::new(r"(?m:(?P<option>\w+)(?:\((?P<args>.*?)\))?)+").unwrap();
   static ref ARGS: Regex = Regex::new(r"(?P<arg>[a-zA-Z0-9]+)(?::(?P<value>[a-zA-Z0-9]+))?").unwrap();
   static ref ARG: Regex = Regex::new(r"(?P<arg>[^:\s]+)(?::(?P<value>[^:\s]+))?").unwrap();
}

macro_rules! capture_get {
    ($capture:ident, $key:expr) => {
        $capture.name($key)
            .and_then(|x| Some(x.as_str())) // If the result is Some, convert the inner type to a string
            .unwrap_or("") // If the result is None, replace it with an empty string
    }
}

pub struct Response {
    pub headers: HashMap<String, String>,
    // pub body: String,
    pub body: Vec<u8>,
    pub status: u32,
}

impl Response {
    pub fn new(body: &Vec<u8>) -> Self {
        Self {
            headers: HashMap::new(),
            body: body.clone(),
            status: 200
        }
    }

    pub fn with_header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string()); self
    }
}

type F = fn(Response, &Vec<Arg>) -> Response;

#[derive(Clone)]
pub struct Processor {
    pub args: Vec<Arg>,
    pub func: F,
}

mod processor_functions {
    use super::*;

    pub fn set_header(input: Response, args: &Vec<Arg>) -> Response {
        args.into_iter().fold(input, |response, arg| match arg {
            Arg { name, value: Some(value) } => response.with_header(name, value),
            Arg { name, value: None } => response,
        })
    }
}

impl Processor {
    pub fn from_str(input: &str) -> Option<Self> {
        Self::from_capture(OPTS.captures(input)?)
    }

    pub fn from_capture(input: regex::Captures) -> Option<Self> {
        let name = input.name("option")?.as_str();
        let args = Arg::parse(capture_get!(input, "args"));
        let output = match name {
            "header" => Self { func: processor_functions::set_header, args },
            "cors" => Self { func: processor_functions::set_header, args: Arg::parse("access-control-allow-origin:*")},
            _ => return None,
        };

        Some(output)
    }

    pub fn apply(&self, input: Response) -> Response {
        (self.func)( input, &self.args )
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
    /// parse an individual argument string into an Arg
    pub fn from_str(input: &str) -> Option<Self> {
        let captures = ARG.captures(input)?;
        Self::from_capture(captures)
    }

    /// parse a whole argstring into a Vec of Args
    pub fn parse(input: &str) -> Vec<Self> {
        ARG.captures_iter(input).filter_map(Self::from_capture).collect()
    }

    pub fn from_capture(input: regex::Captures) -> Option<Self> {
        let name = input.name("arg")?.as_str().to_string();
        let value = input.name("value").and_then(|x| Some(x.as_str().to_string()));
        Some(Self { name, value })
    }
}


#[derive(Debug, Clone)]
pub enum Access {
    Read,
    Exec(Vec<Arg>),
}

use crate::State;
use crate::path_expression::PathMatch;
use tide::Request;

impl Access {

    // TODO: I think it would be useful to move some code from the Route::resolve method to here,
    // but I'm not sure how to do it best
    fn exec(path: &PathMatch, args: &Vec<Arg>, request: &Request<State>) -> Option<Response> {
        todo!();
    }

    pub fn apply(&self, path_match: &PathMatch, request: &Request<State>) -> Option<Response> {
        todo!();
    }
}

#[derive(Clone)]
pub struct Options {
    pub access_type: Access,
    pub post_processors: Vec<Processor>, 
}



impl Options {

    /// Attempt to read the Access Type from the list of options. If one is declared explicitly,
    /// meaning that the first option is either "exec" or "read", return the corresponding member
    /// of the Access Enum and increment the iterator so as to remove it. If it is not declared
    /// explicitly, do not modify the iterator and return None. The caller of this function will
    /// be responsible for assigning a default.
    pub fn get_explicit_access_type(input: &mut Peekable<regex::CaptureMatches>) -> Option<Access> {

        let first = input.peek()?;
        let out = match first.name("option")?.as_str() {
            "exec" => {
                // Parse the arguments to exec() if there are any
                let args = Arg::parse(capture_get!(first, "args"));
                Access::Exec(args)
            },
            "read" => Access::Read,
            _      => return None,
        };

        // If an explicit Access Type was found, increment the iterator to remove it before
        // returning
        input.next();
        Some(out)
    }

    pub fn from_str(input: &str) -> Self {

        let mut captures = OPTS.captures_iter(input).peekable(); 
        let access_type = Self::get_explicit_access_type(&mut captures).unwrap_or(Access::Read);

        let post_processors = captures
            .filter_map(Processor::from_capture)
            .collect();

        Self { access_type, post_processors }
    }
}



#[cfg(test)]
mod test_option_parsing {
    use super::*;

    #[test]
    fn nominal_case() {
        let value = Options::from_str("exec(wild:0 wild:1) header(content-type:text/html)");
        let access_type = value.access_type;

        // check that the access type is Exec, and not Read
        let access_args = match access_type {
            Access::Exec( args ) => args,
            Access::Read => panic!("wrong access type!"),
        };

        // check that there are exactly two arguments
        assert_eq!(access_args.len(), 2);

        // Check that both args are of type "wild" and have a value
        for arg in access_args {
            assert_eq!(&arg.name, "wild");
            arg.value.unwrap();
        }

        let processors = value.post_processors;
        assert_eq!(processors.len(), 1);

        let processor = processors.into_iter().next().expect("The first processor is empty");
        let arg = processor.args.into_iter().next().expect("The first argument is empty");

        assert_eq!(&arg.name, "content-type");
        assert_eq!(&arg.value.expect("arg value is none"), "text/html");
    }

    #[test]
    /// Empty option strings should parse into an access type of Read, and zero post processors
    fn empty_case() {
       let left = Options::from_str("");

       match left.access_type {
            Access::Exec(_) => panic!("access type should be Read"),
            Access::Read => ()
       };

       assert_eq!(left.post_processors.len(), 0);
    }


    #[test]
    /// Option strings that don't specify an access type should default to an access type of Read,
    /// even if other post processors are specified
    fn implicit_read() {
       let left = Options::from_str("header(access-control-allow-origin:*)");

       match left.access_type {
            Access::Exec(_) => panic!("access type should be Read"),
            Access::Read => ()
       };

       assert_eq!(left.post_processors.len(), 1);
    }
}

