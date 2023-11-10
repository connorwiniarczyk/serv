use lexer::{Token};
struct Parser;

impl<I> Parser where I: Iterator<Token> {
    fn new(input: I) -> Self { todo!(); }
    fn parse(self) -> Ast { todo!(); }
}
