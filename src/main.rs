#![allow(unused)]

mod datatypes;
use datatypes::*;

mod servlexer;
mod servparser;
mod engine;

mod error;
mod functions;
mod dictionary;
mod webserver;

use datatypes::module::ServModule;
use value::{ ServValue, ServFn };
use datatypes::reference::Address;

use error::ServError;
use dictionary::Label;
use clap::Parser;
use matchit::Router;
use dictionary::StackDictionary;

type ServResult = Result<ServValue, ServError>;

type Stack<'a> = StackDictionary<'a, ServValue>;

// impl Stack<'_> {
//     fn search(&self, input: Label) -> Result<ServValue, ServError> {
//         let Label::Name(text) = input else {
//             return self.get(input);
//         };

//         let fields: Vec<Label> = text.split(".").map(|x| Label::Name(x.to_owned())).collect();
//         let mut iter = fields.iter();

//         let first = iter.next().ok_or(ServError::new(500, "missing"))?;
// 		let mut value = self.get(first.clone())?;

// 		value.get_member(&mut iter, self).ok_or(ServError::new(500, "missing"))
//     }
// }


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

fn get_input(args: &mut CliArgs) -> Result<String, ServError> {
    if let Some(output) = args.code.take() { return Ok(output) }

    let path = args.path.take().unwrap_or("main.serv".into());
    std::fs::read_to_string(&path).map_err(|e| "could not open file".into())
}

fn populate_defaults(scope: &mut Stack, args: &CliArgs) {
    if scope.get("serv.port").is_err() {
        let port: i64 = args.port.into();
        scope.insert("serv.port", port.into());
    }
}

#[tokio::main]
async fn main() {
    let mut args = CliArgs::parse();
    let input = get_input(&mut args).unwrap();
    let root_module = servparser::parse_root_from_text(&input).unwrap();
    let mut scope = Stack::empty();

    scope.insert_module(functions::standard_library().values);
    scope.insert_module(root_module.values.clone());

    populate_defaults(&mut scope, &args);

    for expr in &root_module.statements {
        engine::resolve(expr.clone().as_expr(), None, &scope);
        // expr.clone().eval(&mut scope).expect("failed expression");
    }

    let mut router = Router::new();
    for (route, list) in root_module.routes() {
        router.insert(route, list.clone());
    }

    webserver::run_webserver(scope, router).await;
}
