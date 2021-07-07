/// A Resolver is a type that can be turned into a tide HTTP response by
/// calling its resolve() method. Resolver types are defined as an enum

use async_process::Command;
use crate::config::Config;
use regex::Regex;
use lazy_static::lazy_static;
use std::iter::Peekable;

use std::collections::HashMap;

use crate::route_table::*;

pub struct RouteOption<'a> {
    args: Vec<Arg>,
    func: fn(Response<'a>, &Vec<Arg>) -> Response<'a>,
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

    type Access<'a> = fn(Response<'a>, &Vec<Arg>) -> Response<'a>;
    // type Access = fn(Response, &Vec<Arg>) -> Response;

    pub fn exec<'a>(input: Response<'a>, args: &Vec<Arg>) -> Response<'a> { todo!(); }
    pub fn read<'a>(input: Response<'a>, args: &Vec<Arg>) -> Response<'a> { todo!(); }

    pub fn get_func(input: &str) -> Access {
        match input {
            "exec" => exec,
            "read" => read,
            _ => panic!(),
        }
    }
}


pub struct Response<'a> {
    pub route: &'a Route,

    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub status: u32,
}

impl<'a> Response<'a> {
    pub fn new(body: Vec<u8>, route: &'a Route) -> Self  {
        Self {
            route,
            headers: HashMap::new(),
            body: body.clone(),
            status: 200
        }
    }

    pub fn with_header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string()); self
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


//     #[test]
//     /// Option strings that don't specify an access type should default to an access type of Read,
//     /// even if other post processors are specified
//     fn implicit_read() {
//        let left = Options::from_str("header(access-control-allow-origin:*)");

//        match left.access_type {
//             Access::Exec(_) => panic!("access type should be Read"),
//             Access::Read => ()
//        };

//        assert_eq!(left.post_processors.len(), 1);
//     }
// }

