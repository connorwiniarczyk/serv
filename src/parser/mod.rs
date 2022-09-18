pub mod token;
pub mod parser;
pub mod error;

use error::{Error, ParseResult};
use parser::Parser;
use std::fs::File;
use std::path::Path;
use std::str::FromStr;
use std::io::Read;

use token::{Token, TokenKind, TokenKind::*};

use super::route_table::{RouteTable, Route};
use super::pattern::{Pattern, Node};

// use crate::command::command::Command;
// use crate::command::Command;

use crate::command::Command;

trait FromToken {
    fn from_token(token: &Token) -> Result<Self, ()> where Self: Sized;
}

impl FromToken for Route {
    fn from_token(token: &Token) -> Result<Self, ()> {
        let token_cl = token.clone();

        let pattern = Pattern::from_token(token_cl.get_child(Path).unwrap()).unwrap();
        let command_list = token_cl.get_child(CommandList).unwrap().clone();

        let mut commands = vec![];
        for command_token in command_list.children.into_iter().filter(|x| x.kind == Command) {
            let command = Command::from_token(&command_token)?;
            commands.push(command);
        }

        Ok(Route{pattern, commands})
    }
}

impl FromToken for Pattern {
    fn from_token(token: &Token) -> Result<Self, ()> {
        if token.kind != Path { return Err(()); }
        let mut output = Pattern::new(vec![]);

        for child in token.children.iter() {
            match child.kind {
                PathAttribute => {
                    let attribute = match &child.value {
                        Some(attr) => attr.as_str(),
                        None => continue,
                    };
                    // let attribute = child.value.as_ref().unwrap_or(continue).as_str();
                    // println!("{:?}", child.value.as_ref().unwrap_or());

                    match attribute {
                        method @ ("GET" | "POST" | "PUT" | "CONNECT" | "DELETE" | "HEAD" | "OPTIONS" | "PATCH" | "TRACE") => {
                            output.methods.insert(FromStr::from_str(method).unwrap());
                        },
                        name => output.name = Some(name.to_string()),
                    };

                    output.attributes.push(attribute.to_string());
                }
                PathNode => output.path.push(Node::from_str(&child.value.as_ref().unwrap_or(&String::new()))),

                PathExt => output.extension = Some(Node::from_str(&child.value.as_ref().unwrap_or(&String::new()))),
                // PathExt => output.extension.push(Node::from_str(&child.value.as_ref().unwrap_or(&String::new()))),
                _ => return Err(()),
            }
        }

        Ok(output)
    }
}

impl FromToken for Command {
    fn from_token(token: &Token) -> Result<Self, ()> {
        if token.kind != Command { return Err(()); }

        let name = token
            .get_child(CommandName)
            .ok_or(())?
            .value.as_ref()
            .ok_or(())?;

        fn get_arg(token: &Token) -> Option<&str> {

            let child = token.get_child(CommandArg)?;

            match child.get_child(Block) {
                Some(multi_line) => multi_line.value.as_deref(),
                None => child.value.as_deref(),
            }

            // token.get_child(CommandArg)?.value.as_deref()
        }

        let result = Command::new(&name, get_arg(token));

        Ok(result)
    }
}

struct RouteTableBuilder {
    tree: Token,
}

impl RouteTableBuilder {
    pub fn generate(self) -> Result<RouteTable, Error> {
        let mut output = RouteTable::new();

        for route_token in self.tree.children.into_iter().filter(|x| x.kind == Route) {
            output.add(Route::from_token(&route_token).unwrap()); 
        }

        Ok(output)
    }
}


pub fn parse<R: Read>(input: R) -> Result<RouteTable, Error> {
    let syntax_tree = Parser::new(input).parse()?;
    let builder = RouteTableBuilder { tree: syntax_tree };
    builder.generate()
}

// pub fn parse_route_file(path: &Path) -> Result<RouteTable, Error> {

//     let file = File::open(&path)?;
//     let mut parser = Parser::new(file);
//     let result = parser.parse()?;

//     // println!("test");
//     println!("\n{}", result);

//     let builder = RouteTableBuilder { tree: result };
//     builder.generate()
// }

pub fn parse_str(input: &str) -> Result<RouteTable, Error> {
    parse(input.as_bytes())
}


