/// A Resolver is a type that can be turned into a tide HTTP response by
/// calling its resolve() method. Resolver types are defined as an enum

use async_process::Command;
use crate::config::Config;
use regex::Regex;
use lazy_static::lazy_static;

use std::collections::HashMap;

// define regular expressions for the module
lazy_static! {
   static ref OPTS: Regex = Regex::new(r"(?m:(?P<option>\w+)(?:\((?P<args>.*?)\))?)+").unwrap();
   static ref ARGS: Regex = Regex::new(r"(?P<arg>[a-zA-Z0-9]+)(?::(?P<value>[a-zA-Z0-9]+))?").unwrap();
   static ref ARG: Regex = Regex::new(r"(?P<arg>[a-zA-Z0-9]+)(?::(?P<value>[a-zA-Z0-9]+))?").unwrap();
}

pub struct Response {
    pub headers: HashMap<String, String>,
    pub body: String,
    pub status: u32,
}

impl Response {
    pub fn new(body: &str) -> Self {
        Self {
            headers: HashMap::new(),
            body: body.to_string(),
            status: 200
        }
    }

    pub fn with_header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string()); 
        self
    }
}




use std::ops::*;
use std::sync::Arc;

type F = Arc<dyn Fn(Response) -> Response + Send + Sync>;
// type F = fn(Response) -> Response;


#[derive(Clone)]
pub struct Processor {
    pub func: F,
}

impl Processor {
    pub fn from_str(input: &str) -> Option<Self> {
        todo!();
    }

    fn header(args: Vec<Arg>) -> F {
        todo!();
    }

    pub fn from_capture(input: regex::Captures) -> Option<Self> {
        let name = input.name("option")?.as_str();
        let args = input.name("args").and_then(|x| Some(x.as_str()));

        let test = "test".to_string();

        let func: F = match name {
            "header" => Arc::new(|x: Response| {
                let abcd = Box::new(test.clone());
                x.with_header("test", &abcd)
            }),
            _ => Arc::new(|x| x),
        };

        // let output = Self { func: Box::new(func) };
        let output = Self { func };
        Some(output)
    }

    pub fn apply(&self, input: Response) -> Response {
        (self.func)(input) 
    }
}

#[derive(Debug, Clone)]
pub struct Arg {
    pub name: String,
    pub value: Option<String>,
}

impl Arg {
    /// argname:value -> Some( name: argname, value Some(value) )
    pub fn from_str(input: &str) -> Option<Self> {
        let captures = ARG.captures(input)?;
        let name = captures.name("arg")?.as_str().to_string();
        let value = captures.name("value").and_then(|x| Some(x.as_str().to_string()));

        println!("{:?} {:?}", name, value);
        Some(Self { name, value })

    }

    pub fn new(name: &str, value: &str) -> Self {
        match value {
            "" => Self { name: name.to_string(), value: None },
            _  => Self { name: name.to_string(), value: Some(value.to_string()) },
        }
    }
}


#[derive(Debug, Clone)]
pub enum Access {
    Read,
    Exec(Vec<Arg>),
}

#[derive(Clone)]
pub struct Options {
    pub access_type: Access,
    pub post_processors: Vec<Processor>, 
}


impl Options {
    
    fn get_args(input: &str) -> Vec<Arg> {
        input.split(", ").map(Arg::from_str).filter_map(|x| x).collect()
    }

    /// Attempt to read the Access Type from the list of options. If one is declared explicitly,
    /// meaning that the first option is either "exec" or "read", return the corresponding member
    /// of the Access Enum and increment the iterator so as to remove it. If it is not declared
    /// explicitly, do not modify the iterator and return None. The caller of this function will
    /// be responsible for assigning a default.
    pub fn get_explicit_access_type(input: &mut regex::CaptureMatches) -> Option<Access> {

        let mut iter = input.peekable();
        let first = iter.peek()?;

        let out = match first.name("option")?.as_str() {
            "exec" => {
                // Parse the arguments to exec() if there are any
                // TODO: this needs to be cleaner
                let args = Self::get_args(first.name("args").and_then(|x| Some(x.as_str())).unwrap_or(""));
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

        let mut captures = OPTS.captures_iter(input); 
        let access_type = Self::get_explicit_access_type(&mut captures).unwrap_or(Access::Read);

        let post_processors = captures
            .filter_map(Processor::from_capture);

        todo!();
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn processor_from_str() {
        Options::from_str("exec(abcd) abcd(test)");
    }

    #[test]
    fn two() {
       let left = Arg::from_str("test").unwrap();
       assert_eq!(left.name, "test");
    }


    #[test]
    fn processor() {
       let proc = Processor::from_str("").unwrap(); 
       let response = Response::new("test");

       let new = proc.apply(response);
       let new = proc.apply(new);
       let new = proc.apply(new);
       let new = proc.apply(new);
       let new = proc.apply(new);


       panic!("{:?}", new.body);
    }
}
