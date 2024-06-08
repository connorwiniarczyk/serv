#![allow(unused)]

// mod functions;
// mod serv_value;
// mod compiler;
mod lexer;
mod engine;
mod parser;
mod template;
mod ast;
mod value;

mod dictionary;

use lexer::TokenKind;
use lexer::*;
use parser::*;

use std::sync::Arc;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::iter::Peekable;

use tokio::net::TcpListener;
use hyper::server::conn;
use hyper_util::rt::TokioIo;

use hyper::service::Service;
use hyper::body::Incoming as IncomingBody;
use hyper::{ Request, Response };
use std::future::Future;
use std::pin::Pin;

use std::net::SocketAddr;
use matchit::Router;

use engine::ServFunction;

pub type Context<'a> = dictionary::Scope
     <'a, String, ServFunction>;
use value::ServValue;

pub trait ServFn {
	fn call(&self, input: ServValue, ctx: &mut Context) -> Result<ServValue, &'static str>;
}


// #[derive(Clone)]
// struct Serv(Arc<GlobalContext>);

// impl Service<Request<IncomingBody>> for Serv {
// 	type Response = Response<ServBody>;
// 	type Error = hyper::Error;
// 	type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

// 	fn call(&self, req: Request<IncomingBody>) -> Self::Future {
// 		let Ok(matched) = self.0.routes.at(req.uri().path()) else {
// 			let not_found_message = ServValue::Text("Error 404: Page Not Found".to_owned());
// 			let res = Ok(Response::builder().status(404).body(not_found_message.into()).unwrap());
// 			return Box::pin(async { res })
// 		};
// 		let mut inner_context = Context::from(&self.0);

// 		for (k, v) in matched.params.iter() {
// 			inner_context.push_str(k, v);
// 		}

// 		let result = matched.value.call(ServValue::None, &mut inner_context).unwrap();
// 		let res = Ok(Response::builder().body(result.into()).unwrap());
// 		Box::pin(async { res })
// 	}
// }


fn eval_function(word: lexer::Token, input: ServValue, ctx: &Context) -> ServValue {
    match word.contents.as_str() {
        "hello" => ServValue::Text("Hello World!".into()),
        "incr" => (input.expect_int().unwrap() + 1).into(),
        _ => todo!(),

    }
}

fn eval(expression: &mut VecDeque<ast::Word>, ctx: &Context) -> ServValue {
    while let Some(n) = expression.pop_front() {
		let v = match n {
    		ast::Word::Function(t) => {
        		eval_function(t, ServValue::Int(0), ctx)
    		},

    // 		ast::Word::Template(t) => {
				// t.render();
    // 		},
    		_ => todo!(),
		};
    }

    todo!();
}

fn compile_word(input: ast::Word) -> ServFunction {
    todo!();
    // match input {
    //     ast::Word::Function(t) => 

    // }
}

fn compile(input: Vec<ast::Word>) -> ServFunction {
    todo!();
}


#[tokio::main]
async fn main() {
	let input_path = std::env::args().nth(1).unwrap_or("src/test.serv".to_string());
	let input = std::fs::read_to_string(&input_path).unwrap();

	let ast = parser::parse_root_from_text(&input).unwrap();

	println!("{:#?}", ast);

	let mut ctx: Context = Context::empty();

	for declaration in ast.0 {
    	if declaration.kind == "word" {
        	ctx.insert(declaration.key.to_owned(), compile(declaration.value.0));
    	}
	}

	return;
	// let input = std::fs::read_to_string("src/test.serv").unwrap();
	// let AstNode::Root(ast) = parser::parse_root_from_text(&input).unwrap() else { panic!(); };

	// let interpreter = Compiler::new();
	// let mut ctx = GlobalContext::new(interpreter);

	// for element in ast {
	// 	match element {
	// 		AstNode::Declaration { ref name, expression } => ctx.insert_word(name, ctx.interpreter.compile(*expression).unwrap()),
	// 		AstNode::Route { ref pattern, expression } => ctx.insert_route(pattern, ctx.interpreter.compile(*expression).unwrap()),
	// 		_ => panic!("unexpected AST node"),
	// 	}
	// }

	// let global_context = Arc::new(ctx);
	// let addr = SocketAddr::from(([0,0,0,0], 4000));
	// let listener = TcpListener::bind(addr).await.unwrap();

	// if let Some(out) = global_context.values.get("out") {
	// 	let output = out.call(ServValue::None, &mut Context::from(&global_context)).unwrap();
	// 	println!("{}", output);
	// }

	// // loop {
	// // 	let (stream, _) = listener.accept().await.unwrap();
	// // 	let io = TokioIo::new(stream);
	// // 	let ctx_new = global_context.clone();

	// // 	tokio::task::spawn(async {
	// // 		conn::http1::Builder::new()
	// // 			.serve_connection(io, Serv(ctx_new))
	// // 			.await
	// // 			.unwrap();
	// // 	});

	// // }
}
