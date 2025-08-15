pub mod cursor;
pub mod parser;




#[derive(Debug)]
pub struct ParseError {
    message: String
}

impl ParseError {
    pub fn new(message: &str) -> Self {
        Self { message: message.to_owned() }
    }
}

impl From<&str> for ParseError {
    fn from(input: &str) -> Self {
        Self::new(input)
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.write_str(&self.message)
    }
}
