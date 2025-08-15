pub mod tokenizer;
pub mod parser;

mod walker;
pub mod cursor;

pub use cursor::{Token, Tokenizer};

use crate::ServModule;
use crate::Stack;
use crate::ServValue;
use crate::Template;

use parser::Parser;
use parser::parse_module;

// pub enum ParseError {
//     Empty,
//     EndOfInput,
//     UnexpectedToken(),
// }


#[derive(Debug)]
pub struct ParseError {
    message: String
}

impl ParseError {
    pub fn new(message: &str) -> Self {
        Self { message: message.to_owned() }
    }
}

impl From<&str> for ParseError {
    fn from(input: &str) -> Self {
        Self::new(input)
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.write_str(&self.message)
    }
}


pub fn parse_template_from_text(input: &str, brackets: bool) -> Result<Template, crate::ServError> {
    todo!();
    // let chars: Vec<char> = input.chars().collect();
    // let tokens = servlexer::tokenize_template(&chars);
    // let mut parser = Parser::new(&tokens);
    // let ast = parse_template(&mut parser)?;

    // Ok(ast)
}

pub fn parse_expression_from_text(input: &str) -> Result<ServValue, crate::ServError> {
    todo!();
    // let chars: Vec<char> = input.chars().collect();
    // let tokens = servlexer::tokenize_serv(&chars);
    // let mut parser = Parser::new(&tokens);
    // let ast = parse_expression(&mut parser)?;

    // Ok(ast)
}

pub fn parse_root_from_text(input: &str, ctx: &mut Stack) -> Result<ServModule, crate::ServError> {
    let chars: Vec<char> = input.chars().collect();
    let tokens = tokenizer::tokenize(&chars);
    let mut parser = Parser::new(&tokens);
    let ast = parse_module(&mut parser, ctx)?;

    Ok(ast)
}
