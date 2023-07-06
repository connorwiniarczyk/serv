
use std::collections::HashMap;


#[derive(Hash, Eq, PartialEq, Clone)]
pub enum PathNode {
    Literal(String),
    Wildcard(String),
    Doublewildcard(String),
}

use PathNode::*;

struct RouteTreeNode<A> {
    children: HashMap<PathNode, RouteTreeNode<A>>,

    literals: HashMap<String, PathNode>,
    wildcards: Vec<(String, PathNode)>,
    doublewildcard: Option<String>,

    value: Option<A>,
}

impl<A> RouteTreeNode<A> {
    fn new() -> Self {
        Self {
            children: HashMap::new(),
            literals: HashMap::new(),
            wildcards: Vec::new(),
            doublewildcard: None,
            value: None,
        }
    }

    fn insert<'a, 'b, I>(&'a mut self, iter: &mut I, value: A) -> Result<(), &'static str>
    where I: Iterator<Item = &'b PathNode> {
        let next = iter.next();
        match (next, self.value.is_some()) {
            (None, false) => { self.value = Some(value); },
            (None, true) => { return Err("entry already exists"); }
            (Some(n), _) => {
                let child = self.children.entry(n.clone()).or_insert(Self::new());
                match n {
                    Literal(s) => { self.literals.insert(s.to_owned(), n.clone()); },
                    Wildcard(s) => todo!(),
                    Doublewildcard(s) => todo!(),
                };
                child.insert(iter, value)?;
            },
        }

        return Ok(())
    }

    fn get<'a, 'b, I>(&'a self, iter: &mut I) -> Option<&'a A>
    where I: Iterator<Item = &'b str> + Clone {
        let next = iter.next();

        match (next, &self.value) {
            (None, None) => { return None },
            (None, Some(v)) => { return Some(v) },
            (Some(n), _) => {
                if let Some(v) = self.literals.get(n) {
                    let child = self.children.get(v).unwrap();
                    return child.get(iter);
                }

                for (name, node) in self.wildcards.iter() {
                    if let Some(v) = self.children.get(&node).unwrap().get(iter) {
                        return Some(v)
                    }
                }

                todo!();
            },
        }
    }
}

pub struct RouteTree<A>(RouteTreeNode<A>);

impl<A> RouteTree<A> {
    pub fn new() -> Self {
        Self(RouteTreeNode::new())
    }

    pub fn insert<'a, 'b, I>(&'a mut self, path: &mut I, value: A) -> Result<(), &'static str>
    where I: Iterator<Item = &'b PathNode> {
        self.0.insert(path, value)?;

        return Ok(())
    }

    pub fn insert_vec(&mut self, path: Vec<PathNode>, value: A) -> Result<(), &'static str> {
        let mut iter = path.iter();
        self.insert(&mut iter, value)?;

        return Ok(())
    }


    pub fn get(&self, path: &str) -> Option<&A> {
        let mut iter = path.split("/");
        return self.0.get(&mut iter);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    // use super::PathNode::*;

    fn create_tree() -> RouteTree<i32> {
        let mut output = RouteTree::new();
        let path = vec![ Literal("one".to_owned()), Literal("two".to_owned()) ];
        output.insert_vec(path, 1);

        return output
    }

    #[test]
    fn test1() {
        let tree = create_tree();
        assert_eq!(tree.get("one/two"), Some(&1));
    }

    #[test]
    fn test2() {
        let tree = create_tree();
        assert_eq!(tree.get("one"), None);
    }
}
