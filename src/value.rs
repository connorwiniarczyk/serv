use crate::template::Template;
use std::collections::VecDeque;
use std::collections::HashMap;
use std::fmt::Display;

use crate::Stack;
use crate::Label;
use crate::ServResult;

use crate::module::Expression;

#[derive(Clone)]
pub enum ServFn {
    Ident,
    Core     (fn(ServValue, &Stack) -> ServResult),
    Meta     (fn(Expression, &mut Stack) -> ServResult),
    ArgFn    (fn(ServValue, ServValue, &Stack) -> ServResult),
    Expr     (Expression, bool),

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

    List(VecDeque<ServValue>),
    Table(HashMap<String, ServValue>),

    Meta { inner: Box<ServValue>, metadata: HashMap<String, ServValue> },
}

use std::rc::Rc;

#[derive(Clone)]
pub struct Transform (pub Rc<dyn Fn(&mut Stack)>);

impl std::fmt::Debug for Transform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.write_str("f")
    }
}

struct ValueMetadata {}

impl ServValue {
    pub fn call(&self, input: Option<ServValue>, scope: &Stack) -> ServResult {
       	match self {
           	Self::Ref(label) => scope.get(label.clone()).ok_or("not found")?.call(input, scope),

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
        if let Self::Meta { inner, metadata } = self { return inner.expect_int() };
        let Self::Int(i) = self else { return Err("expected an int") };
        Ok(i.clone())
    }

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
				let mut iter = l.iter().peekable();
				while let Some(element) = iter.next() {
    				element.fmt(f)?;
    				if iter.peek().is_some() {f.write_str(", ")?}
				}
                f.write_str("]")?;
            }

            Self::Meta { inner, metadata } => inner.fmt(f)?,
            _ => todo!(),
        }

		Ok(())
    }
}
