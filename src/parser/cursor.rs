use super::*;

#[derive(Clone)]
pub struct Cursor<'input> {
	text: &'input Vec<char>,
	pub position: usize,
	line: usize,
	column: usize,
}

impl<'input> Cursor<'input> {
	pub fn new(input: &'input Vec<char>) -> Self {
		Self { text: input, position: 0, line: 0, column: 0 }
	}

	pub fn next(&mut self) -> Option<char> {
		if self.position >= self.text.len() { return None }
		let char = self.text[self.position];
		if char == '\n' { self.line += 1; self.column = 0; }
		self.position += 1;
		Some(char)
	}

	pub fn peek(&mut self) -> Option<char> { self.clone().next() }

	pub fn expect(&mut self, input: &str) -> Result<(), MatchFail> {
		let mut test_cursor = self.clone();
		let mut chars = input.chars();
		while let Some(c) = chars.next() {
			if test_cursor.next() != Some(c) { return Err(MatchFail); }
		}
		*self = test_cursor;
		Ok(())
	}

	pub fn skip(&mut self, input: &[char]) -> () {
		while let Some(ref c) = self.peek() {
			if input.contains(c) == false { break; }
			self.next();
		}
	}

	pub fn consume_identifier(&mut self) -> Result<String, MatchFail> {
		let mut output = String::new();
		let mut clone = self.clone();
		while let Some(c) = clone.next() {
			if c.is_alphanumeric() || c == '_' {
				output.push(c);
				self.next();
			}
			else { break; }

		}
		if output.len() < 1 { return Err(MatchFail); }
		Ok(output)
	}

	pub fn consume_path_segment(&mut self) -> Result<String, MatchFail> {
		let mut output = String::new();
		let mut clone = self.clone();
		while let Some(c) = clone.next() {
			match c {
				'/' | ':' | '.' => break,
				_ => { output.push(c); self.next(); },
			}
		}
		Ok(output)
		
	}

	pub fn debug(&self) {
		let mut clone = self.clone();
		print!("cursor: ");
		while let Some(c) = clone.next() {
			if c == '\n' { break; }
			print!("{}", c);
		}
		print!("\n");
	}
}
