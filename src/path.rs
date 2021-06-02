/// Represents the path element of a URI
/// 
/// I want to use this type as a more flexible way of matching against incoming
/// HTTP requests. It stores paths as a list of PathNodes, an enum which at this
/// time can be either some string, or a wildcard.
///
/// The type implements PartialEq<&str> in order to facilitate matching

#[derive(Debug)]
pub enum PathNode {
    Some(String),
    Wild,
}

#[derive(Debug)]
pub struct Path (Vec<PathNode>);

impl Path {
    pub fn new(path: &str) -> Self {
        let mut output: Vec<PathNode> = vec![];
        for node in path.split("/") {
            println!("{}", node); 

            output.push( match node {
                "*" => PathNode::Wild,
                value  => PathNode::Some(value.to_string()),
            })
        }

        println!(""); 

        Path(output)
    }
}

impl PartialEq<&str> for Path {
    fn eq(&self, other: &&str) -> bool {

        let nodes = self.0.iter().zip(other.split("/"));
        
        for ( path_node, str_node ) in nodes {
            
            let eq: bool = match path_node {
               PathNode::Some(path_node) => path_node == str_node, 
               PathNode::Wild => true,
            };

            if !eq { return false; }
        }

        return true;
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
    fn equality() {
        assert_eq!(Path::new("/test/abcd"), "/test/abcd");
        assert_ne!(Path::new("/test/abc"), "/test/abcd");

        // wildcards
        assert_eq!(Path::new("/test/*"), "/test/abcd");
        assert_eq!(Path::new("/*/abcd"), "/test/abcd");
        assert_eq!(Path::new("/*/*"), "/test/abcd");
        assert_ne!(Path::new("/*/abcd"), "/test/test");

        assert_ne!(Path::new("/"), "");
           
    }
}
