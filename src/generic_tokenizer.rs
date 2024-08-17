#[derive(Debug, Clone, Copy)]
pub struct Token<'input> {
	pub value: &'input [char],
	pub start: usize,
	pub end: usize,
}

impl<'input> Token<'input> {
	pub fn to_string(&self) -> String {
    	let mut output = String::new();
    	for c in self.value {
        	output.push(*c)
    	}
    	output
	}
}

pub struct Tokenizer<'input> {
    input: &'input [char],
    mark: usize,
    index: usize,
}

impl<'input> Tokenizer<'input> {
    pub fn new(input: &'input [char]) -> Self {
        Self { input, mark: 0, index: 0}
    }

    pub fn emit(&mut self) -> Token<'input> {
        let output = Token {
            value: &self.input[self.mark..self.index],
			start: self.mark,
			end: self.index,
        };

        self.mark = self.index;
        output
    }

    pub fn get(&mut self, offset: usize) -> Option<char>{
        let index = self.index + offset;
        if index >= self.input.len() { return None };

        Some(self.input[index])
    }

    pub fn incr_while<F>(&mut self, test: F) where F: Fn(char) -> bool {
        while (self.index < self.input.len() && (test)(self.input[self.index])) {
            self.index += 1;
        }
    }
    pub fn incr(&mut self, i: usize) {
        self.index += i;
    }

    pub fn is_done(&self) -> bool {
		self.mark >= self.input.len()
    }

    pub fn skip(&mut self) {
        if self.index < self.input.len() {
            self.index += 1;
        }

        self.mark = self.index;
    }
}
