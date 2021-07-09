/// Defines the PathExpr type, which is an expression that represent one or 
/// more Paths. For example, a PathExpr of /one/*/three would be equivalent to
/// /one/two/three, /one/four/three, etc.
/// 
/// I want to use this type as a more flexible way of matching against incoming
/// HTTP requests. It stores paths as a list of Nodes, an enum which at this
/// time can be either some string, or a wildcard.
///
/// The type implements PartialEq<&str> in order to facilitate matching

use crate::Request;

use std::path::{Path, PathBuf};
use itertools::Itertools;
use itertools::EitherOrBoth::*;



macro_rules! replace_if_wild {
    ($value:ident, take_from=$wilds:ident) => { match $value {
        Node::Defined(val) => val.as_str(),
        Node::Wild => $wilds.next().unwrap(),
    }} 
}


/// A pattern representing a set of http requests
#[derive(Debug, Clone)]
pub struct RequestPattern {
    pub path: Vec<Node>,
    
    // TODO: should be able to match against GET, POST, PUT etc.
    // pub methods: Vec<Method>,
}

impl RequestPattern {

    pub fn compare<'request>(&'request self, request: &'request Request) -> Result<RequestMatch<'request>, &'request str> {

        let path = request.url().path_segments().ok_or_else(|| "url path cannot be split")?;
        let mut wildcards: Vec<&str> = vec![];

        for pair in self.path.iter().zip_longest(path) {
            match pair {
                Both(left, right) => match left {
                    Node::Defined(left) => if left != right { return Err("paths do not match"); },
                    Node::Wild => { wildcards.push(right) },
                },

                _ => return Err("paths were not of the same length")
            }
        }

        let output = RequestMatch { pattern: &self, request, wildcards };
        Ok(output)
    }
}

#[derive(Debug, Clone)]
pub struct RequestMatch<'request> {
    pub pattern: &'request RequestPattern,
    pub request: &'request Request,
    pub wildcards: Vec<&'request str>,
}

#[derive(Debug, Clone)]
pub struct ResourcePattern {
    pub is_global: bool,
    pub path: Vec<Node>
}

impl ResourcePattern {
    pub fn get_path(&self, request_match: &RequestMatch) -> PathBuf {
        let mut wilds = request_match.wildcards.iter();

        // if the PathExpr template is global, make the resulting PathBuf global as well by
        // prefixing it with "/", otherwise, just use ""
        let prefix = match self.is_global {
            true => Path::new("/").to_path_buf(),
            false => Path::new("").to_path_buf(),
        };

        self.path.iter()
            .map(|node| replace_if_wild!(node, take_from=wilds))
            .fold(prefix, |acc, x| acc.join(&x))
    }
}




#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Defined(String),
    Wild,
}

impl Node {
    pub fn from_str(input: &str) -> Self {
        match input {
            "*" => Self::Wild,
            value => Self::Defined(value.to_string()),
        }
    }
}





use std::fmt;

// Implement Display for Paths and Nodes
impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Defined(value) => write!(f, "{}", value),
            Self::Wild => write!(f, "*"),
        }
    }
}

impl fmt::Display for ResourcePattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let prefix = match self.is_global { true => "/", false => "" };
        let mut nodes = self.path.iter();

        let empty = &Node::from_str("");

        let first = nodes.next().unwrap_or(empty);
        let path = nodes.fold(first.to_string(), |acc, x| format!("{}/{}", acc, x));

        write!(f, "{}{}", prefix, path)
    }
}

impl fmt::Display for RequestPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let nodes = self.path.iter();
        let empty = &Node::from_str("");
        let path = nodes.fold("".to_string(), |acc, x| format!("{}/{}", acc, x));
        write!(f, "{}", path)
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::route_parser as parse;
    
    #[test]
    fn create_path() {
        let request = parse::request("/one/two/*").expect("could not parse path '/one/two/*' ");
        let mut nodes = request.path.iter();
        
        assert_eq!(Some(&Node::from_str("one")), nodes.next());
        assert_eq!(Some(&Node::from_str("two")), nodes.next());
        assert_eq!(Some(&Node::Wild), nodes.next());
        assert_eq!(None, nodes.next());
    }
}




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
