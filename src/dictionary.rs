use std::collections::HashMap;
use matchit::Router;

use hyper::body::{Body, Frame, Incoming as IncomingBody};
use hyper::{ Request, Response };
use hyper::http::request::Parts;


#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum FnLabel {
    Name(String),
    Anonymous(u32),
}

impl FnLabel {
    pub fn name(input: &str) -> Self       { Self::Name(input.to_string()) }
    pub fn anonymous(input: u32) -> Self { Self::Anonymous(input) }
}

impl From<String> for FnLabel {
    fn from(input: String) -> Self {
        Self::Name(input)
    }
}

pub struct StackDictionary<'parent, V> {
    unique_id: u32,

	pub request: Option<Parts>,
	pub words: HashMap<FnLabel, V>,
	pub parent: Option<&'parent StackDictionary<'parent, V>>,
	pub router: Option<Router<V>>,
}

impl<'parent, V: Clone> StackDictionary<'parent, V> {
    pub fn empty() -> Self {
        Self {
            unique_id: 0,
            words: HashMap::new(),
            parent: None,
            request: None,
            router: Some(Router::new()),
        }
    }

    pub fn make_child(&'parent self) -> Self {
        Self {
            unique_id: self.unique_id,
            words: HashMap::new(),
            parent: Some(self),
            router: None,
            request: None,
        }
    }

    pub fn with_input(mut self, input: V) -> Self {
        self.insert_name("in", input);
        self
    }

    pub fn insert(&mut self, key: FnLabel, value: V) {
        self.words.insert(key, value);
    }

    pub fn insert_name(&mut self, key: &str, value: V) {
        self.words.insert(FnLabel::name(key), value);
    }

    pub fn insert_anonymous(&mut self, value: V) -> FnLabel {
        let id = self.get_unique_id();
        self.words.insert(FnLabel::Anonymous(id), value);
        FnLabel::Anonymous(id)
    }

    pub fn get(&self, key: &FnLabel) -> Option<V> {
        self.words.get(key).map(|s| s.clone()).or_else(|| {
            self.parent.and_then(|p| p.get(key))
        })
    }

    pub fn get_unique_id(&mut self) -> u32 {
		let output = self.unique_id;
		self.unique_id += 1;
		return output
    }

    pub fn get_request(&self) -> Option<&Parts> {
        self.request.as_ref().or_else(|| self.parent.and_then(|p| p.get_request()))
    }
}

use std::fmt::Display;

impl Display for FnLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match (self) {
            Self::Name(s) => f.write_str(s)?,
            Self::Anonymous(id) => write!(f, "anonymous function {}", id)?,
        };

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

	#[test]
	fn test() {
    	let mut one: StackDictionary<String, Vec<String>> = StackDictionary::empty();
    	one.insert("hello".to_owned(), vec![])
		// let mut test = StackDictionary { words: HashMap::new(), parent: None };
		// test.insert("1", "2");

		// abcd(&test);

  //       println!("{:?}", test.get("hi"));

		// let test2 = test.make_child();
		// let test3 = test2.make_child();

		// panic!("{:?}", test3.get("1"));
	}
}
