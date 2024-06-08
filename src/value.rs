use crate::ast;
use crate::template::Template;
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub enum ServValue {
    Int(i64),
    Template(ast::Template),
    List(VecDeque<ServValue>),
    Text(String),
}

impl ServValue {
    pub fn expect_int(self) -> Result<i64, &'static str> {
        if let Self::Int(i) = self {
            Ok(i)
        } else {
			Err("expected an int")
        }
    }
}

impl From<i64> for ServValue {
    fn from(value: i64) -> Self {
        Self::Int(value)
    }
}

impl From<Template> for ServValue {
    fn from(value: Template) -> Self {
        Self::Template(value)
    }
}
