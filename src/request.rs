/// Represents the path element of a URI
/// 
/// I want to use this type as a more flexible way of matching against incoming
/// HTTP requests. It stores paths as a list of PathNodes, an enum which at this
/// time can be either some string, or a wildcard.
///
/// The type implements PartialEq<&str> in order to facilitate matching

#[derive(Debug, Clone)]
pub enum PathNode {
    Defined(String),
    Wild,
}

#[derive(Debug, Clone)]
pub struct Path (Vec<PathNode>);

impl Path {
    pub fn new(path: &str) -> Self {
        let mut output: Vec<PathNode> = vec![];
        for node in path.split("/") {
            output.push( match node {
                "*" => PathNode::Wild,
                value  => PathNode::Defined(value.to_string()),
            })
        }

        Path(output)
    }
    pub fn inner_path(&self) -> &Vec<PathNode> {
        return &self.0;
    }
}

// Implements Equality between Paths and Strings, with special cases
// for Wildcards included
impl PartialEq<&str> for PathNode {
    fn eq(&self, other: &&str) -> bool {
        match self {
            PathNode::Defined(path_node) => path_node == other, 
            PathNode::Wild => true,
        }
    }
}

impl PartialEq<&str> for Path {
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

// Implement Display for Paths and PathNodes
impl fmt::Display for PathNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Defined(value) => write!(f, "{}", value),
            Self::Wild => write!(f, "*"),
        }
    }
}

impl fmt::Display for Path {
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
        Path::new("/abcd/*/efg");
        Path::new("/*");
        Path::new("");
        Path::new("abcdefg");
    }

    #[test]
    fn string_conversion() {
        let test_path = "/abcd/*/test";
        assert_eq!(test_path, Path::new(test_path).to_string());
    }

    #[test]
    fn node_equality() {
        assert_eq!(PathNode::Wild, "abcd");
        assert_eq!(PathNode::Defined("abcd".to_string()), "abcd");
        assert_eq!(PathNode::Wild, "abcd");
        assert_eq!(PathNode::Wild, "abcd");
    }

    #[test]
    fn equality() {
        assert_eq!(Path::new("/test/abcd"), "/test/abcd");
        assert_ne!(Path::new("/test/abc"), "/test/abcd");

        // wildcards
        assert_eq!(Path::new("/test/*"), "/test/abcd");
        assert_eq!(Path::new("/*/abcd"), "/test/abcd");
        assert_eq!(Path::new("/*/*"), "/test/abcd");
        assert_ne!(Path::new("/*/abcd"), "/test/test");

    }

    #[test]
    /// More specific paths should not be considered equal to less specific
    /// ones with the same prefix.
    fn specificity() {
        assert_ne!(Path::new("/one/two/three"), "/one/two");
        assert_ne!(Path::new("/one/two"), "/one/two/three");
    }
}
