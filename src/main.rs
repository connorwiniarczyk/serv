#![allow(unused)]

mod tape;
mod error;
mod functions;
mod lexer;
mod parser;
mod template;
mod ast;
mod value;
mod dictionary;
mod webserver;

use lexer::TokenKind;
use lexer::*;
use value::ServValue;
use template::Template;
use functions::*;
use dictionary::FnLabel;

use std::collections::VecDeque;
use std::iter::Peekable;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use matchit::Router;

use tape::Words;

type ServResult = Result<ServValue, &'static str>;
pub type Scope<'a> = dictionary::StackDictionary<'a, ServFunction>;

impl<'a> Scope<'a> {
    pub fn get_str(&self, input: &str) -> Result<ServFunction, &'static str> {
        self.get(&FnLabel::name(input)).ok_or("word not found")
    }
}


#[derive(Clone)]
pub enum ServFunction {
    Literal(ServValue),
    Core(fn(ServValue, &Scope) -> ServResult),
    Meta(fn(&mut Words, ServValue, &Scope) -> ServResult),
    Template(Template),
    List(Vec<FnLabel>),
    Composition(Vec<FnLabel>),
}

impl ServFunction {
    pub fn call(&self, input: ServValue, scope: &Scope) -> ServResult {
        match self {
            Self::Core(f)        => f(input, scope),
            Self::Literal(l)     => Ok(l.clone()),
            Self::Template(t)    => {
                let mut child = scope.make_child();
                child.insert_name("in", ServFunction::Literal(input));
                t.render(&child)
            },
            Self::Composition(v) => {
                let mut child_scope = scope.make_child();
                child_scope.insert_name("in", ServFunction::Literal(input.clone()));

                let mut words: VecDeque<FnLabel> = v.clone().into();
                Words(words).eval(input, &child_scope)
            },
            Self::List(l) => {
                let mut list: VecDeque<ServValue> = VecDeque::new();
                for f in l {
                    list.push_back(scope.get(f).unwrap().call(input.clone(), scope)?);
                }
                Ok(ServValue::List(list))
            }
            Self::Meta(_) => Err("called a meta function when it was not appropriate"),
        }
    }
}

fn compile(input: Vec<ast::Word>, scope: &mut Scope) -> ServFunction {
    let mut output: Vec<FnLabel> = Vec::new();
    let mut iter = input.into_iter();
    while let Some(word) = iter.next() {
        match word {
            ast::Word::Function(t) => output.push(FnLabel::Name(t.contents)),
            ast::Word::List(l) => {
                let mut inner: Vec<FnLabel> = Vec::new();
                for expression in l {
					let func = compile(expression.0, scope);
                    inner.push(scope.insert_anonymous(func));
                }
                output.push(scope.insert_anonymous(ServFunction::List(inner)));
            },
            ast::Word::Template(t) => {
                output.push(scope.insert_anonymous(ServFunction::Template(t)));
            },
            ast::Word::Parantheses(expression) => {
                let func = compile(expression.0, scope);
                output.push(scope.insert_anonymous(func));
            },
            ast::Word::Literal(v) => {
                output.push(scope.insert_anonymous(ServFunction::Literal(v)));
            },
        };
    }


    ServFunction::Composition(output)
}


#[tokio::main]
async fn main() {
	let input_path = std::env::args().nth(1).unwrap_or("src/test.serv".to_string());
	let input      = std::fs::read_to_string(&input_path).unwrap();
	let ast        = parser::parse_root_from_text(&input).unwrap();

	let mut scope: Scope = Scope::empty();
	crate::functions::bind_standard_library(&mut scope);

	for declaration in ast.0 {
    	if declaration.kind == "word" {
        	let func = compile(declaration.value.0, &mut scope);
        	scope.insert(declaration.key.to_owned().into(), func);
    	}

    	else if declaration.kind == "route" {
        	let func = compile(declaration.value.0, &mut scope);
        	scope.router.as_mut().unwrap().insert(declaration.key, func);
    	}
	}

	if let Ok(out) = scope.get_str("out") {
    	let res = out.call(ServValue::None, &scope);
    	println!("{}", res.unwrap());
	}

	println!("starting web server");
	webserver::run_webserver(scope).await;
}
