use crate::cursor::Token;

#[derive(Debug)]
pub struct ServError;

impl From<&str> for ServError {
    fn from(_i: &str) -> Self {
        Self
    }
}

pub struct Parser<'input, K> {
    input: &'input [Token<K>],
    index: isize,
}

impl<'input, K> Parser<'input, K> where K: Clone + PartialEq {
    fn len(&self) -> isize {
        self.input.len().try_into().unwrap()
    }

    pub fn new(input: &'input [Token<K>]) -> Self {
		Self { input, index: 0 }
    }

    pub fn incr(&mut self) -> Result<(), ServError> {
        self.index += 1;
        if self.index >= self.len() {
            return Err("out of bounds".into())
        }

		Ok(())
    }

    pub fn current(&self) -> Result<Token<K>, ServError> {
        Ok(self.input[self.index as usize].clone())
    }

    pub fn get(&self, offset: isize) -> Result<&'input Token<K>, ServError> {
        let i = self.index + offset;
        if i < 0 || i >= self.len() { return Err("out of bounds".into()); }

       	Ok(&self.input[i as usize])
    }

    pub fn next_if_kind(&mut self, kind: K) -> Result<Token<K>, ServError> {
        if self.get(0)?.kind == kind { self.incr()?; self.get(-1).cloned() }
        else {
            Err("incorrect kind".into())
        }
    }

    pub fn expect(&mut self, kind: K) -> Result<Token<K>, ServError> {
        if self.current()?.kind == kind {
            Ok(self.current()?)
        } else {
            Err("failed assertion!".into())
        }
    }
}
