/// Defines the Pattern, and ResourcePattern types, which represent the
/// first two columns of the route table. Patterns represent a set of
/// potential http requests, and ResourcePatterns represent a set of potential
/// resources on the host machine. 

use crate::request_state::RequestState;

use std::path::{Path, PathBuf};
use std::fmt;
use itertools::{Itertools, EitherOrBoth::*};
use hyper::{Request, Body};
use Node::*;

use std::collections::HashSet;
use hyper::Method;

/// A pattern representing a set of http requests
#[derive(Debug, Clone)]
pub struct Pattern {

    pub name: Option<String>,
    pub methods: HashSet<Method>,

    pub attributes: Vec<String>,
    pub path: Vec<Node>,
    pub extension: Option<Node>,
}

use std::collections::HashMap;
type Vars = HashMap<String, String>;

impl Pattern {

    pub fn new(mut path: Vec<Node>) -> Self {
        path.insert(0, Node::val(""));
        Self { name: None, methods: HashSet::new(), path, attributes: vec![], extension: None }
    }

    /// Check the equality of `self` and a given http request. Return an Err
    /// if they are not equal, or returns a `RequestMatch` with metadata about
    /// the match, including a `Vec` of wildcards filled in by the request.
    pub fn compare<'request>(&'request self, request: &'request Request<Body>) -> Result<Vars, ()> {

        let mut output = Vars::new();

        // check to see that the method is valid
        if !(self.methods.len() == 0 || self.methods.contains(request.method())) { return Err(()) }

        // if let Some(methods) = &self.methods {
        //     if methods.contains(request.method()) == false { return Err(()) }
        // }

        let path_full = request.uri().path();

        // split the path by the rightmost '.' to get the extension if one exists
        let (path, mut ext) = path_full.rsplit_once(".")
            .map(|(p, e)| (p, Some(e)))
            .unwrap_or((path_full, None));

        // do piecewise comparisons of nodes in the pattern with nodes in the request path.
        let mut path_iter = path.split("/");
        for node in self.path.iter() {
            match node {
                Value(v) if path_iter.next().ok_or(())? != v => return Err(()), 
                Value(v) => (), 
                Variable(key) => {
                    let value = path_iter.next().ok_or(())?;
                    output.insert(key.to_string(), value.to_string());
                },
                Rest(key) => {
                    let mut value = path_iter.join("/");

                    // handles the extension
                    match (self.extension.as_ref(), ext) {
                        // The Rest variable should consume the extension if no extension is
                        // specified in the pattern 
                        (None, Some(extension)) => {
                            ext = None; 
                            value.push_str(".");
                            value.push_str(&extension);
                        },
                        // otherwise, do nothing
                        _ => (),
                    }

                    output.insert(key.to_string(), value.to_string());
                }, 
            }
        }

        // make sure the request path is not longer than the pattern
        if path_iter.next() != None { return Err(()) }

        match (self.extension.as_ref(), ext) {
            (Some(Value(l)), Some(r)) if l == r => (),
            (Some(Value(l)), Some(r)) => return Err(()),
            (Some(Value(l)), None) => return Err(()),

            (Some(Variable(key)), None) => return Err(()),
            (Some(Variable(key)), Some(r)) => {output.insert(key.to_string(), r.to_string());},

            (None, Some(_)) => return Err(()), 
            (None, None) => (),

            // extensions should not be allowed to be of type Rest
            (Some(Rest(key)), _) => unreachable!(),

        }

        Ok(output)
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

    pub fn from_str(input: &str) -> Self {
        if let Some(value) = input.strip_prefix("**") { 
            return Self::rest(value);
        } else if let Some(value) = input.strip_prefix("*") {
            return Self::var(value);
        } else {
            return Self::val(input);
        }
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

        for attr in self.attributes.iter() {
            f.write_str("@")?;
            write!(f, "{}", attr)?;
        }
        
        write!(f, "{}", self.path.iter().join("/"))?;

        if let Some(ext) = &self.extension {
            f.write_str(".")?;
            write!(f, "{}", ext)?;
        }

        Ok(())

    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pattern_compare() {
        let pattern = Pattern {
            attributes: vec![],
            path: vec![],
            extension: vec![],
        };
    }
}
