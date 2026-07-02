#![allow(unused)]

mod parser;
mod engine;
mod error;
mod functions;
mod webserver;

pub use engine::datatypes;

use engine::datatypes::*;
use engine::datatypes::module::ServModule;
use engine::value::{ ServValue, ServFn };
use engine::dictionary::StackDictionary;
use engine::dictionary::Stack;

use datatypes::reference::Address;
use error::ServError;
use engine::dictionary::Label;
use clap::Parser;
use matchit::Router;

type ServResult = Result<ServValue, ServError>;

/// A parser for serv files
#[derive(Parser, Debug)]
#[command(version, about, long_about)]
struct CliArgs {

	/// The tcp port to listen on
    #[arg(short, long, default_value_t = 4000)]
    port: u16,

	/// The address to listen on
    #[arg(long)]
    host: Option<String>,

	/// Pass serv code directly as an argument, rather than specifying a file
	#[arg(short, long)]
	execute: Vec<String>,

	/// The files to parse
    path: Vec<String>,
}

fn get_input(args: &mut CliArgs) -> Result<String, ServError> {
    let mut output = String::new();
    for line in args.execute.iter() {
        output.push_str(line.as_str());
        output.push_str("\n");
    }

    if args.path.len() == 0 && args.execute.len() == 0 {
        args.path.push("main.serv".into());
    }

    for p in args.path.iter() {
        let Ok(file_contents) = std::fs::read_to_string(p) else {
            return Err("could not open file".into());
        };
		output.push_str(&file_contents);
    }

    return Ok(output)
}

fn populate_defaults(scope: &mut Stack, args: &CliArgs) {
    if engine::resolve_key("server.port", scope).is_err() {
        let port: i64 = args.port.into();
        scope.insert("server.port", port.into());
    }
}

#[tokio::main]
async fn main() {
    let mut args = CliArgs::parse();
    let input = get_input(&mut args).unwrap();

    let mut scope = Stack::empty();
    scope.insert_module(functions::standard_library().values);

    let root_module = parser::parse_root_from_text(&input, &mut scope).unwrap();
    scope.insert_module(root_module.values.clone());

    populate_defaults(&mut scope, &args);

    for expr in &root_module.statements {
        engine::eval(expr.clone(), &mut scope).unwrap();
    }

    let mut router = Router::new();
    for (route, list) in root_module.routes() {
        router.insert(route, list.clone());
    }

    webserver::run_webserver(scope, router).await;
}
