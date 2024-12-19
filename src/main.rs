#![allow(unused)]

mod servlexer;
mod servparser;
mod module;

mod error;
mod functions;
mod template;
mod value;
mod dictionary;
mod webserver;

use module::ServModule;
use error::ServError;
use value::{ ServValue, ServFn };
use dictionary::Label;
use clap::Parser;
use matchit::Router;
use dictionary::StackDictionary;

type ServResult = Result<ServValue, &'static str>;
type Stack<'a> = StackDictionary<'a, ServValue, ()>;


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

    let root_module = servparser::parse_root_from_text(&input).unwrap();
    let mut scope = Stack::empty();

	functions::bind_standard_library(&mut scope);

    for (label, expression) in root_module.definitions {
        scope.insert(label, ServValue::Func(ServFn::Expr(expression, false)));
    }

    for mut expr in root_module.statements {
        expr.eval(&mut scope).expect("failed expression");
    }

    let mut router = Router::new();

    for (route, expr) in root_module.routes {
    	router.insert(route, ServValue::Func(ServFn::Expr(expr, false)));
    }

    webserver::run_webserver(scope, router).await;
}
