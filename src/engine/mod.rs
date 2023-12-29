pub mod template;
pub mod engine;
pub mod table_parser;

use template::Template;

use std::fmt::Display;
use std::convert::TryFrom;
use std::sync::Arc;
use std::pin::Pin;
use std::fmt::Debug;
use std::collections::HashMap;

use hyper::{Response, Body};

use crate::parser::AstNode;

pub trait ServFunctionT: Send + Sync + Debug {
    fn call(&self, input: Value) -> Value;
}

#[derive(Debug)]
pub struct ServFunction(Pin<Arc<ServFunctionT>>);

#[derive(Debug)]
pub enum Expression {
    None,
    Text(String),
    File { path: Box<Expression> },
    Table { input: Box<Expression> },
    Template {
        template: Template,
        arg: Box<Expression>,
    },
}

impl<'rt> Expression {
    pub fn from_node(input: AstNode) -> Result<Self, ()> {
        let AstNode::Expression(parts) = input else { return Err(()) };
        let mut iter = parts.into_iter();
        Self::from_list(iter)
    }

    fn from_list<I: Iterator<Item = AstNode>>(mut input: I) -> Result<Self, ()> {
        let Some(next) = input.next() else { return Ok(Self::None) };
        let output = match next {
            AstNode::Template(_) => {
                Self::Template {
                    template: Template::from_ast(next)?,
                    arg: Box::new(Self::from_list(input)?),
                }
            },
            AstNode::Text(t) => Self::Text(t),
            AstNode::Function(ref name) if name == "file" => {
                Self::File { path: Box::new(Expression::from_list(input)?) }
            },
            AstNode::Function(ref name) if name == "table" => {
                Self::Table { input: Box::new(Expression::from_list(input)?) }
            },
            _ => todo!(),
        };
        Ok(output)
    }

    pub fn eval(&self) -> Value {
        match self {
            Self::None => Value::None,
            Self::Text(t) => Value::Text(t.clone()),
            Self::Template{ template, arg } => {
                let argvalue: Value = arg.eval();
                template.eval(argvalue)
            },
            Self::File { path: e } => {
                let Value::Text(t) = e.eval() else { return Value::None };
                let Ok(contents) = std::fs::read_to_string(t.trim()) else { return Value::None };
                contents.into()
            },
            Self::Table { input: e } => {
                let Value::Text(ref t) = e.eval() else { return Value::None };
                let parsed = table_parser::parse_table(t) else { return Value::None };
                parsed
            },
            _ => todo!(),
            // Self::
        }
    }
}

#[derive(Debug)]
pub enum Value {
    None,
    Text(String),
    Table(HashMap<String, Value>),
}

impl Value {
    pub fn get(&self, input: &str) -> &str {
        match self {
            Self::None => "",
            Self::Text(ref s) => s,
            Self::Table(t) => t.get(input.into()).unwrap().get(""),
        }
    }

    pub fn parse_table(input: &str) -> Self {
        todo!();
    }
}

impl From<&str> for Value {
    fn from(input: &str) -> Self { Self::Text(input.to_owned()) }
}

impl From<String> for Value {
    fn from(input: String) -> Self { Self::Text(input) }
}

impl Into<Response<hyper::Body>> for Value {
    fn into(self) -> Response<Body> {
        let mut output = hyper::Response::builder().status(200);
        match self {
            Value::Text(t) => {
                output.body(t.into()).unwrap()
            },
            Value::Table(t) => {
                let debug = format!("{:#?}", t);
                output.body(debug.into()).unwrap()
            },

            _ => todo!(),
        }
    }
}
