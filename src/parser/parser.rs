use std::io::Read;

use super::token::{ Token, TokenKind };
use super::token::TokenKind::*;
use super::error::{Error, ParseResult};

pub struct Parser<R: Read> {
    reader: R,
    pub tree: Vec<Token>,
    buffer: String,
}


impl<R: Read> Parser<R> {
    pub fn new(reader: R) -> Self {
        Self { 
            reader,
            tree: vec![Token::new(Root)],
            buffer: String::new(),
        }
    }

    fn step(&mut self, character: char) -> ParseResult {
        match (self.mode(), character) {
            (context, '#') if context != Comment && context != Block => self.enter(Comment, None),
            (Comment, '\n') => self.exit(None),
            (Comment, _) => (),

            // describe the parsing modes where whitespace or newline characters should be ignored
            (Root | Route | CommandList | Command | Path, c) if self.is_whitespace(c) => (),
            (Root | CommandList | Route, '\n') => (),

            // new Routes can be denoted with either the / or @ symbols
            (Root,  c @ ('/' | '@')) => {
                self.enter(Route, None);
                self.enter(Path, None);
                match c {
                    '/' => self.enter(PathNode, None),
                    '@' => self.enter(PathAttribute, None),
                    _   => unreachable!(),
                };
            },
            
            // Logic for transitioning from one path element to another
            (PathAttribute, '@') => { self.exit(None); self.enter(PathAttribute, None) },
            (PathAttribute | PathNode, '/') => { self.exit(None); self.enter(PathNode, None) },
            (PathNode, '.') => { self.exit(None); self.enter(PathExt, None) },

            // if a colon is seen anywhere in the path, leave Path parsing and enter CommandList
            // parsing
            (Path | PathNode | PathExt | PathAttribute, ':') => {
                self.exit_until(Route, None);
                self.enter(CommandList, None);
            },

            // allow whitespace in between the end of the path and the colon
            (PathAttribute | PathNode | PathExt , c) if self.is_whitespace(c) => self.exit_until(Path, Some(c)),

            // Define valid characters for path elements
            (PathNode | PathExt | PathAttribute, c @ ('a'..='z' | 'A'..='Z' | '0'..='9' | '*' | '_')) => self.buffer.push(c),


            // Exit CommandList parsing when we see the beginning of a new Path
            (CommandList, c @ ('/' | '@')) => {self.exit_until(Root, Some(c))},

            (CommandList, c @ ('a'..='z' | 'A'..='Z' | '0'..='9')) => self.enter(Command, Some(c)),
            (Command, c @ ('a'..='z' | 'A'..='Z' | '0'..='9')) => self.enter(CommandName, Some(c)),

            (CommandName | CommandArg, c @ (';' | '\n')) => self.exit_until(CommandList, None),
            (CommandName, ' ') => { self.exit(None); self.enter(CommandArg, None); },

            // support for multiple line command args if put inside of quotes
            (CommandArg, '`') if self.buffer.len() == 0 => self.enter(Block, None),
            (Block, '`') => self.exit_until(CommandList, None),

            // CommandArgs support an escape sequence to input special characters
            (CommandArg | Block, '\\') => self.escape_char()?,

            (CommandName | CommandArg | Block, c) => self.buffer.push(c),

            // Handle the invalid character, mode combinations
            (PathNode | PathExt | Path, c) => return Err(Error::new(&format!("{} is not a valid character", c))),
            status => {
                // for token in self.tree.iter() {
                //     println!("{}", token);
                // }
                // todo!("{:?}", status);
            },
        };

        Ok(())
    }

    pub fn parse(mut self) -> Result<Token, Error> {
        while let Some(c) = self.next() {
            self.step(c)?;
        }

        while self.tree.len() > 1 {
            self.exit(None);
        }

        Ok(self.tree[0].clone())
    }

    fn escape_char(&mut self) -> ParseResult {
        match self.next().unwrap() {
            '\n' => (), 
            '\\' => self.step('\\')?, 
            '#' => self.buffer.push('#'), 
            '`' => self.buffer.push('`'), 
            ';' => self.buffer.push(';'), 
            ':' => self.buffer.push(':'), 
            _ => (),
        };

        Ok(())
    }

    fn mode(&self) -> TokenKind {
        return (*self.tree.last().unwrap()).kind
    }

    fn enter(&mut self, mode: TokenKind, c: Option<char>) {
        self.buffer = String::new();
        self.tree.push(Token::new(mode));

        if let Some(c) = c { self.step(c).unwrap(); };
    }

    fn exit(&mut self, c: Option<char>) {
        
        let mut last_token = self.tree.pop().unwrap();
        last_token.set_value(&self.buffer);

        let parent = self.tree.last_mut().unwrap();
        parent.add_child(last_token);

        self.buffer = String::new();

        if let Some(c) = c { self.step(c).unwrap(); }

    }

    fn exit_until(&mut self, mode: TokenKind, c: Option<char>) {
        while self.tree.last().unwrap().kind != mode {
            self.exit(None);
        }

        if let Some(c) = c { self.step(c).unwrap() }
    }

    fn delimit(&mut self) {
        let mode = self.mode();
        self.exit(None);
        self.enter(mode, None);
    }

    fn is_whitespace(&self, character: char) -> bool {
        match character {
            '\t' | ' ' => true,
            _ => false,
        } 
    }

}

impl<R: Read> Iterator for Parser<R>{
    type Item = char;

    fn next(&mut self) -> Option<char> {
        let mut buffer = [0u8;1];
        match self.reader.read(&mut buffer) {
            Ok(1) => return Some(buffer[0] as char),
            _ => None,
        }
    }
}
