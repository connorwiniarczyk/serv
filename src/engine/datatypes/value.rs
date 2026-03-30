use std::collections::VecDeque;
use std::collections::HashMap;
use std::fmt::Display;

use crate::template::Template;
use crate::Stack;
use crate::Label;
use crate::ServModule;
use crate::ServError;
use crate::ServResult;
use crate::functions::json;

pub use crate::servstring::ServString;
pub use crate::servlist::ServList;

use super::super::dictionary::Address;
use super::super::dictionary::DatabaseConnection;

#[derive(Clone)]
pub enum ServFn {
    // Ident,
    Core     (fn(ServValue, &Stack) -> ServResult),
    Meta     (fn(ServList, &mut Stack) -> ServResult),
    ArgFn    (fn(ServValue, ServValue, &Stack) -> ServResult),
    Expr     (ServList, bool),

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
            // Self::Ident       => f.write_str("Ident"),
            Self::Core(_)     => f.write_str("Core"),
            Self::Meta(_)     => f.write_str("Meta"),
            Self::ArgFn(_)    => f.write_str("ArgFn"),
            Self::Expr(_, _)  => f.write_str("Expr"),
            Self::Template(_) => f.write_str("Template "),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum ServType {
    None,
    Ref,
    Func,
    Bool,
    Int,
    Float,
    Text,
    List,
    Table,
    Module,
}

impl From<&ServValue> for ServType {
    fn from(input: &ServValue) -> Self {
        match input {
			ServValue::None      => Self::None,
			ServValue::Ref(_)    => Self::Ref,
			ServValue::Func(_)   => Self::Func,
			ServValue::Bool(_)   => Self::Bool,
			ServValue::Int(_)    => Self::Int,
			ServValue::Float(_)  => Self::Float,
			ServValue::Text(_)   => Self::Text,
			ServValue::List(_)   => Self::List,
			ServValue::Table(_)  => Self::Table,
			ServValue::Module(_) => Self::Module,
        }
    }
}

impl From<ServValue> for ServType {
    fn from(input: ServValue) -> Self {
        ServType::from(&input)
    }
}

impl Display for ServType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::None   => f.write_str("NONE"),
            Self::Ref    => f.write_str("REF"),
            Self::Func   => f.write_str("FUNC"),
            Self::Bool   => f.write_str("BOOL"),
            Self::Int    => f.write_str("INT"),
            Self::Float  => f.write_str("FLOAT"),
            Self::Text   => f.write_str("TEXT"),
            Self::List   => f.write_str("LIST"),
            Self::Table  => f.write_str("TABLE"),
            Self::Module => f.write_str("MODULE"),
        };

		Ok(())
    }
}

#[derive(Clone, Debug)]
pub enum ServValue {
    None,
    Ref(Address),
    Func(ServFn),
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
        crate::engine::resolve(self.clone(), input, scope)
    }

    pub fn expect_int(&self) -> Result<i64, &'static str> {
        let Self::Int(i) = self else { return Err("expected an int") };
        Ok(i.clone())
    }

    pub fn expect_module(self) -> Result<ServModule, ServError> {
        let ServValue::Module(m) = self else {
			return Err(ServError::expected_type(ServType::Module, self));
        };

		Ok(m)
    }

    pub fn expect_ref(self) -> Result<Address, ServError> {
        let ServValue::Ref(addr) = self else {
			return Err(ServError::expected_type(ServType::Ref, self));
        };

		Ok(addr)
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
        let ServValue::Text(t) = self else {
			return Err(ServError::expected_type(ServType::Text, self));
        };

		t.as_str()
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

impl From<ServModule> for ServValue {
    fn from(value: ServModule) -> Self {
        Self::Module(value)
    }
}

impl From<String> for ServValue {
    fn from(value: String) -> Self {
        Self::Text(value.into())
    }
}

impl From<ServString> for ServValue {
    fn from(value: ServString) -> Self {
        Self::Text(value)
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
			// ServValue::Ref(label) => self.write(self.0.get(label)?, dest)?,
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


impl Display for ServValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let mut scope = Stack::empty();
        DefaultSerializer(&scope).write(self.clone(), f).unwrap();
		Ok(())
    }
}
