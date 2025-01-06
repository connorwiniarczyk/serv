use crate::ServValue;

#[derive(Debug)]
pub enum ServError {
    General(u16, String),
    Io(std::io::Error),
    Fmt(std::fmt::Error),
    MissingLabel(crate::dictionary::Label),
    UnexpectedType(String, ServValue),

}

impl ServError {
	pub fn new(code: u16, message: &str) -> Self {
    	Self::General(code, message.into())
	}

	pub fn expected_type(expected: &str, actual: ServValue) -> Self {
    	Self::UnexpectedType(expected.into(), actual)
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
        }
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
