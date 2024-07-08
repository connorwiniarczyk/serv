use crate::ast;
use crate::template::Template;
use std::collections::VecDeque;
use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum ServValue {
    None,
    Int(i64),
    List(VecDeque<ServValue>),
    Text(String),
}

impl ServValue {
    pub fn expect_int(self) -> Result<i64, &'static str> {
        let Self::Int(i) = self else { return Err("expected an int") };
        Ok(i)
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
            Self::None => f.write_str("none")?,
            Self::Text(ref t) => f.write_str(t)?,
            Self::Int(i) => write!(f, "{}", i)?,
            Self::List(l) => {
                f.write_str("[")?;
				let mut iter = l.iter().peekable();
				while let Some(element) = iter.next() {
    				write!(f, "{}", element)?;
    				if iter.peek().is_some() {f.write_str(", ")?}
				}
                f.write_str("]")?;
            }
        }
		Ok(())
    }
}
