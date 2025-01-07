use std::collections::VecDeque;
use crate::{ServFn, ServValue, ServResult, ServError};

#[derive(Debug, Clone, Default)]
pub struct ServList(VecDeque<ServValue>);

impl ServList {
    pub fn new() -> Self {
		Self(VecDeque::new())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn pop(&mut self) -> ServResult {
        self.0.pop_front().ok_or(crate::ServError::new(500, "empty list"))
    }

    pub fn push(&mut self, input: ServValue) {
        self.0.push_front(input);
    }

    pub fn push_back(&mut self, input: ServValue) {
        self.0.push_back(input);
    }

    pub fn get(&self, index: usize) -> Result<&ServValue, crate::ServError>{
        self.0.get(index).ok_or(crate::ServError::new(500, "not found"))
    }

    pub fn concat(self, input: Self) -> Self {
        let mut output = VecDeque::new();
        self.0.into_iter().for_each(|x| output.push_back(x));
        input.0.into_iter().for_each(|x| output.push_back(x));

        Self(output)
    }

    pub fn as_expr(self) -> ServValue {
        ServValue::Func(ServFn::Expr(self, false))
    }

    pub fn eval(&mut self, scope: &mut crate::Stack) -> ServResult {
        let Ok(mut next) = self.pop() else { return Ok(ServValue::None) };

        if let ServValue::Ref(ref label) = next {
            if let Ok(value) = scope.get(label.clone()) {
                next = value;
            }
        }

        match next {
			ServValue::Func(ServFn::Expr(e, _)) => {
    			*self = e.concat(std::mem::take(self));
    			self.eval(scope)
			},

            ServValue::Func(ServFn::Meta(f)) => {
                f(std::mem::take(self), scope)
            },

            ServValue::Func(ServFn::ArgFn(f)) => {
                let arg = self.pop()?;
                let rest = self.eval(scope)?;
                f(arg, rest, scope)
            },

            f => f.call(Some(self.eval(scope)?), scope),
        }
    }
}

impl Iterator for ServList {
    type Item = ServValue;
	fn next(&mut self) -> Option<Self::Item> {
		self.pop().ok()
	}
}

impl FromIterator<ServValue> for ServList {
    fn from_iter<T>(iter: T) -> Self where T: IntoIterator<Item = ServValue> {
        let mut output = Self::new();
        for value in iter {
            output.push_back(value);
        }
        output
    }
}

impl From<Vec<ServValue>> for ServList {
	fn from(input: Vec<ServValue>) -> Self {
    	ServList(input.into())
	}
}
