use crate::ast;
use crate::template::Template;
use std::collections::VecDeque;
use std::collections::HashMap;
use std::fmt::Display;

use crate::Stack;

use crate::Label;
use crate::ServResult;

pub fn eval(mut expr: VecDeque<ServValue>, scope: &Stack) -> ServResult {
    match expr.pop_front() {
        Some(ServValue::Ref(label)) => {
			expr.push_front(scope.get(label.clone()).ok_or("")?);
			eval(expr, scope)
        },

        Some(ServValue::Func(ServFn::Meta(f))) => {
            f(expr, scope)
        },

        Some(ServValue::Func(ServFn::Expr(e, _))) => {
            for word in e.into_iter().rev() {
                expr.push_front(word);
            }
            eval(expr, scope)
        },
       	Some(ServValue::Func(ServFn::ArgFn(f))) => {
           	let arg  = expr.pop_front().ok_or("word expected")?;
           	let rest = eval(expr, scope)?;
           	f(arg, rest, scope)
       	},

        Some(ref f @ ServValue::Func(ref a)) => {
            let rest = eval(expr, scope)?;
            f.call(Some(rest), scope)
        },

        Some(constant) => Ok(constant),
        None => Ok(ServValue::Func(ServFn::Ident)),
    }
}

#[derive(Clone)]
pub enum ServFn {
    Ident,
    Core     (fn(ServValue, &Stack) -> ServResult),
    Meta     (fn(VecDeque<ServValue>, &Stack) -> ServResult),
    ArgFn    (fn(ServValue, ServValue, &Stack) -> ServResult),
    Expr     (VecDeque<ServValue>, bool),
    Template (Template),
}

impl std::fmt::Debug for ServFn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::Ident    => f.write_str("Ident"),
            Self::Core(_)     => f.write_str("Core"),
            Self::Meta(_)     => f.write_str("Meta"),
            Self::ArgFn(_)   => f.write_str("ArgFn"),
            Self::Expr(_, _)     => f.write_str("Expr"),
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


impl ServValue {
    pub fn call(&self, input: Option<ServValue>, scope: &Stack) -> ServResult {
       	match self {
           	Self::Ref(label) => scope.get(label.clone()).ok_or("not found")?.call(input, scope),

           	Self::Func(ServFn::Core(f)) => f(input.unwrap_or_default(), scope),
           	Self::Func(ServFn::Expr(e, _)) => {
               	let mut child = e.clone();
               	if let Some(v) = input { child.push_back(v); }
               	eval(child, scope)
           	},
           	Self::Func(ServFn::Ident) => Ok(input.unwrap_or_default()),
           	Self::Func(ServFn::Template(t)) => {
               	if let Some(v) = input {
                   	let mut child = scope.make_child();
                   	child.insert("in".into(), v);
               	}
               	t.render(scope)
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
    				write!(f, r#""{}": {}"#, k, v)?;
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
        }

		Ok(())
    }
}
