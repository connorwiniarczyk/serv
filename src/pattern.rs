/// Defines the Pattern, and ResourcePattern types, which represent the
/// first two columns of the route table. Patterns represent a set of
/// potential http requests, and ResourcePatterns represent a set of potential
/// resources on the host machine. 

use crate::Request;
use crate::request_state::RequestState;

use std::path::{Path, PathBuf};
use std::fmt;
use itertools::{Itertools, EitherOrBoth::*};


/// A pattern representing a set of http requests
#[derive(Debug, Clone)]
pub struct Pattern {
    pub path: Vec<Node>,
    
    // TODO: should be able to match against GET, POST, PUT etc.
    // pub methods: Vec<Method>,
}

impl Pattern {

    /// Check the equality of `self` and a given http request. Return an Err
    /// if they are not equal, or returns a `RequestMatch` with metadata about
    /// the match, including a `Vec` of wildcards filled in by the request.
    pub fn compare<'request>(&'request self, request: &'request Request, state: &mut RequestState) -> bool {
        let path = request.url().path_segments().unwrap(); 
        for pair in self.path.iter().zip_longest(path) {
            match pair {
                Both(left, right) => match left {
                    Node::Value(left) => if left != right { return false; },
                    Node::Variable(name) => { state.variables.insert(format!("{}",name), right.to_string()); },
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

macro_rules! route_pattern {
    (var:$node:expr) => {{
        Pattern{ path: vec![node!($node)]}
    }}
}

impl Node {
    pub fn val(input: &str) -> Self {
        Self::Value(input.to_string())
    }

    pub fn var(input: &str) -> Self {
        Self::Variable(input.to_string())
    }
}

// Implement Display for Paths and Nodes
impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Value(value) => write!(f, "{}", value),
            Self::Variable(name) => write!(f, "*{}", name),
        }
    }
}

impl fmt::Display for Pattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let nodes = self.path.iter();
        let empty = &Node::val("");
        let path = nodes.fold("".to_string(), |acc, x| format!("{}/{}", acc, x));
        write!(f, "{}", path)
    }
}
