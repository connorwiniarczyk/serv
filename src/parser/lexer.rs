

pub enum Token {
    BeginRoute,
    Slash,
    Dot,
    PathValue,
    PathWildcard,
    PathDoubleWildcard,
    PathEnd,
    Separator,
}

pub struct Lexer<R: Read> {}

impl<R: Read> Lexer {
    pub fn new(input: R) -> Self {
        todo!();
    }
}

impl Iterator<Token> for Lexer {
    fn next(&mut self) -> Option<Token> {
        todo!();
    }
}
