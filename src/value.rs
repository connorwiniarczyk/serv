use crate::ast;
use crate::template::Template;
use std::collections::VecDeque;
use std::collections::HashMap;
use std::fmt::Display;

use crate::Stack;

use crate::Label;
use crate::ServResult;

#[derive(Clone)]
pub enum ServFn {
    Core     (fn(ServValue, &Stack) -> ServResult),
    CoreMeta (fn(ServValue, &Stack) -> ServResult),
    Expr     (VecDeque<ServValue>),
    ExprMeta (VecDeque<ServValue>),

    Template(Template),
}

impl ServFn {
	pub fn call(&self, input: ServValue, stack: &Stack) -> ServResult {
        match self {
            Self::Core(f)        => f(input, stack),
            Self::CoreMeta(f)    => f(input, stack),
            Self::Expr(i)        => {
                let mut inner = i.clone();
                let Some(front) = inner.pop_front() else { return Ok(input) };
                if front.is_meta(stack) {
                    inner.push_back(input);
                    front.eval(Some(ServValue::List(inner)), stack)
                }

                else {
					front.eval(Some(Self::Expr(inner).call(input, stack)?), stack)
                }

            },
            // Self::ExprMeta(i)    => {
            //     let mut inner = i.clone();
            //     let Some(front) = inner.pop_front() else { return Ok(ServValue::None) };
            //     let remainder = Self::Expr(inner).call(input, stack)?;
            //     front.eval(Some(remainder), stack)
            // },
            Self::Template(t)    => {
                let mut child = stack.make_child();
                child.insert_name("in", input);
                t.render(&child)
            },
            otherwise => todo!(),
        }
	}
}

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

    Ref(Label),
    FnLiteral(ServFn),

    Meta { inner: Box<ServValue>, metadata: HashMap<String, ServValue> },
}

impl Default for ServValue {
	fn default() -> Self {
    	Self::None
	}
}

impl ServValue {
    fn is_meta(&self, stack: &Stack) -> bool {
        if let Self::Ref(label) = self { return stack.get(label.clone()).unwrap().is_meta(stack) };
        if let Self::Meta {inner, ..} =  self { return inner.is_meta(stack) };

        let Self::FnLiteral(f) = self else { return false };
        match f {
            ServFn::Core(_)     => false,
            ServFn::CoreMeta(_) => true,
            ServFn::Expr(_)     => false,
            ServFn::ExprMeta(_) => true,
            ServFn::Template(_) => false,
        }
    }

    pub fn eval(&self, input: Option<Self>, scope: &crate::Stack) -> ServResult {
        match (self, input) {

            // (Self::Expr(inner), i) => {
            //     let mut new_inner = inner.clone();
            //     let first = new_inner.pop_front().unwrap_or(Self::None);
                
            //     if i.is_some() { new_inner.push_back(i.unwrap()) };

            //     first.eval(Some(Self::Expr(new_inner)), scope)
            // },

            // (Self::FnLiteral(f), Some(Self::Expr(inner))) => {
            //     if (!f.is_meta()) {
            //         let result = Self::Expr(inner.clone()).eval(None, scope)?;
            //         f.call(result, scope)
            //     } else {
            //         f.call(Self::List(inner.clone()), scope)
            //     }
            // },

            (Self::FnLiteral(f), Some(i)) => f.call(i, scope),
            (Self::FnLiteral(f), None)    => f.call(ServValue::None, scope),


            (Self::Ref(label), i) => {
                let deref = scope.get(label.clone()).ok_or("err")?;
                deref.eval(i, scope)
            },

            // (Self::ServFn(label), Some(i)) => scope.get(label).ok_or("not found")?.call(i.clone(), scope),

            otherwise => Ok(self.clone()),
            // otherwise => Err("not supported"),
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
            Self::None => f.write_str("NONE")?,
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
    //         Self::Expr(e) => {
    //             f.write_str("expr: (")?;
				// let mut iter = e.iter().peekable();
				// while let Some(element) = iter.next() {
    // 				element.fmt(f)?;
    // 				if let Some(_) = iter.peek() { f.write_str(" ")?; }
				// }
				// f.write_str(")")?;
    //         },
            // Self::ServFn(func) => write!(f, "{}", func)?,
            Self::List(l) => {
                f.write_str("[")?;
				let mut iter = l.iter().peekable();
				while let Some(element) = iter.next() {
    				match element {
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
