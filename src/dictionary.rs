use std::collections::HashMap;

pub trait Key: Eq + PartialEq + std::hash::Hash {}
impl<T: Eq + PartialEq + std::hash::Hash> Key for T {}

pub struct StackDictionary<'parent, K, V> {
	words: HashMap<K, V>,
	parent: Option<&'parent StackDictionary<'parent, K, V>>,
}

impl<'parent, K: Key, V: Clone> StackDictionary<'parent, K, V> {
    pub fn empty() -> Self {
        Self { words: HashMap::new(), parent: None }
    }

    pub fn make_child(&'parent self) -> Self {
        Self { words: HashMap::new(), parent: Some(self) }
    }

    pub fn insert(&mut self, key: K, value: V) {
        self.words.insert(key, value);
    }

    pub fn get(&self, key: &K) -> Option<V> {
        self.words.get(key).map(|s| s.clone()).or_else(|| {
            self.parent.and_then(|p| p.get(key))
        })
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
