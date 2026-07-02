use crate::ServValue;
use crate::ServType;

// use crate::parsetool;

use crate::parser::ParseError;

#[derive(Debug)]
pub enum ServError {
    Empty,
    Todo,
    General(u16, String),
    Io(std::io::Error),
    Utf8(std::str::Utf8Error),
    Fmt(std::fmt::Error),
    MissingLabel(crate::engine::dictionary::Label),

    UnexpectedType(ServType, ServType),
    InsertWithEmptyAddress,
    InsertIntoInvalidType,

}

impl ServError {
	pub fn new(code: u16, message: &str) -> Self {
    	Self::General(code, message.into())
	}

	pub fn expected_type<T: Into<ServType>>(expected: ServType, actual: T) -> Self {
    	Self::UnexpectedType(expected, actual.into())
	}
}

impl std::fmt::Display for ServError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::General(code, message) => write!(f, "err {}: {}", code, message),
            Self::Io(err) => write!(f, "io error: {}", err),
            Self::Fmt(err) => write!(f, "fmt error: {}", err),
            Self::MissingLabel(label) => write!(f, "missing label {}", label),

            Self::UnexpectedType(expected, actual) => write!(f, "expected type {}, found {}", expected, actual),
            Self::InsertWithEmptyAddress => f.write_str("empty address"),

            other => write!(f, "{:?}", self),
        }
    }
}

impl From<std::str::Utf8Error> for ServError {
    fn from(input: std::str::Utf8Error) -> Self {
        Self::Utf8(input)
    }
}

impl From<&str> for ServError {
    fn from(input: &str) -> Self {
        Self::new(500, input)
    }
}

impl From<std::io::Error> for ServError {
    fn from(input: std::io::Error) -> Self {
        Self::Io(input)
    }
}

impl From<std::fmt::Error> for ServError {
    fn from(input: std::fmt::Error) -> Self {
        Self::Fmt(input)
    }
}

impl From<ParseError> for ServError {
    fn from(input: ParseError) -> Self {
        Self::Todo
        // Self::new(500, &input.to_string())
    }
}
