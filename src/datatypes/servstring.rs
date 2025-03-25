use bytes::Bytes;
use crate::ServError;
use crate::ServValue;

#[derive(Debug, Clone)]
enum Data {
	Text(String),
	Bytes(Bytes),
}

impl From<String> for Data {
	fn from(input: String) -> Self {
		Self::Text(input)
	}
}

impl From<Bytes> for Data {
	fn from(input: Bytes) -> Self {
		Self::Bytes(input)
	}
}

impl From<&str> for Data {
	fn from(input: &str) -> Data {
		Self::Text(input.to_owned())
	}
}

impl From<&[u8]> for Data {
	fn from(input: &[u8]) -> Data {
		Self::Bytes(Bytes::copy_from_slice(input))
	}
}

#[derive(Debug, Clone)]
pub struct ServString {
    pub mime: Option<&'static str>,
    data: Data,
}

impl ServString {
    pub fn as_str(&self) -> Result<&str, ServError> {
        match &self.data {
			Data::Text(ref s) => Ok(s),
			Data::Bytes(b) => Ok(std::str::from_utf8(b)?),
        }
    }

   pub fn from_bytes<T: Into<Bytes>>(input: T) -> Self {
        let data: Bytes = input.into();
        if let Ok(text) = std::str::from_utf8(&data) {
            Self { mime: None, data: text.into() }
        }
        else {
            Self { mime: None, data: data.into() }
        }

    }

    pub fn from_text<T: Into<String>>(input: T) -> Self {
        let data: String = input.into();
        Self { mime: None, data: data.into() }
    }

    pub fn as_value(self) -> ServValue {
        ServValue::Text(self)
    }
}

impl<I> From<I> for ServString where I: Into<Data> {
    fn from(input: I) -> Self {
        Self { mime: None, data: input.into() }
    }
}
