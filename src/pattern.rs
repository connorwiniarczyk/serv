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


/// A pattern representing a set of http requests
#[derive(Debug, Clone)]
pub struct Pattern {
    pub attributes: Vec<String>,
    pub path: Vec<Node>,
    pub extension: Option<Node>,
}

use std::collections::HashMap;
type Vars = HashMap<String, String>;

impl Pattern {

    /// Check the equality of `self` and a given http request. Return an Err
    /// if they are not equal, or returns a `RequestMatch` with metadata about
    /// the match, including a `Vec` of wildcards filled in by the request.
    pub fn compare<'request>(&'request self, request: &'request Request<Body>) -> Result<Vars, ()> {

        let mut output = Vars::new();

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
        

        // match (self.extension.as_ref(), ext) {
        //     (Some(node), Some(e)) => node.compare(&mut std::iter::once(e), &mut output)?,
        //     (None, None) => (),
        //     _ => return Err(()),
        // };

        // let mut path_iter = path.split("/");

        // let result: Result<(), ()> = self.path.iter()
        //     .map(|node| node.compare(&mut path_iter, &mut output))
        //     .collect();

        // if let Some(_) = path_iter.next() {
        //     return Err(());
        // } 

        // result.map(|_| output)
    }

    pub fn new(mut path: Vec<Node>) -> Self {
        path.insert(0, Node::val(""));
        Self { path, attributes: vec![], extension: None }
    }
}



#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Value(String),
    Variable(String),
    Rest(String),
}

impl Node {
    pub fn compare<'a, I: Iterator<Item = &'a str>>(&'a self, path: &mut I, vars: &mut HashMap<String, String>) -> Result<(), ()> {

        let next: &str = path.next().ok_or(())?;

        match self {
            Value(value) => match value == next {
                true => Ok(()),
                false => Err(()),
            },
            Variable(name) => {
                vars.insert(name.to_string(), next.to_string());
                Ok(())

            },
            Rest(name) => {
                vars.insert(name.to_string(), path.join("/"));
                Ok(())
            }
        }
    }

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
