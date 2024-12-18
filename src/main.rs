#![allow(unused)]

mod servlexer;
mod servparser;
mod module;

mod tape;
mod error;
mod functions;
mod template;
// mod ast;
mod value;
mod dictionary;
mod webserver;

use module::ServModule;

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

// fn compile(input: ast::Expression, scope: &mut Stack) -> ServValue {
//     todo!();
    // let mut output: VecDeque<ServValue> = VecDeque::new();
    // let mut iter = input.0.into_iter();
    // while let Some(word) = iter.next() {
    //     match word {
    //         ast::Word::Function(t) => output.push_back(ServValue::Ref(Label::Name(t))),
    //         ast::Word::Literal(v) => output.push_back(v),
    //         ast::Word::Template(t) => {
    //             let template = ServFn::Template(t);
    //             let label = scope.insert_anonymous(ServValue::Func(template));
    //             output.push_back(ServValue::Ref(label));
    //         },
    //         ast::Word::Parantheses(e) => {
    //             // let inner = if meta { compile_meta(words, scope) } else { compile(words, scope) };
    //             let inner = compile(e, scope);
    //             output.push_back(inner);
    //         },

    //         otherwise => panic!(),
    //     };
    // }

    // let func = ServFn::Expr(output, input.1);
    // ServValue::Func(func)
// }

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

fn print(input: ServValue, scope: &Stack) -> ServResult {
    println!("{}", input);
    Ok(input)
}

#[tokio::main]
async fn main() {
    let mut args = CliArgs::parse();
    let input = get_input(&mut args).unwrap();


    let root_module = servparser::parse_root_from_text(&input).unwrap();
    let mut scope = Stack::empty();

	functions::bind_standard_library(&mut scope);
    scope.insert(Label::name("print"), ServValue::Func(ServFn::Core(print)));

    for (label, expression) in root_module.definitions {
        scope.insert(label, ServValue::Func(ServFn::Expr(expression, false)));
    }

    for mut expr in root_module.statements {
        expr.eval(&mut scope);
    }

    let mut router = Router::new();

    for (route, expr) in root_module.routes {
    	router.insert(route, ServValue::Func(ServFn::Expr(expr, false)));
    }

    webserver::run_webserver(scope, router).await;
}
