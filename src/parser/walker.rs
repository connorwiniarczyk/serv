use super::cursor::Token;
use super::ParseError;

// #[derive(Debug)]
// pub struct ParseError;

// impl From<&str> for ParseError {
//     fn from(_i: &str) -> Self {
//         Self
//     }
// }

pub struct Walker<'input, K> {
    input: &'input [Token<K>],
    index: isize,
}

impl<'input, K> Walker<'input, K> where K: Copy + Clone + PartialEq {
    fn len(&self) -> isize {
        self.input.len().try_into().unwrap()
    }

    pub fn new(input: &'input [Token<K>]) -> Self {
		Self { input, index: 0 }
    }

    pub fn incr(&mut self) -> Result<(), ParseError> {
        self.index += 1;
        if self.index >= self.len() {
            return Err("out of bounds".into())
        }

		Ok(())
    }

    pub fn current(&self) -> Result<Token<K>, ParseError> {
        Ok(self.input[self.index as usize].clone())
    }

    pub fn get(&self, offset: isize) -> Result<&'input Token<K>, ParseError> {
        let i = self.index + offset;
        if i < 0 || i >= self.len() { return Err("out of bounds".into()); }

       	Ok(&self.input[i as usize])
    }

    pub fn kind(&self, offset: isize) -> Result<K, ParseError> {
		Ok(self.get(offset)?.kind)
    }

    pub fn next_if_kind(&mut self, kind: K) -> Result<Token<K>, ParseError> {
        if self.get(0)?.kind == kind { self.incr()?; self.get(-1).cloned() }
        else {
            Err("incorrect kind".into())
        }
    }

    pub fn expect(&mut self, kind: K) -> Result<Token<K>, ParseError> {
        if self.current()?.kind == kind {
            Ok(self.current()?)
        } else {
            Err("failed assertion!".into())
        }
    }
}
