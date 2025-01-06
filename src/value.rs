use std::collections::VecDeque;
use std::collections::HashMap;
use std::fmt::Display;

use crate::template::Template;
use crate::Stack;
use crate::Label;
use crate::ServError;
use crate::ServResult;

pub use crate::servlist::ServList;

#[derive(Clone)]
pub enum ServFn {
    Ident,
    Core     (fn(ServValue, &Stack) -> ServResult),
    Meta     (fn(ServList, &mut Stack) -> ServResult),
    ArgFn    (fn(ServValue, ServValue, &Stack) -> ServResult),
    Expr     (ServList, bool),

    Route(String),
    Template (Template),
}

impl std::fmt::Debug for ServFn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::Ident       => f.write_str("Ident"),
            Self::Core(_)     => f.write_str("Core"),
            Self::Meta(_)     => f.write_str("Meta"),
            Self::ArgFn(_)    => f.write_str("ArgFn"),
            Self::Expr(_, _)  => f.write_str("Expr"),
            Self::Route(_)    => f.write_str("Route "),
            Self::Template(_) => f.write_str("Template "),
        }
    }
}

#[derive(Clone, Debug)]
pub enum ServValue {
    Ref(Label),
    Func(ServFn),

    None,
    Bool(bool),
    Int(i64),
    Float(f64),
    Text(String),
    Raw(Vec<u8>),

    List(ServList),
    Table(HashMap<String, ServValue>),

    Module(crate::ServModule),
}

impl ServValue {
    pub fn call(&self, input: Option<ServValue>, scope: &Stack) -> ServResult {
       	match self {
           	Self::Ref(label) => scope.get(label.clone())?.call(input, scope),

           	Self::Func(ServFn::Core(f)) => f(input.unwrap_or_default(), scope),
           	Self::Func(ServFn::Expr(e, _)) => {
               	let mut child = scope.make_child();
               	let mut expr = e.clone();
               	if let Some(v) = input { expr.push_back(v) };
               	expr.eval(&mut child)
           	},
           	Self::Func(ServFn::Ident) => Ok(input.unwrap_or_default()),
           	Self::Func(ServFn::Template(t)) => {
               	if let Some(v) = input {
                   	let mut child = scope.make_child();
                   	child.insert("in".into(), v.clone());
                   	child.insert(":".into(), v);
                   	t.render(&child)
               	}
               	else {
                   	t.render(scope)
               	}
           	},


           	constant => Ok(constant.clone()),
       	}
    }

    pub fn expect_int(&self) -> Result<i64, &'static str> {
        let Self::Int(i) = self else { return Err("expected an int") };
        Ok(i.clone())
    }

    pub fn is_truthy(&self) -> bool {
        match self {
        	ServValue::None        => false,
        	ServValue::Bool(false) => false,
        	ServValue::Int(0)      => false,
        	otherwise => true,
        }
    }
}

impl Default for ServValue {
	fn default() -> Self {
    	Self::None
	}
}

impl From<i64> for ServValue {
    fn from(value: i64) -> Self {
        Self::Int(value)
    }
}

impl From<ServList> for ServValue {
    fn from(value: ServList) -> Self {
        Self::List(value)
    }
}

pub trait Serializer {
	fn write<'buf>(&mut self, value: ServValue, dest: &'buf mut Buffer<'buf>) -> Result<(), ServError>;
}

type Buffer<'a> = dyn std::fmt::Write + 'a;

#[derive(Clone)]
pub struct JsonSerializer<'scope> {
    scope: &'scope Stack<'scope>,
}

impl<'a> JsonSerializer<'a> {
    pub fn new(scope: &'a Stack<'a>) -> Self {
        Self { scope }
    }
}

impl<'a> Serializer for JsonSerializer<'a> {
    fn write<'b>(&mut self, value: ServValue, dest: &'b mut Buffer<'b>) -> Result<(), ServError> {
        match value {
            _ => todo!(),
			ServValue::Ref(label) => self.write(self.scope.get(label)?, dest)?,
			f @ ServValue::Func(_) => self.write(f.call(None, self.scope)?, dest)?,

			ServValue::Raw(t)      => todo!("json serialize raw bytes"),
			ServValue::Module(t)   => todo!("json serialize modules"),
			ServValue::None     => dest.write_str("0")?,
			ServValue::Bool(b)  => dest.write_str(if b {"true"} else {"false"})?,
			ServValue::Float(v) => dest.write_str(&v.to_string())?,
			ServValue::Int(v)   => dest.write_str(&v.to_string())?,
			ServValue::Text(t)  => {
    			dest.write_str("\"");
    			dest.write_str(&t);
    			dest.write_str("\"")?
			},

			ServValue::List(list) => {
    			dest.write_str("[");
    			let mut iter = list.peekable();
    			while let Some(value) = iter.next() {
        			self.write(value, dest)?;
        			if iter.peek().is_some() {
            			dest.write_str(", ");
        			}
    			}
    			dest.write_str("]")?

			},

			ServValue::Table(table) => {
    			dest.write_str("{");
    			let mut iter = table.into_iter().peekable();
    			while let Some((key, value)) = iter.next() {
        			dest.write_str("\"");
        			dest.write_str(&key);
        			dest.write_str("\"");
        			dest.write_str(": ");
        			self.write(value, dest)?;
        			if iter.peek().is_some() {
            			dest.write_str(", ");
        			}
    			}
    			dest.write_str("}")?

			},
        };

        Ok(())
    }
}



impl Display for ServValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::Ref(Label::Name(n))       => f.write_str(n)?,
            Self::Ref(Label::Anonymous(id)) => f.write_str("()")?,
            Self::Func(v)                   => f.write_str("()")?,

            Self::None                      => f.write_str("NONE")?,
            Self::Bool(v)                   => v.fmt(f)?,
            Self::Float(v)                  => v.fmt(f)?,
            Self::Int(i)                    => i.fmt(f)?,
            Self::Text(ref t)               => f.write_str(t)?,
            Self::Raw(bytes) => {
                if let Ok(text) = std::str::from_utf8(bytes) {
                    f.write_str(text)?;
                } else {
                    f.debug_list().entries(bytes.iter()).finish()?
                }
            },
            Self::Table(table) => {
                f.write_str("{")?;

				let mut iter = table.iter().peekable();
				while let Some((k, v)) = iter.next() {
    				match v {
        				Self::Text(ref t) => write!(f, r#""{}": "{}""#, k, v),
        				otherwise     => write!(f, r#""{}": "{}""#, k, v),
    				}?;

    				if iter.peek().is_some() {f.write_str(", ")?}
				}

                f.write_str("}")?;
            },

            Self::List(l) => {
                f.write_str("[")?;
				let mut iter = l.clone().peekable();
				while let Some(element) = iter.next() {
    				element.fmt(f)?;
    				if iter.peek().is_some() {f.write_str(", ")?}
				}
                f.write_str("]")?;
            },

            Self::Module(m) => f.write_str("a module")?,
            // _ => todo!(),
        }

		Ok(())
    }
}
