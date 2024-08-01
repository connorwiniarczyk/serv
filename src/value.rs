use crate::ast;
use crate::template::Template;
use std::collections::VecDeque;
use std::collections::HashMap;
use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum ServValue {
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

impl Default for ServValue {
	fn default() -> Self {
    	Self::None
	}
}

impl ServValue {
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
            Self::Raw(bytes) => f.debug_list().entries(bytes.iter()).finish()?,
            Self::Int(i) => write!(f, "{}", i)?,
            Self::Meta { inner, metadata } => inner.fmt(f)?,
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
        }
		Ok(())
    }
}
