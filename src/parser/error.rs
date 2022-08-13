
use std::fmt::{Display, Debug, Formatter};


pub struct Error {
    message: String,
}

impl Error {
    pub fn new(message: &str) -> Self {
        Self { message: message.to_string() }
    }
}


impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self.message)
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self.message)
    }
}

pub type ParseResult = Result<(), Error>;
