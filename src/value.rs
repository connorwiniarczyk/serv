use std::collections::VecDeque;
use std::collections::HashMap;
use std::fmt::Display;

use crate::template::Template;
use crate::Stack;
use crate::Label;
use crate::ServError;
use crate::ServResult;


pub use crate::servstring::ServString;
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

impl From<ServFn> for ServValue {
    fn from(input: ServFn) -> Self{
       Self::Func(input)
    }
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
    Text(ServString),
    List(ServList),
    Table(HashMap<String, ServValue>),

    Module(crate::ServModule),
}

impl ServValue {
    pub fn call(&self, input: Option<ServValue>, scope: &Stack) -> ServResult {
       	match self {
           	Self::Ref(label) => scope.get(label.clone())?.call(input, scope),
			Self::Module(m) => m.clone().call(input, &mut scope.make_child()),

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
                   	child.insert_name("in", v.clone());
                   	child.insert_name(":", v);
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

    pub fn as_str(&self) -> Result<&str, ServError> {
        match self {
            ServValue::Text(t) => t.as_str(),
            otherwise => Err(ServError::expected_type("string", self.clone())),
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

impl From<String> for ServValue {
    fn from(value: String) -> Self {
        Self::Text(value.into())
    }
}

type Buffer<'a> = &'a mut (dyn std::fmt::Write + 'a);

pub trait Serializer {
	fn write<'buf>(&mut self, value: ServValue, dest: Buffer<'buf>) -> Result<(), ServError>;
}

#[derive(Clone)]
pub struct DefaultSerializer<'s>(pub &'s Stack<'s>);

impl<'a> Serializer for DefaultSerializer<'a> {
    fn write<'b>(&mut self, value: ServValue, dest: Buffer<'b>) -> Result<(), ServError> {
        match value {
			ServValue::Ref(label) => self.write(self.0.get(label)?, dest)?,
			f @ ServValue::Func(_) => self.write(f.call(None, self.0)?, dest)?,

			ServValue::Text(t) => {
    			if let Ok(inner) = t.as_str() {
        			dest.write_str(inner)?;
    			} else {
        			dest.write_str("RAW")?;
    			}
			},

			otherwise => {
				json::serializer(self.0).write(otherwise, dest)?
			},
       };

        Ok(())
    }
}

use crate::functions::json;

impl Display for ServValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let mut scope = Stack::empty();
        DefaultSerializer(&scope).write(self.clone(), f).unwrap();
		Ok(())
    }
}
