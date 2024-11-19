use std::collections::HashMap;
use matchit::Router;

use hyper::body::{Body, Frame, Incoming as IncomingBody};
use hyper::{ Request, Response };
use hyper::http::request::Parts;


#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum Label {
    Name(String),
    Anonymous(u32),
}

impl Label {
    pub fn name(input: &str) -> Self       { Self::Name(input.to_string()) }
    pub fn anonymous(input: u32) -> Self { Self::Anonymous(input) }
}

impl From<String> for Label {
    fn from(input: String) -> Self {
        Self::Name(input)
    }
}

impl From<&String> for Label {
    fn from(input: &String) -> Self {
        Self::Name(input.clone())
    }
}

impl From<&str> for Label {
    fn from(input: &str) -> Self {
        Self::Name(input.to_string())
    }
}

#[derive(Clone)]
pub struct StackDictionary<'parent, V, M> {
    unique_id: u32,
	pub parent: Option<&'parent Self>,
	pub words: HashMap<Label, V>,

	pub metadata: Option<M>,

	pub request: Option<Parts>,
	pub router: Option<Router<V>>,
}

impl<'parent, V: Clone, M> StackDictionary<'parent, V, M> {
    pub fn empty() -> Self {
        Self {
            unique_id: 0,
            words: HashMap::new(),
            parent: None,
            metadata: None,

			// deprecated
            request: None,
            router: Some(Router::new()),
        }
    }

    pub fn make_child(&'parent self) -> Self {
        Self {
            unique_id: self.unique_id,
            words: HashMap::new(),
            parent: Some(self),
            metadata: None,

			// deprecated
            request: None,
            router: Some(Router::new()),
        }
    }

    pub fn with_input(mut self, input: V) -> Self {
        self.insert_name("in", input);
        self
    }

    pub fn insert(&mut self, key: Label, value: V) {
        self.words.insert(key, value);
    }

    pub fn insert_name(&mut self, key: &str, value: V) {
        self.words.insert(Label::name(key), value);
    }

    pub fn insert_anonymous(&mut self, value: V) -> Label {
        let id = self.get_unique_id();
        self.words.insert(Label::Anonymous(id), value);
        Label::Anonymous(id)
    }

    pub fn get<L: Into<Label>>(&self, l: L) -> Option<V> {
        let key: Label = l.into();
        self.words.get(&key).map(|s| s.clone()).or_else(|| {
            self.parent.and_then(|p| p.get_label(&key))
        })
    }

    fn get_label(&self, key: &Label) -> Option<V> {
        self.words.get(key).map(|s| s.clone()).or_else(|| {
            self.parent.and_then(|p| p.get_label(key))
        })
    }

    pub fn get_unique_id(&mut self) -> u32 {
		let output = self.unique_id;
		self.unique_id += 1;
		return output
    }

    pub fn get_request(&self) -> Option<&Parts> {
        None
        // self.request.as_ref().or_else(|| self.parent.and_then(|p| p.get_request()))
    }
}

use std::fmt::Display;

impl Display for Label {
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
