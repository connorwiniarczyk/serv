/// Defines the PathExpr type, which is an expression that represent one or 
/// more Paths. For example, a PathExpr of /one/*/three would be equivalent to
/// /one/two/three, /one/four/three, etc.
/// 
/// I want to use this type as a more flexible way of matching against incoming
/// HTTP requests. It stores paths as a list of Nodes, an enum which at this
/// time can be either some string, or a wildcard.
///
/// The type implements PartialEq<&str> in order to facilitate matching

#[derive(Debug, Clone)]
pub enum Node {
    Defined(String),
    Wild,
}

#[derive(Debug, Clone)]
pub struct PathExpr {
    inner: Vec<Node>,
}

impl PathExpr {
    pub fn new(path: &str) -> Self {
        let mut output: Vec<Node> = vec![];
        for node in path.split("/") {
            output.push( match node {
                "*" => Node::Wild,
                value  => Node::Defined(value.to_string()),
            })
        }

        PathExpr{ inner: output }
    }

    pub fn inner_path(&self) -> &Vec<Node> {
        return &self.inner;
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

impl PartialEq<Self> for Node {
    fn eq(&self, other: &Self) -> bool {
        true
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
