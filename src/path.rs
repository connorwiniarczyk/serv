/// Represents the path element of a URI
/// 
/// I want to use this type as a more flexible way of matching against incoming
/// HTTP requests. It stores paths as a list of PathNodes, an enum which at this
/// time can be either some string, or a wildcard.
///
/// The type implements PartialEq<&str> in order to facilitate matching

#[derive(Debug)]
pub enum PathNode {
    Defined(String),
    Wild,
}


#[derive(Debug)]
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
        self.inner_path().iter()
            .zip(other.split("/"))
            .all(|(x, y)| x == &y)
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
}
