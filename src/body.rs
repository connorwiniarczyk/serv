
use hyper;


#[derive(Debug, Clone)]
pub struct HttpError {
    code: u32,
    message: Option<String>,
}

use std::fmt::{Display, Formatter};

impl Display for HttpError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match &self.message {
            Some(message) => write!(f, "{}: {}", self.code, message),
            None => write!(f, "{}", self.code),
        }
    }
}

impl From<std::io::Error> for HttpError {
    fn from(input: std::io::Error) -> Self {
        Self {
            code: 404,
            message: Some(input.to_string()),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Body {
    // Bytes(Bytes),
    Raw(Vec<u8>),
    Txt(String),
    // TODO: Stream(Stream),

    Err(HttpError),
}

use Body::*;

impl Body {
    pub fn from_str(input: &str) -> Self {
        Txt(input.to_string())
    }

    pub fn from_bytes(input: impl Into<Vec<u8>>) -> Self {
        Raw(input.into())
    }

    // TODO: this should probably be a proper trait
    fn add(mut l: Self, mut r: Self) -> Self {
        match (l, r) {
            (Raw(mut a), Raw(mut b)) => { a.append(&mut b); Raw(a)}
            (Txt(mut a), Txt(mut b)) => { a.push_str(&mut b); Txt(a)}
            (Raw(mut a), Txt(mut b)) => { a.append(&mut b.as_bytes().to_vec()); Raw(a)}
            (Txt(mut a), Raw(mut b)) => { 
                let mut new = a.as_bytes().to_vec();
                new.append(&mut b);
                Raw(new)
            },
            (Err(a), _) => Err(a),
            (_, Err(b)) => Err(b),
        }
    }

    pub fn append<T: Into<Self>>(&mut self, input: T) {
        //TODO: getting rid of this clone might be important for optimization later
        *self = Self::add(self.clone(), input.into());
    }

    pub fn replace<T: Into<Self>>(&mut self, input: T) {
        *self = input.into();
    }

    pub fn data(&self) -> &[u8] {
        match self {
            Txt(a) => a.as_bytes(),
            Raw(a) => a.as_slice(),
            Err(e) => "error".as_bytes(),
        }
    }
}

impl Into<hyper::Body> for Body {
    fn into(self) -> hyper::Body {
        match self {
            Txt(x) => x.into(),
            Raw(x) => x.into(),
            Err(err) => err.to_string().into(),
        }
    }
}

impl From<hyper::Body> for Body {
    fn from(input: hyper::Body) -> Self {
        todo!();
    }
}

impl From<&str> for Body {
    fn from(input: &str) -> Self {
        Txt(input.to_string())
    }
}

impl From<&[u8]> for Body {
    fn from(input: &[u8]) -> Self {
        Raw(input.to_vec())
    }
}
