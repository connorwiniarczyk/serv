use json;
use json::JsonValue;

use std::collections::{HashSet, HashMap};

#[derive(Hash, PartialEq, Eq, Clone)]
struct Key(Vec<String>);

impl Key {
    fn from_str(input: &str) -> Self {
        let mut inner = Vec::new();
        for key in input.split(".") {
            inner.push(key.to_owned());
        }
        Self(inner)
    }

    fn with_depth(&self, depth: usize) -> &[String] {
        &self.0[0..depth]
    }

    fn depth(&self) -> usize {
        self.0.len()
    }

    fn new() -> Self {
        Self(Vec::new())
    }

    fn push(&mut self, input: &str) {
        self.0.push(input.to_owned());
    }

    fn traverse(&self) -> KeyTraverseIter {
        todo!();
    }
}

struct KeyTraverseIter<'key> {
    inner: &'key Key,
    cursor: usize,
}

impl<'key> Iterator for KeyTraverseIter<'key> {
    type Item = Key;

    fn next(&mut self) -> Option<Self::Item> {
        todo!();
    }
}

enum Value {
    String(String),
    Collection(HashSet<Key>),
}

impl Value {
    fn from_str(input: &str) -> Self {
        Self::String(input.to_owned())
    }

    fn collection_with_element(input: &str) -> Self {
        let mut inner = HashSet::new();
        inner.insert(Key::from_str(input));
        Self::Collection(inner)
    }

    fn empty_collection() -> Self {
        Self::Collection(HashSet::new())
    }

    fn to_str(&self) -> Result<&str, ()> {
        match self {
            Self::String(c) => Ok((c.as_str())),
            _ => Err(())
        }
    }
}

pub struct Vars {
    data: HashMap<Key, Value>,
}

impl Vars {
    pub fn new() -> Self {
        Self { data: HashMap::new() }
    }

    fn insert_value(&mut self, key: Key, value: &str) {
        self.data.insert(key, Value::from_str(value));
    }

    pub fn insert(&mut self, key_str: &str, value: &str) {
        let key = Key::from_str(key_str);

        let mut i: usize = 0;
        while (i < key.depth()) {
            if i == key.depth() - 1 {
                self.insert_value(key.clone(), value);
            }

            i += 1;
        }


        // let mut acc = Key::new();
        // let mut prev = acc.clone();

        // let mut key_steps = key.0.iter().peekable();
        // while let Some(step) = key_steps.next() {
        //     acc.push(step);
        //     if key_steps.peek().is_some() {
        //         todo!();
        //     } else {
        //         todo!();
        //     }
        // }

        // todo!();

        // let key = Key::from_str(key_str);
        // self.data.insert(key.clone(), Value::from_str(value));

        // let mut key_iter = key.split(".").peekable();
        // let mut acc = String::new();
        // acc.push_str(key_iter.next().unwrap());

        // while let Some(next) = key_iter.next() {
        //     match self.data.get_mut(acc.as_str()) {
        //         Some(Value::String(_)) => todo!(),
        //         Some(Value::Collection(s)) => {
        //             s.insert(key.to_owned());
        //         },
        //         None => {
        //             let new_collection = Value::collection_with_element(key);
        //             self.data.insert(acc.clone(), new_collection);
        //         },
        //     };

        //     acc.push_str("."); 
        //     acc.push_str(next); 
        // }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        todo!();
        // let mut output = String::new();

        // match self.data.get(key)?{
        //     Value::String(v) => return Some(v.clone()),
        //     Value::Collection(c) => {
        //         let mut output = String::new();
        //         let mut stack: Vec<String> = Vec::new();
        //         let mut level = 1;

        //         output.push_str("{\n");
        //         for k in c.iter() {
        //             output.push_str("\t");
        //             output.push_str(k);
        //             output.push_str(": ");
        //             output.push_str(self.data.get(k).unwrap().to_str().unwrap());
        //             output.push_str(",\n");
        //         }
        //         output.push_str("}\n");

        //         return Some(output);
        //     }
        // }
    }
}

use std::fmt::{Debug, Formatter};

impl Debug for Vars {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!();
        // for (key, value) in self.data.iter() {
        //     write!(f, "{}: ", key)?;
        //     match value {
        //         Value::String(v) => write!(f, "{}, \n", v)?, // f.write_str(v)?,
        //         Value::Collection(c) => {
        //             f.write_str("\n")?;
        //             for k in c.iter() {
        //                 f.write_str("\t");
        //                 f.write_str(k);
        //                 f.write_str("\n");
        //             }

        //         },
        //     }
        // }

        // Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn insert() {
        // let mut vars = Vars::new();
        // vars.insert("object.first", "first");
        // vars.insert("object.second", "second");
        // vars.insert("object.third", "third");

        // vars.insert("object.inner.first", "fourth");

        // // println!("{:?}", vars);

        // // assert_eq!(vars.get("hello"), Some("abcd".to_owned()));

        // println!("{}", vars.get("object").unwrap());
        // panic!();

    }
}
