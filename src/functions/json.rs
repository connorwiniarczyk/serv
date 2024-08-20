use crate::cursor::Tokenizer as Cursor;
use crate::cursor::Token;
use crate::ServValue;

use crate::{Scope, ServResult, FnLabel, ServFunction};

#[derive (PartialEq, Clone, Copy, Debug)]
enum TokenKind {
    OpenObject,
    CloseObject,
    OpenList,
    CloseList,
    Number,
    Text,
    Colon,
    Comma,
    Identifier,
}

use TokenKind::*;

type JsonToken = Token<TokenKind>;

fn tokenize_json<'input>(cursor: &mut Cursor<'input>) -> Vec<JsonToken> {
    let mut output: Vec<JsonToken> = Vec::new();
    while cursor.get(0).is_some() {
		match cursor.get(0).unwrap() {
    		'{' => { cursor.incr(1); output.push(cursor.emit(OpenObject)) },
    		'}' => { cursor.incr(1); output.push(cursor.emit(CloseObject)) },
    		'[' => { cursor.incr(1); output.push(cursor.emit(OpenList)) },
    		']' => { cursor.incr(1); output.push(cursor.emit(CloseList)) },
    		':' => { cursor.incr(1); output.push(cursor.emit(Colon)) },
    		',' => { cursor.incr(1); output.push(cursor.emit(Comma)) },

    		'"' => {
        		cursor.incr(1);
        		_ = cursor.emit(());
        		cursor.incr_while(|x| x != '"');
        		output.push(cursor.emit(Text));
        		if cursor.get(0) != Some('"') { panic!("end of input while parsing string") };
        		cursor.incr(1);
        		_ = cursor.emit(());
    		}

    		c @ _ if c.is_alphabetic() => {
        		cursor.incr_while(|x| x.is_alphanumeric() || x == '_');
        		output.push(cursor.emit(Identifier));
    		}

    		c @ _ if c.is_numeric() => {
        		cursor.incr_while(|x| x.is_numeric() || x == '.');
        		output.push(cursor.emit(Number));
    		}

    		c @ _ if c.is_whitespace() => {
        		cursor.incr_while(|x| x.is_whitespace());
        		_ = cursor.emit(());
    		}

    		c @ _ => panic!("unexpected value {}", c),
		};
    }

    output
}

use std::iter::Peekable;
struct Parser<I>(Peekable<I>) where I: Iterator<Item = JsonToken>;

use crate::VecDeque;
use std::collections::HashMap;

impl<I> Parser<I> where I: Iterator<Item = JsonToken> {
    fn new(input: I) -> Self {
        Self(input.peekable())
    }

    fn parse_value(&mut self) -> Result<ServValue, ()> {
        let valid = [Text, Number, OpenObject, OpenList];
        let token = self.0.next_if(|t| valid.contains(&t.kind)).ok_or(())?;
        let output = match token.kind {
            Text       => ServValue::Text(token.value),
            Number     => ServValue::Int(token.value.parse().unwrap()),
            OpenList   => self.parse_list()?,
            OpenObject => self.parse_object()?,

            _ => unreachable!(),
        };

        Ok(output)
    }

    fn parse_object(&mut self) -> Result<ServValue, ()> {
        let mut output: HashMap<String, ServValue> = HashMap::new();
        let valid_keys = [Text, Identifier, Number];
        while self.0.peek().expect("end of input while parsing object").kind != CloseObject {
            let key = self.0.next_if(|t| valid_keys.contains(&t.kind)).ok_or(())?.value;
            let _colon = self.0.next_if(|t| t.kind == Colon).ok_or(())?;
            output.insert(key, self.parse_value()?);
            let _comma = self.0.next_if(|t| t.kind == Comma);
        }
        let _close = self.0.next().unwrap();
        Ok(ServValue::Table(output))
    }

    fn parse_list(&mut self) -> Result<ServValue, ()> {
        let mut output: VecDeque<ServValue> = VecDeque::new();
        while self.0.peek().expect("end of input while parsing list").kind != CloseList {
            output.push_back(self.parse_value()?);
            let _comma = self.0.next_if(|t| t.kind == Comma);
        }
        let _close = self.0.next().unwrap();
        Ok(ServValue::List(output))
    }
}

fn parse_json_from_str(input: &str) -> ServValue {
    let chars: Vec<char> = input.chars().collect();
    let mut cursor = Cursor::new(&chars);
    let tokens = tokenize_json(&mut cursor);
    let mut parser = Parser::new(tokens.into_iter());

    parser.parse_value().unwrap()
}

fn json_from(input: ServValue, scope: &Scope) -> ServResult {
    Ok(parse_json_from_str(&input.to_string()))
}

pub fn bind(scope: &mut Scope) {
	scope.insert(FnLabel::name("json.from"), ServFunction::Core(json_from));
	scope.insert(FnLabel::name("json"), ServFunction::Core(json_from));
}
