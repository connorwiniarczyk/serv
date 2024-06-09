#![allow(unused)]

mod lexer;
mod parser;
mod template;
mod ast;
mod value;

mod dictionary;

use lexer::TokenKind;
use lexer::*;

use template::Template;

use std::collections::VecDeque;
use std::iter::Peekable;

use tokio::net::TcpListener;
// use hyper::server::conn;
// use hyper_util::rt::TokioIo;
// use hyper::service::Service;
// use hyper::body::Incoming as IncomingBody;
// use hyper::{ Request, Response };
// use std::scope::Future;
// use std::pin::Pin;

use std::net::SocketAddr;
use matchit::Router;

type ServResult = Result<ServValue, &'static str>;
pub type Scope<'a> = dictionary::StackDictionary<'a, FnLabel, ServFunction>;
use value::ServValue;

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct FnLabel(String);

impl From<String> for FnLabel {
    fn from(input: String) -> Self {
        Self(input)
    }
}

fn hello_world(input: ServValue, scope: &Scope) -> ServResult {
    Ok(ServValue::Text("hello world".to_owned()))
}

fn uppercase(input: ServValue, scope: &Scope) -> ServResult {
    if let ServValue::Text(t) = input {
        Ok(ServValue::Text(t.to_uppercase()))
    } else {
        Err("wrong value")
    }
}

struct Words(VecDeque<FnLabel>);

impl Words {
    pub fn eval(&mut self, input: ServValue, scope: &Scope) -> ServResult {
        let Some(next) = self.0.pop_front() else { return Ok(input) };
        let rest = self.eval(input, scope)?;
        scope.get(&next).unwrap().call(rest, scope)
    }
}


#[derive(Clone)]
pub enum ServFunction {
    Core(fn(ServValue, &Scope) -> ServResult),
    Template(Template),
    Composition(Vec<FnLabel>),
}

impl ServFunction {
    pub fn call(&self, input: ServValue, scope: &Scope) -> ServResult {
        match self {
            Self::Core(f)        => f(input, scope),
            Self::Template(t)    => t.render(scope),
            Self::Composition(v) => {
                let mut words: VecDeque<FnLabel> = v.clone().into();
                Words(words).eval(input, scope)
            },
        }
    }
}


fn compile(input: Vec<ast::Word>, scope: &mut Scope) -> ServFunction {
    let mut output: Vec<FnLabel> = Vec::new();
    for word in input.into_iter() {
        let next = match word {
            ast::Word::Function(t) => FnLabel(t.contents.to_owned()),
            ast::Word::Template(t) => {
                scope.insert(FnLabel("template.01".to_owned()), ServFunction::Template(t));
                FnLabel("template.01".to_owned())
            },
            _ => todo!(),
        };

        output.push(next);
    }

    ServFunction::Composition(output)
}


#[tokio::main]
async fn main() {
	let input_path = std::env::args().nth(1).unwrap_or("src/test.serv".to_string());
	let input = std::fs::read_to_string(&input_path).unwrap();

	let ast = parser::parse_root_from_text(&input).unwrap();

	// println!("{:#?}", ast);

	let mut scope: Scope = Scope::empty();

	scope.insert(FnLabel("hello".to_owned()), ServFunction::Core(hello_world));
	scope.insert(FnLabel("uppercase".to_owned()), ServFunction::Core(uppercase));

	for declaration in ast.0 {
    	if declaration.kind == "word" {
        	let func = compile(declaration.value.0, &mut scope);
        	scope.insert(declaration.key.to_owned().into(), func);
    	}
	}

	if let Some(out) = scope.get(&FnLabel("out".to_owned())) {
    	let res = out.call(ServValue::None, &scope);
    	println!("{:?}", res);
	}

	return;
}
