

#[derive(Debug)]
pub struct ServError {
    message: String,
}

impl ServError {
	pub fn new(input: &str) -> Self {
    	Self { message: input.to_string() }
	}
}

impl std::fmt::Display for ServError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.write_str(self.message.as_str())
    }
}

impl From<&str> for ServError {
    fn from(input: &str) -> Self {
        Self::new(input)
    }
}

impl From<std::io::Error> for ServError {
    fn from(input: std::io::Error) -> Self {
        Self { message: format!("io error: {}", input) }
    }
}
