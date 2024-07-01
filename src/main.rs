#![allow(unused)]

mod functions;
mod lexer;
mod parser;
mod template;
mod ast;
mod value;
mod dictionary;

use lexer::TokenKind;
use lexer::*;
use value::ServValue;
use template::Template;
use functions::*;

use std::collections::VecDeque;
use std::iter::Peekable;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use matchit::Router;

type ServResult = Result<ServValue, &'static str>;
pub type Scope<'a> = dictionary::StackDictionary<'a, FnLabel, ServFunction>;

impl<'a> Scope<'a> {
    pub fn get_str(&self, input: &str) -> Result<ServFunction, &'static str> {
        self.get(&FnLabel(input.to_string())).ok_or("word not found")
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct FnLabel(String);

impl From<String> for FnLabel {
    fn from(input: String) -> Self {
        Self(input)
    }
}

struct Words(VecDeque<FnLabel>);

impl Words {
    pub fn next(&mut self) -> Option<FnLabel> {
        self.0.pop_front()
    }

    pub fn empty() -> Self {
        Self(VecDeque::new())
    }

    pub fn eval(&mut self, input: ServValue, scope: &Scope) -> ServResult {
        let Some(next) = self.next() else { return Ok(input) };
        let Some(next_fn) = scope.get(&next) else { panic!("word not found: {:?}", next)};

        if let ServFunction::Meta(m) = next_fn {
			return (m)(self, input, scope);
        } else {
            let rest = self.eval(input, scope)?;
            return next_fn.call(rest, scope)
        }
    }
}


#[derive(Clone)]
pub enum ServFunction {
    Literal(ServValue),
    Core(fn(ServValue, &Scope) -> ServResult),
    Meta(fn(&mut Words, ServValue, &Scope) -> ServResult),
    Template(Template),
    Composition(Vec<FnLabel>),
}

impl ServFunction {
    pub fn call(&self, input: ServValue, scope: &Scope) -> ServResult {
        match self {
            Self::Core(f)        => f(input, scope),
            Self::Literal(l)     => Ok(l.clone()),
            Self::Template(t)    => {
                let mut child = scope.make_child();
                child.insert(FnLabel("in".to_owned()), ServFunction::Literal(input));
                t.render(&child)
            },
            Self::Composition(v) => {
                let mut child_scope = scope.make_child();
                child_scope.insert(FnLabel("in".to_owned()), ServFunction::Literal(input.clone()));

                let mut words: VecDeque<FnLabel> = v.clone().into();
                Words(words).eval(input, &child_scope)
            },
            Self::Meta(_) => Err("called a meta function when it was not appropriate"),
        }
    }
}

fn compile(input: Vec<ast::Word>, scope: &mut Scope) -> ServFunction {
    let mut output: Vec<FnLabel> = Vec::new();
    let mut iter = input.into_iter();
    while let Some(word) = iter.next() {
        match word {
            ast::Word::Function(t) => output.push(FnLabel(t.contents.to_owned())),
            ast::Word::Template(t) => {
                let unique_id = scope.get_unique_id();
                let label = format!("template.{}", unique_id);
                scope.insert(FnLabel(label.clone()), ServFunction::Template(t));
                output.push(FnLabel(label));
            },
            ast::Word::Parantheses(expression) => {
                let unique_id = scope.get_unique_id();
                let label = format!("lambda.{}", unique_id);
                let inner_func = compile(expression.0, scope);
                scope.insert(FnLabel(label.clone()), inner_func);
                output.push(FnLabel(label));
            },
            ast::Word::Literal(v) => {
                let unique_id = scope.get_unique_id();
                let label = format!("literal.{}", unique_id);
                scope.insert(FnLabel(label.clone()), ServFunction::Literal(v));
                output.push(FnLabel(label));
            },
        };
    }

    ServFunction::Composition(output)
}

#[tokio::main]
async fn main() {
	let input_path = std::env::args().nth(1).unwrap_or("src/test.serv".to_string());
	let input = std::fs::read_to_string(&input_path).unwrap();

	let ast = parser::parse_root_from_text(&input).unwrap();

	let mut scope: Scope = Scope::empty();

	scope.insert(FnLabel("hello".to_owned()),     ServFunction::Core(hello_world));
	scope.insert(FnLabel("uppercase".to_owned()), ServFunction::Core(uppercase));
	scope.insert(FnLabel("incr".to_owned()), ServFunction::Core(incr));
	scope.insert(FnLabel("decr".to_owned()), ServFunction::Core(decr));
	scope.insert(FnLabel("%".to_owned()), ServFunction::Core(math_expr));

	scope.insert(FnLabel("!".to_owned()),         ServFunction::Meta(drop));
	scope.insert(FnLabel("map".to_owned()),         ServFunction::Meta(map));
	scope.insert(FnLabel("[".to_owned()),         ServFunction::Meta(list_open));
	scope.insert(FnLabel("using".to_owned()),         ServFunction::Meta(using));
	scope.insert(FnLabel("let".to_owned()),         ServFunction::Meta(using));
	scope.insert(FnLabel("choose".to_owned()),         ServFunction::Meta(choose));
	scope.insert(FnLabel("*".to_owned()),         ServFunction::Meta(apply));


	for declaration in ast.0 {
    	if declaration.kind == "word" {
        	let func = compile(declaration.value.0, &mut scope);
        	scope.insert(declaration.key.to_owned().into(), func);
    	}
	}

	if let Some(out) = scope.get(&FnLabel("out".to_owned())) {
    	let res = out.call(ServValue::None, &scope);
    	println!("{}", res.unwrap());
	}
}
