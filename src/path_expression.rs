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

#[derive(Debug, Clone)]
pub struct PathMatch {
    pub wildcards: Vec<String>,
}

impl PathMatch {
    pub fn to_path(self, template: &PathExpr) -> PathBuf {

        let mut wilds = self.wildcards.iter();

        let out: String = template.inner.iter()
            .map(|node| match node {
                Node::Defined(node) => node,
                Node::Wild => wilds.next().unwrap(),
            })
            .fold(String::new(), |acc, x| acc + "/" + x);
            
        println!("{:?}", out);
        PathBuf::from(out)
    }
}

#[derive(Debug, Clone)]
pub struct PathExpr {
    inner: Vec<Node>,
}

impl PathExpr {
    pub fn new(path: &str) -> Self {

        let mut inner: Vec<Node> = path.split("/").map(Node::from_str).collect(); 
        // if inner[0] == "" { inner[0] = Node::Defined("/".to_string()) };
        Self { inner }
    }

    pub fn inner_path(&self) -> &Vec<Node> {
        return &self.inner;
    }

    // TODO: this should return a Result type with error information
    pub fn match_request( &self, request: &str ) -> Option<PathMatch> {

        let mut wildcards: Vec<String> = vec![];

        let path_nodes = request.split("/");
        let zipped_nodes = self.inner.iter().zip(path_nodes);

        for ( left, right ) in zipped_nodes {
            println!("{:?}, {:?}", left, right);

            match left {
                Node::Defined( left ) => {
                    if left != right { return None; }
                },
                Node::Wild => { wildcards.push(right.to_string()) },
            }
        };

        return Some(PathMatch{ wildcards })
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
        let out = self.inner_path().iter().skip(1)
            .fold(String::new(), |acc, x| format!("{}/{}", acc, x));

        write!(f, "{}", out)
    }
}

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
        let test_path = "/abcd/*/test";
        assert_eq!(test_path, PathExpr::new(test_path).to_string());
    }

    #[test]
    fn node_equality() {
        assert_eq!(Node::Wild, "abcd");
        assert_eq!(Node::Defined("abcd".to_string()), "abcd");
        assert_eq!(Node::Wild, "abcd");
        assert_eq!(Node::Wild, "abcd");
    }

    #[test]
    fn equality() {
        assert_eq!(PathExpr::new("/test/abcd"), "/test/abcd");
        assert_ne!(PathExpr::new("/test/abc"), "/test/abcd");

        // wildcards
        assert_eq!(PathExpr::new("/test/*"), "/test/abcd");
        assert_eq!(PathExpr::new("/*/abcd"), "/test/abcd");
        assert_eq!(PathExpr::new("/*/*"), "/test/abcd");
        assert_ne!(PathExpr::new("/*/abcd"), "/test/test");

    }

    #[test]
    /// More specific paths should not be considered equal to less specific
    /// ones with the same prefix.
    fn specificity() {
        assert_ne!(PathExpr::new("/one/two/three"), "/one/two");
        assert_ne!(PathExpr::new("/one/two"), "/one/two/three");
    }
}
