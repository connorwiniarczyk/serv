mod token;
mod parser;
mod error;

use error::{Error, ParseResult};
use parser::Parser;
use std::fs::File;
use super::route_table::RouteTable;
use std::path::Path;

use token::{Token, TokenKind, TokenKind::*};

use super::pattern::Pattern;
use super::command::Command;

trait FromToken {
    fn from_token(token: Token) -> Result<Self, ()> where Self: Sized;
}

impl FromToken for Pattern {
    fn from_token(token: Token) -> Result<Self, ()> {
        if (token.kind != Path) { return Err(()); }

        todo!();
    }
}

struct RouteTableBuilder {
    tree: Token,
}

impl RouteTableBuilder {
    pub fn generate(self) -> Result<RouteTable, Error> {
        todo!();
    }
}


pub fn parse_route_file(path: &Path) -> Result<RouteTable, Error> {

    let file = File::open(&path).unwrap();
    let mut parser = Parser::new(file);
    let result = parser.parse()?;

    println!("{}", result);

    let builder = RouteTableBuilder { tree: result };
    builder.generate()
}


