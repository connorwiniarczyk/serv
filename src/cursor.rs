#[derive(Debug, Clone)]
pub struct Token<K> {
    pub kind: K,
	pub value: String,
	pub start: usize,
	pub end: usize,
}

impl<K: Clone> Token<K> {
	pub fn to_string(&self) -> String {
    	self.value.clone()
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

    pub fn emit<K>(&mut self, k: K) -> Token<K> {
        let value: String = self.input[self.mark..self.index].iter().collect();
        let output = Token {
            kind: k,
            value,
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
