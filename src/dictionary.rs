use std::collections::HashMap;
use std::fmt::Display;
use matchit::Router;
use crate::ServError;

use std::collections::VecDeque;

use hyper::body::{Body, Frame, Incoming as IncomingBody};
use hyper::{ Request, Response };
use hyper::http::request::Parts;

pub use crate::datatypes::reference::{Label, Address};


#[derive(Clone)]
pub struct StackDictionary<'parent, V> {
    unique_id: u32,
	parent: Option<&'parent Self>,
	words: HashMap<Label, V>,

	pub request: Option<Parts>,
}

impl<'parent, V: Clone> StackDictionary<'parent, V> {
    pub fn empty() -> Self {
        Self {
            unique_id: 0,
            words: HashMap::new(),
            parent: None,
            request: None,
        }
    }

    pub fn make_child(&'parent self) -> Self {
        Self {
            unique_id: self.unique_id,
            words: HashMap::new(),
            parent: Some(self),
            request: None,
        }
    }

    pub fn insert<L: Into<Label>>(&mut self, key: L, value: V) {
        self.words.insert(key.into(), value);
    }

    pub fn insert_module(&mut self, value: HashMap<Label, V>) {
        self.words.extend(value);
    }

    // fn insert_anonymous(&mut self, value: V) -> Label {
    //     let id = self.get_unique_id();
    //     self.words.insert(Label::Anonymous(id), value);
    //     Label::Anonymous(id)
    // }

    pub fn get<L: Into<Label>>(&self, l: L) -> Result<V, ServError> {
        let key: Label = l.into();
        let value = self.words.get(&key);

        if let Some(v) = value {
            return Ok(v.clone());
        };

		let Some(parent) = self.parent else {
    		return Err(ServError::MissingLabel(key));
		};

		return parent.get(key)
    }

  //   fn get_unique_id(&mut self) -> u32 {
		// let output = self.unique_id;
		// self.unique_id += 1;
		// return output
  //   }

    pub fn get_request(&self) -> Option<&Parts> {
        self.request.as_ref().or_else(|| self.parent.and_then(|p| p.get_request()))
    }
}


impl Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match (self) {
            Self::Name(s) => f.write_str(s)?,
            Self::Route(s) => f.write_str(s)?,
            // Self::Anonymous(id) => write!(f, "anonymous function {}", id)?,
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
	}
}
