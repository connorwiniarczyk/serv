use json;
use json::JsonValue;

use std::collections::{HashSet, HashMap};

struct StrIndex(usize, usize);

#[derive(Clone, Copy, Hash)]
struct Entry {
    key_index: (usize, usize),
    value_index: (usize, usize),
}

enum Value {
    String(Entry),
    Collection(HashSet<Entry>),
}

pub struct Vars {
    value_buffer: String,
    value_cursor: usize,
    key_buffer: String,
    key_cursor: usize,

    variables: HashMap<String, Value>,
}

impl Vars {
    pub fn new() -> Self {
        todo!()
        // Self { buffer: String::with_capacity(128), cursor: 0, variables: HashMap::new() }
    }

    pub fn insert(&mut self, key: &str, value: &str) {
        let entry = Entry {
            key_index: (self.key_cursor, self.key_cursor + key.len()),
            value_index: (self.value_cursor, self.value_cursor + value.len()),
        };

        self.key_buffer.push_str(key);
        self.key_cursor += value.len();

        self.value_buffer.push_str(value);
        self.value_cursor += value.len();

        self.variables.insert(key.to_owned(), Value::String(entry));

        let mut key_iter = key;
        let mut acc = String::new();

        while let Some((next, rest)) = key_iter.split_once(".") {
            acc.push_str(next); 
            match self.variables.get(acc.as_str()) {
                Some(Value::String(_)) => todo!(),
                Some(Value::Collection(s)) => s.insert(acc),
                None => todo!(),
            }

        }

        todo!();
    }
}


// pub struct Object(JsonValue);

// impl Object {

//     pub fn new() -> Self {
//         todo!();
//     }

//     fn insert_inner<I>(&mut self, mut key: std::iter::Peekable<I>, value: &str) -> Result<(), &'static str>
//     where I: Iterator<Item = &'static str>, {

//         let next = key.next();
//         let is_last = key.peek().is_none();

//         if is_last {
//             current_obj.insert(key.next(), value);
//             return Ok(());
//         }

//         todo!();
//     }

//     pub fn insert(&mut self, key: &str, value: &str) -> Result<(), &'static str> {

//         let mut current_obj = &mut self.0;
//         let mut key_iter = key.split(".").peekable();

//         // while let Some(component) = key_iter.next() {

//         //     let is_last = key_iter.peek().is_none();
//         //     if is_last {
//         //         current_obj.insert(component, value);
//         //     }

//         //     else {
//         //         let mut inner: &mut json::object::Object = if let JsonValue::Object(ref mut obj) = current_obj { obj } else { unreachable!() };
//         //         let mut next_value = inner.get_mut(component).ok_or("value does not exist")?;

//         //         match next_value {
//         //             JsonValue::Object(_) => current_obj = next_value,
//         //             JsonValue::Null => {
//         //                 let next = JsonValue::new_object();
//         //                 inner.insert(component, next);
//         //             }
//         //             _ => todo!(),
//         //         }

//         //         // match &next_value {
//         //         //     JsonValue::Object(_) => current_obj = obj,
//         //         //     _ => todo!(),
//         //         // }
//         //     }

//             // let mut next_value = current_obj.get_mut(key_component).ok_or("value does not exist")?;

//             // if next_value.is_null() {
//             //     next_value.insert(key_component, JsonValue::new_object());

//             // }

//             // match next_value {
//             //     JsonValue::Object(ref mut obj) => current_obj = obj,
//             //     Null => {

//             //     }
//             //     _ => todo!(),
//             // }
//         // }

//         todo!();
//     }

//     pub fn get(&mut self, key: &str) -> Result<String, &'static str> {
//         todo!();
//         // let mut current_obj = &self.0;
//         // let mut key_iter = key.split(".").peekable();
//         // while let Some(key_component) = key_iter.next() {
//         //     let next_value = current_obj.get(key_component).ok_or("Value does not exist")?;
//         //     if let Some(_) = key_iter.peek() {
//         //         match next_value {
//         //             JsonValue::Object(obj) => current_obj = obj,
//         //             JsonValue::Array(array) => todo!(),
//         //             _ => return Err("invalid key"),
//         //         };
//         //     } else {
//         //         match next_value {
//         //             JsonValue::String(inner) => return Ok(inner.to_owned()),
//         //             JsonValue::Object(inner) => return Ok(inner.pretty(2)),
//         //             _ => todo!(),
//         //         };
//         //     }
//         // }

//         // Err("failed")
//     }
// }

// struct KeyIter<'a>{
//     inner: &'a str,
//     cursor: usize,
// }

// impl<'a> Iterator for KeyIter<'a> {
//     type Item = Result<&'a str, &'static str>;

//     fn next(&mut self) -> Option<Self::Item> {
//         let mut current = 

//         todo!();
//     }
// }

// impl<'a> KeyIter<'a> {
//     fn new(input: &'a str) -> Self {
//         Self { inner: input, cursor: 0 }
//     }
// }
