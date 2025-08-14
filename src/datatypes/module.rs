use crate::ServValue;
use crate::Stack;
use crate::Label;
use crate::ServFn;
use crate::ServResult;
use crate::value::ServList;
use crate::ServError;

use std::iter::Peekable;
use std::collections::hash_map::Entry;

use crate::dictionary::Address;

use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
pub struct ServModule {
    pub values: HashMap<Label, ServValue>,
    pub statements: Vec<ServList>
}

impl ServModule {
    pub fn empty() -> Self {
        Self {
            values: HashMap::new(),
            statements: Vec::new(),
        }
    }

  //   pub fn deep_insert<'a, I: Iterator<Item = &'a Label>>(&mut self, key: &'a mut Peekable<I>, value: ServValue) {
		// todo!();
  //   }

   //  pub fn get(&self, i: &mut impl Iterator<Item=Label>) -> Option<ServValue> {
   //      let mut active = self;
   //      while let Some(n) = i.next() {
   //          let next = active.values.get(&n)?;
			// match next {
   //  			ServValue::Module(m)
			// }
   //      }
   //      todo!();
   //  }


 //    fn get_internal<'a>(&self, a: &mut impl Iterator<Item=&'a Label>) -> Option<&ServValue> {
 //        todo!();
 //    }

	// pub fn get<K: Into<Address>>(&self, key: K) -> Option<&ServValue> {
 //    	let mut iter = key.into().iter();
 //    	let value = self.values.get(iter.next()?)?;
 //    	return value.get_member(&mut iter)
	// }

	// fn get_entry_if_module(&mut self, key: &Label) -> Entry<Label, ServValue> {
	// 	todo!();
	// }

    pub fn insert_internal<'a, I: Iterator<Item=&'a Label>>(&mut self, iter: &mut Peekable<I>, value: ServValue) -> Result<(), ServError> {
        let next = iter.next().unwrap();
        if iter.peek().is_none() {
			self.values.insert(next.clone(), value);
			return Ok(())
        }

        let dest = self.values.entry(next.clone()).or_insert(Self::empty().into());
        let ServValue::Module(m) = dest else {
            return Err(ServError::new(500, "tried to insert into something that was not a module"));
        };

        m.insert_internal(iter, value)?;
		Ok(())
    }

    pub fn insert<K: Into<Address>>(&mut self, k: K, value: ServValue) -> Result<(), ServError> {
        self.insert_internal(&mut k.into().iter().peekable(), value)
    }

    pub fn insert_declaration(&mut self, key: Option<Address>, value: ServList) {
        if key.is_none() { return self.statements.push(value) };

        self.insert(key.unwrap(), value.as_expr());
    }

    pub fn routes(&self) -> impl Iterator<Item=(&str, &ServValue)> {
        self.values.iter().filter_map(|(l, v)| {
            if let Label::Route(ref name) = l {
                Some((name.as_str(), v))
            }

            else {
                None
            }
        })
    }


    pub fn call(self, input: Option<ServValue>, scope: &mut Stack) -> ServResult {
		if self.statements.len() == 0 { return Ok(ServValue::Module(self)) }

		scope.insert_module(self.values);
        let mut output = ServValue::None;
        for mut expr in self.statements {
            output = expr.as_expr().call(input.clone(), scope)?;
        }

		Ok(output)
    }
}
