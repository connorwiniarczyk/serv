use crate::value::ServList;

use parsetool::cursor::{ Tokenizer, Token };
use crate::{Stack, ServResult, Label, ServValue, ServFn };

use std::iter::Peekable;
struct Parser<I>(Peekable<I>) where I: Iterator<Item = JsonToken>;

use std::collections::VecDeque;
use std::collections::HashMap;

use crate::value::Serializer;
use crate::ServError;

type Buffer<'b> = &'b mut (dyn std::fmt::Write + 'b);

#[derive(Clone)]
pub struct JsonSerializer<'scope> {
    tab: &'static str,
    indent: isize,
    scope: &'scope Stack<'scope>,
}

pub fn serializer<'scope>(scope: &'scope Stack<'scope>) -> JsonSerializer<'scope> {
    JsonSerializer::new(scope)
}

impl<'a> JsonSerializer<'a> {
    pub fn new(scope: &'a Stack<'a>) -> Self {
        Self {
            indent: 0,
            tab: "  ",
            scope
        }
    }

    fn line_break<'b>(&self, dest: Buffer<'b>) {
		dest.write_char('\n');
		for _ in 0..self.indent { dest.write_str(self.tab); }
    }
}

impl<'a> Serializer for JsonSerializer<'a> {
    fn write<'b>(&mut self, value: ServValue, dest: Buffer<'b>) -> Result<(), ServError> {
        match value {
			ServValue::Ref(label) => self.write(self.scope.get(label)?, dest)?,
			f @ ServValue::Func(_) => self.write(f.call(None, self.scope)?, dest)?,

			// ServValue::Module(t)   => todo!("json serialize modules"),
			ServValue::Module(t)   => dest.write_str("module")?,
			ServValue::None     => dest.write_str("0")?,
			ServValue::Bool(b)  => dest.write_str(if b {"true"} else {"false"})?,
			ServValue::Float(v) => dest.write_str(&v.to_string())?,
			ServValue::Int(v)   => dest.write_str(&v.to_string())?,
			ServValue::Text(t)  => {
    			dest.write_str("\"");
    			dest.write_str(t.as_str().unwrap_or("RAW"));
    			dest.write_str("\"")?
			},

			ServValue::List(list) => {
    			dest.write_str("[");
    			self.indent += 1;
    			self.line_break(dest);

    			let mut iter = list.peekable();
    			while let Some(value) = iter.next() {
        			self.write(value, dest)?;
        			if iter.peek().is_some() {
            			dest.write_char(',');
            			self.line_break(dest);
        			}
    			}

    			self.indent -= 1;
    			self.line_break(dest);
    			dest.write_str("]")?

			},

			ServValue::Table(table) => {
    			dest.write_str("{");
    			self.indent += 1;
    			self.line_break(dest);

    			let mut iter = table.into_iter().peekable();
    			while let Some((key, value)) = iter.next() {
        			dest.write_str("\"");
        			dest.write_str(&key);
        			dest.write_str("\"");
        			dest.write_str(": ");
        			self.write(value, dest)?;
        			if iter.peek().is_some() {
            			dest.write_str(",");
            			self.line_break(dest);
        			}
    			}
    			self.indent -= 1;
    			self.line_break(dest);
    			dest.write_str("}")?
			},
        };

        Ok(())
    }
}

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

fn tokenize_json<'input>(cursor: &mut Tokenizer<'input>) -> Vec<JsonToken> {
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

    		c @ _ if c.is_numeric() || c == '-' => {
        		if c == '-' { cursor.incr(1)};
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


impl<I> Parser<I> where I: Iterator<Item = JsonToken> {
    fn new(input: I) -> Self {
        Self(input.peekable())
    }

    fn parse_value(&mut self) -> Result<ServValue, ()> {
        let valid = [Text, Number, OpenObject, OpenList];
        let token = self.0.next_if(|t| valid.contains(&t.kind)).ok_or(())?;
        let output = match token.kind {
            Text       => ServValue::Text(token.value.into()),
            Number     => ServValue::Float(token.value.parse().unwrap()),
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
        let mut output = ServList::new();
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
    let mut cursor = Tokenizer::new(&chars);
    let tokens = tokenize_json(&mut cursor);
    let mut parser = Parser::new(tokens.into_iter());

    parser.parse_value().unwrap()
}

fn json_from(input: ServValue, scope: &Stack) -> ServResult {
    // println!("{:?}", input.clone().to_string());
    Ok(parse_json_from_str(&input.to_string()))
}

use crate::ServModule;

pub fn get_module() -> ServModule {
    let mut output = ServModule::empty();
	output.insert("json",      ServFn::Core(json_from).into());
	output.insert("json.from", ServFn::Core(json_from).into());

	output
}
