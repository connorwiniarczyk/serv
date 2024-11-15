use crate::ast;
use crate::template::Template;
use std::collections::VecDeque;
use std::collections::HashMap;
use std::fmt::Display;

use crate::Scope;

use crate::ServFunction;
use crate::FnLabel;
use crate::ServResult;

#[derive(Clone)]
pub enum ServValue {
    None,
    Bool(bool),
    Int(i64),
    Float(f64),
    Text(String),
    Raw(Vec<u8>),

    List(VecDeque<ServValue>),
    Table(HashMap<String, ServValue>),

	ServFn(FnLabel),
    Expr(VecDeque<ServValue>),

    Meta { inner: Box<ServValue>, metadata: HashMap<String, ServValue> },
}

impl Default for ServValue {
	fn default() -> Self {
    	Self::None
	}
}

impl ServValue {
    pub fn eval(&self, input: Self, scope: &Scope) -> ServResult {
        // println!("eval called {} with input {}", self, input);
        match (self, input) {

            (Self::Expr(inner), i) => {
                let mut new_inner = inner.clone();
                let first = new_inner.pop_front().unwrap_or(Self::None);
                new_inner.push_back(i);
                first.eval(Self::Expr(new_inner), scope)
            },

            (Self::ServFn(label), Self::Expr(inner)) => {
                let func = scope.get(label).ok_or("err")?;

                if (!func.is_meta()) {
                    let result = Self::Expr(inner.clone()).eval(ServValue::None, scope)?;
                    func.call(result, scope)
                } else {
                    func.call(Self::List(inner.clone()), scope)
                }
            },

            (Self::ServFn(label), i) => scope.get(label).ok_or("not found")?.call(i.clone(), scope),

            otherwise => Ok(self.clone()),
        }
    }
    pub fn expect_int(&self) -> Result<i64, &'static str> {
        if let Self::Meta { inner, metadata } = self { return inner.expect_int() };
        let Self::Int(i) = self else { return Err("expected an int") };
        Ok(i.clone())
    }

    // pub fn insert_metadata(&mut self, key: &str, value: ServValue) {
    pub fn metadata(&mut self) -> &mut HashMap<String, ServValue> {
    	if let ServValue::Meta { ref inner, ref mut metadata } = self {
        	metadata
    	} else {
        	let mut metadata = HashMap::new();
        	let inner = Box::new(std::mem::take(self));
        	*self = Self::Meta { inner, metadata };

			let Self::Meta { inner, ref mut metadata } = self else { unreachable!() };
			metadata
    	}
    }

    pub fn get_metadata(&self) -> Option<&HashMap<String, ServValue>> {
        if let Self::Meta { inner, metadata } = self {
            Some(metadata)
        } else {
            None
        }
    }

    pub fn ignore_metadata(self) -> ServValue {
        if let Self::Meta { inner, .. } = self {
            *inner
        } else {
            self
        }
    }
}

impl From<i64> for ServValue {
    fn from(value: i64) -> Self {
        Self::Int(value)
    }
}

impl Display for ServValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::None => f.write_str("")?,
            Self::Bool(v) => v.fmt(f)?,
            Self::Float(v) => v.fmt(f)?,
            Self::Text(ref t) => f.write_str(t)?,
            Self::Raw(bytes) => {
                if let Ok(text) = std::str::from_utf8(bytes) {
                    f.write_str(text)?;
                } else {
                    f.debug_list().entries(bytes.iter()).finish()?
                }
            },
            Self::Int(i) => write!(f, "{}", i)?,
            Self::Table(table) => {
                f.write_str("{")?;

				let mut iter = table.iter().peekable();
				while let Some((k, v)) = iter.next() {
    				f.write_str("\"")?;
    				f.write_str(k)?;
    				f.write_str("\"")?;
    				f.write_str(": ")?;

    				match v {
        				ServValue::None => f.write_str("0")?,
        				ServValue::Text(t) => {
            				f.write_str("\"")?;
            				t.fmt(f)?;
            				f.write_str("\"")?;
        				},
        				a => a.fmt(f)?,
    				}

    				if iter.peek().is_some() {f.write_str(", ")?}
				}

                f.write_str("}")?;
            },
            Self::Expr(e) => {
                f.write_str("expr: (")?;
				let mut iter = e.iter().peekable();
				while let Some(element) = iter.next() {
    				element.fmt(f)?;
    				if let Some(_) = iter.peek() { f.write_str(" ")?; }
				}
				f.write_str(")")?;
            },
            Self::ServFn(func) => write!(f, "{}", func)?,
            Self::List(l) => {
                f.write_str("[")?;
				let mut iter = l.iter().peekable();
				while let Some(element) = iter.next() {
    				match element {
        				ServValue::None => f.write_str("0")?,
        				ServValue::Text(t) => {
            				f.write_str("\"")?;
            				t.fmt(f)?;
            				f.write_str("\"")?;
        				},
        				a => a.fmt(f)?,
    				}
    				if iter.peek().is_some() {f.write_str(", ")?}
				}
                f.write_str("]")?;
            }

            Self::Meta { inner, metadata } => inner.fmt(f)?,

            otherwise => f.write_str("")?,
        }

		Ok(())
    }
}
