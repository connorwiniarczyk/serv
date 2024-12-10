#![allow(unused)]

mod servlexer;
mod servparser;

mod tape;
mod error;
mod functions;
mod template;
mod ast;
mod value;
mod dictionary;
mod webserver;

use crate::error::ServError;

use servlexer::TokenKind;

use value::ServValue;
use template::Template;
use functions::*;
use dictionary::Label;

use clap::Parser;

use std::collections::VecDeque;
use std::iter::Peekable;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use matchit::Router;

type ServResult = Result<ServValue, &'static str>;

use dictionary::StackDictionary;

type Stack<'a> = StackDictionary<'a, ServValue, ()>;

use crate::value::ServFn;

// fn compile_meta(input: Vec<ast::Word>, scope: &mut Stack) -> ServValue {
//     let ServValue::FnLiteral(ServFn::Expr(e)) = compile(input, scope) else { panic!() };
//     ServValue::FnLiteral(ServFn::ExprMeta(e))
// }

fn compile(input: ast::Expression, scope: &mut Stack) -> ServValue {
    let mut output: VecDeque<ServValue> = VecDeque::new();
    let mut iter = input.0.into_iter();
    while let Some(word) = iter.next() {
        match word {
            ast::Word::Function(t) => output.push_back(ServValue::Ref(Label::Name(t))),
            ast::Word::Literal(v) => output.push_back(v),
            ast::Word::Template(t) => {
                let template = ServFn::Template(t);
                let label = scope.insert_anonymous(ServValue::Func(template));
                output.push_back(ServValue::Ref(label));
            },
            ast::Word::Parantheses(e) => {
                // let inner = if meta { compile_meta(words, scope) } else { compile(words, scope) };
                let inner = compile(e, scope);
                output.push_back(inner);
            },

            otherwise => panic!(),
        };
    }

    let func = ServFn::Expr(output, input.1);
    ServValue::Func(func)
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

fn ast_bind_to_scope(ast: ast::AstRoot, scope: &mut Stack) {
	for declaration in ast.0 {
    	if declaration.kind == "word" {
        	let func = compile(declaration.value, scope);
        	scope.insert(declaration.key.to_owned().into(), func);
    	}

    	else if declaration.kind == "route" {
        	let func = compile(declaration.value, scope);
        	scope.router.as_mut().unwrap().insert(declaration.key, func);
    	}

    	else if declaration.kind == "include" {
        	let func = compile(declaration.value, scope);
        	let value = func.call(None, &scope).expect("include function failed").to_string();
        	let ast = servparser::parse_root_from_text(&value).expect("include string failed to parse");
			ast_bind_to_scope(ast, scope);
    	}
	}
}

#[tokio::main]
async fn main() {
    let mut args = CliArgs::parse();
    let input = get_input(&mut args).unwrap();

	if args.execute {
    	let ast = servparser::parse_expression_from_text(&input).unwrap();
    	let mut scope = Stack::empty();
    	crate::functions::bind_standard_library(&mut scope);

    	let func = compile(ast, &mut scope);
    	let output = func.call(None, &scope).expect("error");

	} else {
    	let ast = servparser::parse_root_from_text(&input).unwrap();
    	let mut scope = Stack::empty();
    	crate::functions::bind_standard_library(&mut scope);
    	ast_bind_to_scope(ast, &mut scope);

    	if let Some(out) = scope.get("out") {
        	let res = out.call(None, &scope);
        	println!("{}", res.unwrap());
    	}

    	// println!("starting web server");
    	webserver::run_webserver(scope).await;
	}
}
