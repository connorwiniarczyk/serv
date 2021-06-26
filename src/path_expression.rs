/// Defines the PathExpr type, which is an expression that represent one or 
/// more Paths. For example, a PathExpr of /one/*/three would be equivalent to
/// /one/two/three, /one/four/three, etc.
/// 
/// I want to use this type as a more flexible way of matching against incoming
/// HTTP requests. It stores paths as a list of Nodes, an enum which at this
/// time can be either some string, or a wildcard.
///
/// The type implements PartialEq<&str> in order to facilitate matching

use std::path::{Path, PathBuf};


#[derive(Debug, Clone)]
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

macro_rules! replace_if_wild {
    ($value:ident, take_from=$wilds:ident) => { match $value {
        Node::Defined(val) => val,
        Node::Wild => $wilds.next().unwrap(),
    }} 
}

#[derive(Debug, Clone)]
pub struct PathMatch<'a> {
    pub path: &'a PathExpr, // the path that was used to generate this PathMatch
    pub request: &'a str, // the input that was used
    pub wildcards: Vec<String>,
}

impl PathMatch<'_>{
    pub fn to_path(&self, template: &PathExpr) -> PathBuf {

        let mut wilds = self.wildcards.iter();

        // if the PathExpr template is global, make the resulting PathBuf global as well by
        // prefixing it with "/", otherwise, just use ""
        let prefix = match template.is_global {
            true => Path::new("/").to_path_buf(),
            false => Path::new("").to_path_buf(),
        };

        template.iter()
            .map(|node| replace_if_wild!(node, take_from=wilds))
            .fold(prefix, |acc, x| acc.join(&x))
    }
}

#[derive(Debug, Clone)]
pub struct PathExpr {
    is_global: bool,
    inner: Vec<Node>,
}

impl PathExpr {
    pub fn new(path: &str) -> Self {
        let is_global = path.starts_with("/"); // paths are global if they start with "/"
        let local_path = path.strip_prefix("/").unwrap_or(path); // remove the leading "/" if it exists
        let inner: Vec<Node> = local_path.split("/").map(Node::from_str).collect(); 
        Self { is_global, inner }
    }

    pub fn inner_path(&self) -> &Vec<Node> {
        return &self.inner;
    }

    pub fn iter(&self) -> std::slice::Iter<Node> {
        self.inner.iter()
    }

    // TODO: this should return a Result type with error information
    // TODO: I think PathMatch should work exclusively with &str and take a lifetime parameter
    /// Match an http request against a PathExpr
    /// If the paths match, returns a PathMatch struct with the corresponding vec of wildcards,
    /// returns None otherwise
    pub fn match_request<'a>(&'a self, request: &'a str) -> Option<PathMatch<'a>> {

        let mut wildcards: Vec<&str> = vec![];

        let path_nodes = request.strip_prefix("/").unwrap_or(request).split("/");
        let zipped_nodes = self.iter().zip(path_nodes);

        // if any left and right pair do not match, return None,
        // if the left element is a Wild, add the corresponding right element to the wildcards vec
        for (left, right) in zipped_nodes {
            match left {
                Node::Defined(left) => if left != right { return None; },
                Node::Wild => { wildcards.push(right) },
            }
        };

        // convert wildcards to a vec of owned Strings
        let owned_wildcards = wildcards
            .into_iter()
            .map(|x| x.to_string())
            .collect();

        return Some(PathMatch{ wildcards: owned_wildcards, request: &request, path: &self })
    }
}

// Implements Equality between PathExprs and Strings, with special cases
// for Wildcards included
impl PartialEq<&str> for Node {
    fn eq(&self, other: &&str) -> bool {
        match self {
            Node::Defined(path_node) => path_node == other, 
            Node::Wild => true,
        }
    }
}

// deprecated
impl PartialEq<&str> for PathExpr {
    fn eq(&self, other: &&str) -> bool {

        // Immediately return false if their lengths do not match
        // prevents /one/two from equaling /one/two/three because of the 
        // way the zip function works
        //
        // TODO: this needs to be cleaned up
        // I should try switching to the itertools crate at some point
        if other.split("/").collect::<Vec<&str>>().len() != self.inner_path().len() {
            return false;
        }

        self.inner_path().iter()
            .zip(other.split("/"))
            .all(|(x, y)| x == &y)
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

impl fmt::Display for PathExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let prefix = match self.is_global { true => "/", false => "" };
        let mut nodes = self.inner_path().iter();

        let first = nodes.next().unwrap();
        let path = nodes.fold(first.to_string(), |acc, x| format!("{}/{}", acc, x));

        write!(f, "{}{}", prefix, path)
    }
}

// ----------
// UNIT TESTS
// ----------

#[cfg(test)]
mod path_matching {
    use super::*;

    #[test]
    fn one() {
        let expr = PathExpr::new("/one/two/*");
        let out = expr.match_request("/one/two/three").unwrap();
        let new_path = out.to_path(&PathExpr::new("/*/one/two"));

        assert_eq!(new_path, PathBuf::from("/three/one/two"));
    }

    #[test]
    fn two() {
        let left = PathExpr::new("/one/*/two")
            .match_request("/one/test/two").unwrap()
            .to_path(&PathExpr::new("*"));

        let right = PathBuf::from("test");
        assert_eq!(left, right);
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn create_path() {
        PathExpr::new("/abcd/*/efg");
        PathExpr::new("/*");
        PathExpr::new("");
        PathExpr::new("abcdefg");
    }

    #[test]
    fn string_conversion() {
        assert_eq!("/one/two", PathExpr::new("/one/two").to_string());
        assert_eq!("two/three", PathExpr::new("two/three").to_string());
        assert_ne!("/two/three", PathExpr::new("two/three").to_string());
    }

    #[test]
    fn node_equality() {
        assert_eq!(Node::Wild, "abcd");
        assert_eq!(Node::Defined("abcd".to_string()), "abcd");
        assert_eq!(Node::Wild, "abcd");
        assert_eq!(Node::Wild, "abcd");
    }

    #[test]
    /// More specific paths should not be considered equal to less specific
    /// ones with the same prefix.
    fn specificity() {
        assert_ne!(PathExpr::new("/one/two/three"), "/one/two");
        assert_ne!(PathExpr::new("/one/two"), "/one/two/three");
    }
}
