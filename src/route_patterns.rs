/// Defines the RequestPattern, and ResourcePattern types, which represent the
/// first two columns of the route table. RequestPatterns represent a set of
/// potential http requests, and ResourcePatterns represent a set of potential
/// resources on the host machine. 

use crate::Request;
use crate::request_state::RequestState;

use std::path::{Path, PathBuf};
use std::fmt;
use itertools::{Itertools, EitherOrBoth::*};


/// A pattern representing a set of http requests
#[derive(Debug, Clone)]
pub struct RequestPattern {
    pub path: Vec<Node>,
    
    // TODO: should be able to match against GET, POST, PUT etc.
    // pub methods: Vec<Method>,
}

impl RequestPattern {

    /// Check the equality of `self` and a given http request. Return an Err
    /// if they are not equal, or returns a `RequestMatch` with metadata about
    /// the match, including a `Vec` of wildcards filled in by the request.
    pub fn compare<'request>(&'request self, request: &'request Request, state: &mut RequestState) -> bool {
        let path = request.url().path_segments().unwrap(); 
        for pair in self.path.iter().zip_longest(path) {
            match pair {
                Both(left, right) => match left {
                    Node::Value(left) => if left != right { return false; },
                    Node::Variable(name) => { state.variables.insert(format!("path:{}",name), right.to_string()); },
                },
                 _ => return false,
            }
        }

        return true
    }

    pub fn new(path: Vec<Node>) -> Self {
        Self { path }
    }
}



#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Value(String),
    Variable(String),
}

// macro_rules! node {
//     (*$name:stmt) => {{ println!("{}", $name); Node::Variable($name.to_string()) }};
//     ($name:stmt) => {{ println!("{}", $name); Node::Value($name.to_string()) }};
// }

macro_rules! route_pattern {
    (var:$node:expr) => {{
        RequestPattern{ path: vec![node!($node)]}
    }}
}


// pub(crate) use node;
// pub(crate) use route_pattern;

impl Node {
    pub fn val(input: &str) -> Self {
        Self::Value(input.to_string())
    }

    pub fn var(input: &str) -> Self {
        Self::Variable(input.to_string())
    }

    // pub fn from_str(input: &str) -> Self {
    //     match input {
    //         value => Self::Value(value.to_string()),
    //         "*" => Self::Variable("test".to_string()),
    //     }
    // }
}

///// Represents a successful match between an HTTP request and a RequestPattern,
///// and is meant to provide information about the match to the route's options.
///// Contains a list of "filled in" wildcards that can be used by the
///// ResourcePattern to generate a concrete path, or by options via the `wild`
///// argument. 
/////
///// For example: if the RequestPattern is `/one/*/three/*`, and the request has
///// a path of `/one/two/three/four`, the wildcards field will be `["two", "four"]`
/////
///// RequestMatches contain only pointers which are only valid for the lifetime
///// of the request that generated the match, hence the 'request lifetime
///// parameter
//#[derive(Debug, Clone)]
//pub struct RequestMatch<'request> {
//    pub pattern: &'request RequestPattern,
//    pub request: &'request Request,
//    pub wildcards: Vec<&'request str>,
//}

// Implement Display for Paths and Nodes
impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Value(value) => write!(f, "{}", value),
            Self::Variable(name) => write!(f, "*{}", name),
        }
    }
}

// impl fmt::Display for ResourcePattern {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         let prefix = match self.is_global { true => "/", false => "" };
//         let mut nodes = self.path.iter();

//         let empty = &Node::from_str("");

//         let first = nodes.next().unwrap_or(empty);
//         let path = nodes.fold(first.to_string(), |acc, x| format!("{}/{}", acc, x));

//         write!(f, "{}{}", prefix, path)
//     }
// }

impl fmt::Display for RequestPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let nodes = self.path.iter();
        let empty = &Node::val("");
        let path = nodes.fold("".to_string(), |acc, x| format!("{}/{}", acc, x));
        write!(f, "{}", path)
    }
}



// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::parser::route_parser as parse;
    
//     #[test]
//     fn create_path() {
//         let request = parse::request("/one/two/*").expect("could not parse path '/one/two/*' ");
//         let mut nodes = request.path.iter();
        
//         assert_eq!(Some(&Node::from_str("one")), nodes.next());
//         assert_eq!(Some(&Node::from_str("two")), nodes.next());
//         assert_eq!(Some(&Node::Wild), nodes.next());
//         assert_eq!(None, nodes.next());
//     }
// }




//// Implements Equality between PathExprs and Strings, with special cases
//// for Wildcards included
//impl PartialEq<&str> for Node {
//    fn eq(&self, other: &&str) -> bool {
//        match self {
//            Node::Defined(path_node) => path_node == other, 
//            Node::Wild => true,
//        }
//    }
//}

//// deprecated
//impl PartialEq<&str> for PathExpr {
//    fn eq(&self, other: &&str) -> bool {

//        // Immediately return false if their lengths do not match
//        // prevents /one/two from equaling /one/two/three because of the 
//        // way the zip function works
//        //
//        // TODO: this needs to be cleaned up
//        // I should try switching to the itertools crate at some point
//        if other.split("/").collect::<Vec<&str>>().len() != self.inner_path().len() {
//            return false;
//        }

//        self.inner_path().iter()
//            .zip(other.split("/"))
//            .all(|(x, y)| x == &y)
//    }

//}
