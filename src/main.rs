#![allow(unused)]

mod serv_tokenizer;
mod tape;
mod error;
mod functions;
mod cursor;
mod parser;
mod template;
mod ast;
mod value;
mod dictionary;
mod webserver;

use crate::error::ServError;

use serv_tokenizer::TokenKind;
use value::ServValue;
use template::Template;
use functions::*;
use dictionary::FnLabel;

use clap::Parser;

use std::collections::VecDeque;
use std::iter::Peekable;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use matchit::Router;

// use tape::Words;

type ServResult = Result<ServValue, &'static str>;
// pub type Scope<'a> = dictionary::StackDictionary<'a, ServFunction>;

// impl<'a> Scope<'a> {
//     pub fn get_str(&self, input: &str) -> Result<ServFunction, &'static str> {
//         self.get(&FnLabel::name(input)).ok_or("word not found")
//     }
// }


// #[derive(Clone)]
// pub enum ServFunction {
//     Literal(ServValue),
//     Core(fn(ServValue, &Scope) -> ServResult),
//     Meta(fn(ServValue, &Scope) -> ServResult),
//     // Meta(fn(&mut Words, ServValue, &Scope) -> ServResult),
//     Template(Template),
//     List(Vec<FnLabel>),
//     Composition(Vec<FnLabel>),
// }

// impl ServFunction {
//     pub fn is_meta(&self) -> bool {
//         match (self) {
//             Self::Literal(_)     => false,
//             Self::Core(_)        => false,
//             Self::Meta(_)        => true,
//             Self::Template(_)    => false,
//             Self::List(_)        => false,
//             Self::Composition(_) => false,
//         }
//     }
//     pub fn call(&self, input: ServValue, scope: &Scope) -> ServResult {
//         match self {
//             Self::Core(f)        => f(input, scope),
//             Self::Meta(f)        => f(input, scope),
//             Self::Literal(l)     => Ok(l.clone()),
//             Self::Template(t)    => {
//                 todo!();
//                 // let mut child = scope.make_child();
//                 // child.insert_name("in", ServFunction::Literal(input));
//                 // t.render(&child)
//             },
//             Self::Composition(v) => {
//                 let mut child_scope = scope.make_child();
//                 child_scope.insert_name("in", ServFunction::Literal(input.clone()));

//                 let mut words: VecDeque<FnLabel> = v.clone().into();
//                 Words(words).eval(input, &child_scope)
//             },
//             Self::List(l) => {
//                 let mut list: VecDeque<ServValue> = VecDeque::new();
//                 for f in l {
//                     list.push_back(scope.get(f).unwrap().call(input.clone(), scope)?);
//                 }
//                 Ok(ServValue::List(list))
//             }
//         }
//     }
// }

use dictionary::StackDictionary;

type Stack<'a> = StackDictionary<'a, ServValue>;

impl<'a> Stack<'a> {
    pub fn get_str(&self, input: &str) -> Result<ServValue, &'static str> {
        self.get(&FnLabel::name(input)).ok_or("word not found")
    }
}

fn incr(value: ServValue, stack: &Stack) -> ServResult {
    Ok(ServValue::Int(value.expect_int()? + 1))
}

use crate::value::ServFn;

fn compile(input: Vec<ast::Word>, scope: &mut Stack) -> ServValue {
    let mut output: VecDeque<ServValue> = VecDeque::new();
    let mut iter = input.into_iter();
    while let Some(word) = iter.next() {
        match word {
            ast::Word::Function(t) => output.push_back(ServValue::Ref(FnLabel::Name(t))),
            ast::Word::Literal(v) => output.push_back(v),
            ast::Word::Template(t) => {
                let template = ServFn::Template(t);
                let label = scope.insert_anonymous(ServValue::FnLiteral(template));
                output.push_back(ServValue::Ref(label));
            },
            ast::Word::Parantheses(expression) => {
                let inner = compile(expression.0, scope);
                output.push_back(inner);
            },

            otherwise => panic!(),
        };
    }

    let func = ServFn::Expr(output);
    ServValue::FnLiteral(func)
}

/// A parser for serv files
#[derive(Parser, Debug)]
#[command(version, about, long_about)]
struct CliArgs {

	/// The tcp port to listen on
    #[arg(short, long, default_value_t = 4000)]
    port: u16,

	/// The tcp port to listen on
    #[arg(long)]
    host: Option<String>,

	/// Parse the file as a single expression only and run it immediately
	#[arg(short, long, default_value_t = false)]
    execute: bool,

	/// Pass serv code directly as an argument, rather than specifying a file
	#[arg(short, long)]
	code: Option<String>,

	/// The file to parse
    path: Option<String>,

}

fn get_input(args: &mut CliArgs) -> Result<String, ServError>{
    if let Some(output) = args.code.take() { return Ok(output) }

    let path = args.path.take().unwrap_or("main.serv".into());
    std::fs::read_to_string(&path).map_err(|e| "could not open file".into())
}

#[tokio::main]
async fn main() {
    let mut args = CliArgs::parse();
    let input = get_input(&mut args).unwrap();

	if args.execute {
    	let ast = parser::parse_expression_from_text(&input).unwrap();
    	let mut scope = Stack::empty();
    	crate::functions::bind_standard_library(&mut scope);

    	let func = compile(ast.0, &mut scope);
    	let output = func.eval(None, &scope).expect("error");

    	println!("{}", output);

	} else {
    	let ast = parser::parse_root_from_text(&input).unwrap();
    	let mut scope = Stack::empty();
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

    	if let Some(out) = scope.get(&FnLabel::Name("out".into())) {
        	let res = out.eval(None, &scope);
        	println!("{}", res.unwrap());
    	}

    	// println!("starting web server");
    	// webserver::run_webserver(scope).await;
	}
}
