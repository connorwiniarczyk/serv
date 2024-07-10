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
use dictionary::FnLabel;

use std::collections::VecDeque;
use std::iter::Peekable;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use matchit::Router;

type ServResult = Result<ServValue, &'static str>;
pub type Scope<'a> = dictionary::StackDictionary<'a, ServFunction>;

impl<'a> Scope<'a> {
    pub fn get_str(&self, input: &str) -> Result<ServFunction, &'static str> {
        self.get(&FnLabel::name(input)).ok_or("word not found")
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
    List(Vec<FnLabel>),
    Composition(Vec<FnLabel>),
}

impl ServFunction {
    // TODO:
    // pub fn call_with_words(&self, input: ServValue, scope: &Scope, words: &mut Words) -> ServResult {
    //     match self {
    //         Self::Core(f) => f(words.eval(scope.get("in").unwrap_or(), scope)?, scope),
    //         _ => todo!(),
    //     }
    // }

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

use hyper::service::Service;
use hyper::body::{Body, Frame, Incoming as IncomingBody};
use hyper::{ Request, Response };
use std::sync::Arc;
use std::pin::Pin;
use std::future::Future;
use std::task::{Poll, Context};

pub struct ServBody(Option<VecDeque<u8>>);

impl ServBody {
	pub fn new() -> Self {
		Self(Some("hello!".bytes().collect()))
	}
}

impl From<ServValue> for ServBody {
	fn from(input: ServValue) -> Self {
		Self(Some(input.to_string().bytes().collect()))
	}
}

impl Body for ServBody {
	type Data = VecDeque<u8>;
	type Error = &'static str;

	fn poll_frame(self: Pin<&mut Self>, _: &mut Context) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
		if let Some(data) = self.get_mut().0.take() {
			Poll::Ready(Some(Ok(Frame::data(data))))
		} else {
			Poll::Ready(None)
		}
	}
}

#[derive(Clone)]
struct Serv<'a>(Arc<Scope<'a>>);

impl Service<Request<IncomingBody>> for Serv<'_> {
	type Response = Response<ServBody>;
	type Error = hyper::Error;
	type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

	fn call(&self, req: Request<IncomingBody>) -> Self::Future {
    	let router = self.0.router.as_ref().unwrap();
    	let Ok(matched) = router.at(req.uri().path()) else { panic!() };

		let mut scope = self.0.make_child();
    	for (k, v) in matched.params.iter() {
        	println!("{:?}", v);
			scope.insert(FnLabel::name(k), ServFunction::Literal(ServValue::Text(v.to_string())));
    	}

    	let result = matched.value.call(ServValue::None, &mut scope).unwrap();
		let res = Ok(Response::builder().body(result.into()).unwrap());
		Box::pin(async { res })

    	// todo!();
		// let Ok(matched) = self.0.routes.at(req.uri().path()) else {
		// 	let not_found_message = ServValue::Text("Error 404: Page Not Found".to_owned());
		// 	let res = Ok(Response::builder().status(404).body(not_found_message.into()).unwrap());
		// 	return Box::pin(async { res })
		// };
		// let mut inner_context = Context::from(&self.0);

		// for (k, v) in matched.params.iter() {
		// 	inner_context.push_str(k, v);
		// }

		// let result = matched.value.call(ServValue::None, &mut inner_context).unwrap();
		// let res = Ok(Response::builder().body(result.into()).unwrap());
		// Box::pin(async { res })
	}
}


use hyper_util::rt::TokioIo;
use hyper::server::conn::http1::Builder;

async fn run_webserver(scope: Scope<'static>) {
	let addr = SocketAddr::from(([0,0,0,0], 4000));
	let listener = TcpListener::bind(addr).await.unwrap();

	let scope_arc = Arc::new(scope);

	loop {
		let (stream, _) = listener.accept().await.unwrap();
		let io = TokioIo::new(stream);

		let scope_arc = scope_arc.clone();

		tokio::task::spawn(async move {
			Builder::new()
				.serve_connection(io, Serv(scope_arc))
				.await
				.unwrap();
		});
	}
}

#[tokio::main]
async fn main() {
	let input_path = std::env::args().nth(1).unwrap_or("src/test.serv".to_string());
	let input      = std::fs::read_to_string(&input_path).unwrap();
	let ast        = parser::parse_root_from_text(&input).unwrap();

	let mut scope: Scope = Scope::empty();

	scope.insert(FnLabel::name("hello"),     ServFunction::Core(hello_world));
	scope.insert(FnLabel::name("uppercase"), ServFunction::Core(uppercase));
	scope.insert(FnLabel::name("incr"),      ServFunction::Core(incr));
	scope.insert(FnLabel::name("decr"),      ServFunction::Core(decr));
	scope.insert(FnLabel::name("%"),         ServFunction::Core(math_expr));
	scope.insert(FnLabel::name("sum"),       ServFunction::Core(sum));
	scope.insert(FnLabel::name("read"),       ServFunction::Core(read_file));
	scope.insert(FnLabel::name("file"),       ServFunction::Core(read_file));
	scope.insert(FnLabel::name("inline"),       ServFunction::Core(inline));

	scope.insert(FnLabel::name("!"),         ServFunction::Meta(drop));
	scope.insert(FnLabel::name("map"),       ServFunction::Meta(map));
	scope.insert(FnLabel::name("using"),     ServFunction::Meta(using));
	scope.insert(FnLabel::name("let"),       ServFunction::Meta(using));
	scope.insert(FnLabel::name("choose"),    ServFunction::Meta(choose));
	scope.insert(FnLabel::name("*"),         ServFunction::Meta(apply));

	for declaration in ast.0 {
    	// println!("{:?}", declaration);
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

	run_webserver(scope).await;
}
