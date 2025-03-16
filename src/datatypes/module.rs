use crate::ServValue;
use crate::Stack;
use crate::Label;
use crate::ServFn;
use crate::ServResult;
use crate::value::ServList;

use std::collections::HashMap;

use matchit::Router;

#[derive(Clone, Debug)]
pub struct Element {
    pub pattern: Option<ServValue>,
    pub action: ServList,
}

struct ElementId(usize);


#[derive(Clone, Debug, Default)]
pub struct ServModule {
    pub values: HashMap<Label, ServValue>,
    pub statements: Vec<ServList>
}

impl ServModule {
    pub fn empty() -> Self {
        Self { values: HashMap::new(), statements: Vec::new() }
    }

    pub fn insert<L: Into<Label>>(&mut self, label: L, value: ServValue) {
        self.values.insert(label.into(), value);
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

    pub fn insert_declaration(&mut self, label: Option<Label>, value: ServList) {
        if label.is_some() {
            self.values.insert(label.unwrap(), value.as_expr());
        }

        else {
            self.statements.push(value);
        }
    }

    pub fn call(self, input: Option<ServValue>, scope: &mut Stack) -> ServResult {
		if self.statements.len() == 0 { return Ok(ServValue::Module(self)) }

		scope.insert_module(self.values);
        let mut output = ServValue::None;
        for mut expr in self.statements {
            output = expr.as_expr().call(input.clone(), scope)?;
            // if let Some(ref i) = input {
            //     expr.push_back(i.clone());
            // }
            // output = expr.eval(scope)?;
        }

		Ok(output)
    }
}
