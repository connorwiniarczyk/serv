/// Defines the Pattern, and ResourcePattern types, which represent the
/// first two columns of the route table. Patterns represent a set of
/// potential http requests, and ResourcePatterns represent a set of potential
/// resources on the host machine. 

// use crate::Request;
use crate::request_state::RequestState;

use std::path::{Path, PathBuf};
use std::fmt;
use itertools::{Itertools, EitherOrBoth::*};

use hyper::{Request, Body};


/// A pattern representing a set of http requests
#[derive(Debug, Clone)]
pub struct Pattern {
    pub path: Vec<Node>,
    
    // TODO: should be able to match against GET, POST, PUT etc.
    // pub methods: Vec<Method>,
}

use std::collections::HashMap;
type Vars = HashMap<String, String>;

impl Pattern {

    /// Check the equality of `self` and a given http request. Return an Err
    /// if they are not equal, or returns a `RequestMatch` with metadata about
    /// the match, including a `Vec` of wildcards filled in by the request.
    pub fn compare<'request>(&'request self, request: &'request Request<Body>) -> Result<Vars, ()> {
        let path = request.uri().path().split("/"); 
        let mut vars = Vars::new();

        for pair in self.path.iter().zip_longest(path) {
            match pair {
                Both(pattern, value) => match pattern {

                    // if the node is a value , check for equality
                    Node::Value(pattern) => if pattern != value { return Err(()); },

                    // if the node is a variable, equality is implied, but we need to update the
                    // variables with the value of the request at the same spot
                    Node::Variable(name) => {
                        vars.insert(name.to_string(), value.to_string());
                    },

                    Node::Rest(name) => {
                        let rest = request.uri().path().split("/").skip_while(|x| x != &value).join("/");
                        vars.insert(name.to_string(), rest);
                        return Ok(vars)
                    }
                },
                 _ => return Err(()),
            }
        }

        return Ok(vars);
    }

    pub fn new(mut path: Vec<Node>) -> Self {
        path.insert(0, Node::val(""));
        Self { path }
    }
}



#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Value(String),
    Variable(String),
    Rest(String),
}

impl Node {
    pub fn val(input: &str) -> Self {
        Self::Value(input.to_string())
    }

    pub fn var(input: &str) -> Self {
        Self::Variable(input.to_string())
    }

    pub fn rest(input: &str) -> Self {
        Self::Rest(input.to_string())
    }
}

// Implement Display for Paths and Nodes
impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Value(value) => write!(f, "{}", value),
            Self::Variable(name) => write!(f, "*{}", name),
            Self::Rest(name) => write!(f, "**{}", name),
        }
    }
}

impl fmt::Display for Pattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path.iter().join("/"))
    }
}
