use std::fmt::{ Display, Formatter, Error as FmtError };
use crate::template::Template;

pub enum ServValueKind {
    None,
    Int,
    Text,
    Template,
    List,
}

#[derive(Debug)]
pub enum ServValue {
	None,
	Boolean(bool),
	Int(i64),
	Float(f64),
	Text(String),
	Bin(Vec<u8>),
	List(Vec<ServValue>),
	Template(Template),
	// Scope(WordTable),
}

impl ServValue {
	pub fn from_str(input: &str) -> Self {
		Self::Text(input.chars().collect())
	}

	pub fn as_str(&self) -> &str {
		match self {
			Self::None => "",
			Self::Text(ref s) => s,
			Self::Int(i) => "integer",
	        Self::List(elements) => "list",
			_ => todo!(),
		}
	}

	pub fn to_string(&self) -> String {
		match self {
			Self::None => "".to_owned(),
			Self::Text(s) => s.clone(),
			Self::Int(i) => i.to_string(),
	        Self::List(elements) => elements.iter().map(|x| x.to_string()).collect(),
			_ => todo!(),
		}
	}

	pub fn as_bytes(&self) -> &[u8] {
		match self {
			Self::Bin(ref b) => b,
			x @ _ => x.as_str().as_bytes(),
		}
	}

	pub fn as_int(&self) -> Result<i64, &'static str> {
		if let Self::Int(i) = self {
			Ok(i.clone())
		} else {
			Err("!")
		}

	}
}

impl TryFrom<ServValue> for String {
	type Error = &'static str;

	fn try_from(input: ServValue) -> Result<String, Self::Error> { Ok(input.to_string()) }
}

impl TryFrom<ServValue> for Vec<ServValue> {
	type Error = &'static str;
	fn try_from(input: ServValue) -> Result<Vec<ServValue>, Self::Error> {
		match input {
            ServValue::List(l) => Ok(l),
            _ => Err("!"),
		}
	}
}

impl TryFrom<ServValue> for i64 {
	type Error = &'static str;

	fn try_from(input: ServValue) -> Result<i64, Self::Error> {
		match input {
			ServValue::Int(i) => Ok(i),
			ServValue::Float(f) => Ok(f.floor() as i64),
			_ => Err("could not convert this type to an integer"),
		}
	}
}

impl<T> From<Vec<T>> for ServValue where T: Into<ServValue> {
	fn from(input: Vec<T>) -> Self {
		let inner_list: Vec<ServValue> = input.into_iter().map(|x| x.into()).collect();
		Self::List(inner_list)
	}
}

impl From<String> for ServValue {
	fn from(input: String) -> Self { Self::Text(input) }
}

impl From<&str> for ServValue {
	fn from(input: &str) -> Self { Self::Text(input.to_owned()) }
}

impl From<i64> for ServValue {
	fn from(input: i64) -> Self { Self::Int(input) }
}


impl Display for ServValue {
	fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
		match self {
			Self::None => f.write_str("None")?,
			Self::Boolean(b) => if *b {f.write_str("true")?} else {f.write_str("false")?},
			Self::Int(i) => write!(f, "{}", i)?,
			Self::Float(i) => write!(f, "{}", i)?,
			Self::Text(ref t) => f.write_str(t)?,
			Self::List(t) => {
				f.write_str("[\n")?;
				let mut iter = t.iter().peekable();
				while let Some(elem) = iter.next() {
					f.write_str("  ")?;
					f.write_str(elem.to_string().as_str())?;
					f.write_str(",\n")?;
					// if iter.peek().is_some() { f.write_str(",\n")?; }
				}
				f.write_str("]")?;
			},
			Self::Bin(b) => {
				let text = std::str::from_utf8(b).unwrap();
				f.write_str(text)?;
			},
		};

		Ok(())
	}
}

use hyper::body::{ Body, Buf, Frame };
use std::task::{ Poll, Context };
use std::pin::Pin;
use std::collections::VecDeque;
use std::future::Future;

pub struct ServBody(Option<VecDeque<u8>>);

impl ServBody {
	pub fn new() -> Self {
		Self(Some("hello!".bytes().collect()))
	}
}

impl From<ServValue> for ServBody {
	fn from(input: ServValue) -> Self {
		Self(Some(input.to_string().bytes().collect()))
	}
}

impl Body for ServBody {
	type Data = VecDeque<u8>;
	type Error = &'static str;

	fn poll_frame(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
		if let Some(data) = self.get_mut().0.take() {
			Poll::Ready(Some(Ok(Frame::data(data))))
		} else {
			Poll::Ready(None)
		}
	}
}


