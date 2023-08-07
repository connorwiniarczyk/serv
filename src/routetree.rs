
use std::collections::HashMap;

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
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
	doublewildcard: Option<(String, PathNode)>,

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
				// let child = self.children.entry(n.clone()).or_insert(Self::new());
				let child = self.children.entry(n.clone()).or_insert(Self::new());
				match n {
					Literal(s) => { self.literals.insert(s.to_owned(), n.clone()); },
					Wildcard(s) => { self.wildcards.push((s.to_owned(), n.clone())); },
					Doublewildcard(s) => { self.doublewildcard = Some((s.to_owned(), n.clone())); },
				};
				child.insert(iter, value)?;
			},
		}

		return Ok(())
	}

	fn get<'a, 'b, I>(&'a self, iter: &mut I, vars: &mut HashMap<String, String>) -> Option<&'a A>
	where I: Iterator<Item = &'b str> + Clone {
		let next = iter.next();

		match (next, &self.value) {
			(None, None) => { return None },
			(None, Some(v)) => { return Some(v) },
			(Some(n), _) => {
				if let Some(node) = self.literals.get(n) {
					let child = self.children.get(node).unwrap();
					return child.get(iter, vars);
				}

				for (name, node) in self.wildcards.iter() {
					let mut maybe_vars: HashMap<String, String> = HashMap::new();
					maybe_vars.insert(name.clone(), n.to_owned());
					if let Some(v) = self.children.get(&node).unwrap().get(iter, &mut maybe_vars) {
						vars.extend(maybe_vars);
						return Some(v)
					}
				}

				if let Some((name, node)) = &self.doublewildcard {
					let child = self.children.get(&node).unwrap();
					while iter.next().is_some() {};
					return child.get(iter, vars);
				}

				return None;
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
		let mut vars: HashMap<String, String> = HashMap::new();
		let mut iter = path.split("/");

		iter.next();
		return self.0.get(&mut iter, &mut vars);
	}

	pub fn get_special(&self, name: &str) -> Option<A> {
		todo!();
	}

	pub fn insert_special(name: &str, value: A) {
		todo!();
	}
}

use std::fmt::Debug;

impl<A> Debug for RouteTree<A>
where A: Debug {
	fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
		// let inner = self.0;
		self.0.literals.fmt(fmt)?;
		self.0.value.fmt(fmt)?;

		Ok(())
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::parser::parse_str_id_only as parse;

	fn create_tree() -> RouteTree<i32> {
		let mut output = RouteTree::new();
		let path = vec![ Literal("one".to_owned()), Literal("two".to_owned()) ];
		output.insert_vec(path, 1);

		return output
	}

	#[test]
	fn test_root() {
		let tree = parse("/: {1}").expect("failed to parse");
		assert_eq!(tree.get("/"), Some(&0));

	}

	#[test]
	fn test1() {
		let tree = create_tree();
		assert_eq!(tree.get("/one/two"), Some(&1));
	}

	#[test]
	fn test2() {
		let tree = create_tree();
		assert_eq!(tree.get("/one"), None);
	}

	#[test]
	fn test3() {
		let tree = parse("/one/*two/three: {1}").expect("failed to parse");
		assert_eq!(tree.get("/one/a/three"), Some(&0));
	}
}
