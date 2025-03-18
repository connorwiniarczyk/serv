use std::fmt::Display;


#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum Label {
    Name(String),
    Route(String),
}

impl Label {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Name(v) => v.as_str(),
            Self::Route(v) => v.as_str(),
        }
    }
 }

impl From<&str> for Label {
    fn from(input: &str) -> Self {
        if input.chars().nth(0) == Some('/') {
            Self::Route(input.to_owned())
        } else {
            Self::Name(input.to_string())
        }
    }
}

// Iterator around address
struct AddressIter<'a>(&'a Address, usize);

impl<'a> Iterator for AddressIter <'a> {
    type Item = &'a Label;

    fn next(&mut self) -> Option<Self::Item> {
        if self.1 >= (self.0.0.len() - 0) { return None };

		let i = self.1;
        self.1 += 1;

        return Some(&self.0.0[i])
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct Address(Vec<Label>);

impl Address {
    pub fn iter(&self) -> impl Iterator<Item = &Label> {
        AddressIter(self, 0)
    }

    pub fn len(&self) -> usize {
        return self.0.len()
    }
}

impl From<&str> for Address {
    fn from(input: &str) -> Self {
        let mut output = Vec::new();
        for part in input.split(".") {
            output.push(Label::Name(part.to_owned()))
        }

        Self(output)
    }
}

impl From<Label> for Address {
    fn from(input: Label) -> Self {
        Self(vec![input])
    }
}

impl Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match (self) {
            Self::Name(s) => f.write_str(s)?,
            Self::Route(s) => f.write_str(s)?,
            // Self::Anonymous(id) => write!(f, "anonymous function {}", id)?,
        };

        Ok(())
    }
}
