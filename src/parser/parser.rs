use std::io::Read;

use super::token::{Token, TokenKind};
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
            (context, '#') if context != Comment && context != MultiLine => self.enter(Comment, None),
            (Comment, '\n') => self.exit(None),
            (Comment, _) => (),

            (Root | Route | CommandList | Command | Path, c) if self.is_whitespace(c) => (),
            (Root | CommandList | Route, '\n') => (),

            (Root,  c @ ('/' | '@' | '.')) => self.enter(Route, Some(c)),
            (Route, c @ ('/' | '@' | '.')) => self.enter(Path, Some(c)),
            (Route, ':') => self.enter(CommandList, None), 

            (Path, ':') => self.exit(Some(':')), 
            (Path, '/') => self.enter(PathNode, None), 
            (Path, '@') => self.enter(PathAttribute, None), 
            (Path, '.') => self.enter(PathExt, None), 

            (PathNode | PathExt | PathAttribute, c @ ('a'..='z' | 'A'..='Z' | '0'..='9' | '*')) => self.buffer.push(c),
            (PathNode | PathExt | PathAttribute, c @ ('.' | '@' | '/' | ':')) => self.exit(Some(c)),
            (PathNode | PathExt | Path, c) if self.is_whitespace(c) => self.exit(Some(c)),
            (PathNode | PathExt | Path, c) => return Err(Error::new(&format!("{} is not a valid character", c))),

            (CommandList, c @ ('/' | '@')) => {self.exit(None); self.exit(Some(c))},
            (CommandList, c @ ('a'..='z' | 'A'..='Z' | '0'..='9')) => self.enter(Command, Some(c)),

            (Command, c @ ('a'..='z' | 'A'..='Z' | '0'..='9')) => self.enter(CommandName, Some(c)),
            (Command, '\n') => self.exit(Some('\n')),
            (Command, ';') => self.delimit(),

            (CommandName | CommandArg, c @ (';' | '\n')) => self.exit(Some(c)),
            (CommandName, ' ') => { self.exit(None); self.enter(CommandArg, None); },

            // support for multiple line command args if put inside of quotes
            (CommandArg, '`') if self.buffer.len() == 0 => self.enter(MultiLine, None),
            (MultiLine, '`') => { self.exit(None); self.exit(None) },

            // CommandArgs support an escape sequence to input special characters
            (CommandArg | MultiLine, '\\') => self.escape_char()?,

            (CommandName | CommandArg | MultiLine, c) => self.buffer.push(c),

            status => todo!("{:?}", status),
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
