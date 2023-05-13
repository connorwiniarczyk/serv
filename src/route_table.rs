use std::fmt;
use prettytable::{ Table, row, cell };
use itertools::Itertools;
use crate::pattern::{Pattern};
use crate::parser;
use crate::request_state::{RequestState};
use std::cell::RefCell;

use crate::stream_handle::{StreamHandle};

use std::future::Future;
use std::pin::Pin;

use std::sync::Mutex;
use std::sync::Arc;

use pollster::FutureExt as _;

use rhai::{Engine, EvalAltResult, Scope, NativeCallContext};

use std::ops::Deref;
use futures_util::StreamExt;
use futures_util::stream;
use hyper::body::Bytes;
use bytes::BytesMut;
use std::fs::File;
use std::io::Read;

use hyper::{Request, Response};
use std::collections::HashMap;
use hyper::Body;

use crate::script::*;

type Task = Pin<Box<dyn Sync + Send + Future<Output = ()>>>;

use hyper::body::Sender;

struct Resolver {
	engine: Engine,
	sender: Arc<Mutex<Sender>>,

	// idk
	tasks: Vec<Task>,
	body: Body,
}

impl Resolver {
	fn new(engine: Engine) -> Self {
		let (sender_r, body) = Body::channel();
		let sender = Arc::new(Mutex::new(sender_r));

		Self {engine, sender, tasks: Vec::default(), body }
	}

	fn sender(&self) -> Sender {
		todo!();
	}

	fn register_read_fn(&mut self) {
		let sender = self.sender.clone();
		self.engine.register_fn("read", move |path: &str| {
			let mut file = File::open(path).expect("no file"); 
			let mut buffer = [0u8; 1024];
			loop {
				let n = file.read(&mut buffer).expect("reading from file failed");
				if n == 0 { break }
				sender.lock().unwrap().send_data(Bytes::copy_from_slice(&buffer[0..n])).block_on();
			}
		});
	}

	fn resolve(self) -> Body {
		todo!();
	}
}


#[derive(Clone)]
pub struct Route {
	pub pattern: Pattern,
	pub script: String,
}

impl Route {
	pub async fn resolve(&self, mut request: Request<hyper::Body>) -> Result<Response<hyper::Body>, http::Error> {
		let engine = create_engine();
		let text = r#"file("main.rs")"#;
		let res: StreamHandle = engine.eval(text).unwrap();
		let res_inner = Arc::try_unwrap(res.inner).unwrap();

		let mut out = hyper::Response::builder().status(200);
		out.body(res_inner)
		// let body = hyper::Body::wrap_stream(res_inner);
		// todo!();
	}
}

impl fmt::Debug for Route {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{:<8}", self.pattern)
	}
}

impl fmt::Display for Route {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{:<8}", self.pattern)
	}
}

pub struct RouteTable {
	pub table: Vec<Route>,
	pub names: HashMap<String, usize>,
}

impl RouteTable {
	pub fn add(&mut self, route: Route ) {
		if let Some(ref name) = route.pattern.name {
			self.names.insert(name.clone(), self.table.len());
		}
		self.table.push( route ); 
	}

	pub fn new() -> Self {
		Self { table: Vec::new(), names: HashMap::new() }
	}

	/// get the route with the given assigned name, or None
	pub fn get(&self, name: &str) -> Option<&Route> {
		self.names.get(name).map(|index| &self.table[index.clone()])
	}

	pub async fn resolve(&self, mut req: Request<hyper::Body>) -> Result<Response<hyper::Body>, http::Error> {
		println!("incoming http request: {}", req.uri());
		for route in self.table.iter() {

			match route.pattern.compare(&req) {
				Ok(_) => return route.resolve(req).await,
				Err(_) => continue,
			};
		}

		return Ok(Response::new(hyper::Body::from("404 error")))
	}
}

impl Default for RouteTable {
	fn default() -> Self {
		todo!();
		// parser::parse_str("/**path: read $(path)").unwrap()
	}
}

impl fmt::Display for RouteTable {
	fn fmt(&self, f: &mut fmt::Formatter<'_>)  -> fmt::Result {
		// todo!();

		let mut table = Table::new();
		table.add_row(row![ "ROUTE", "COMMANDS"]);
		for row in self.table.iter() {
			table.add_row(row![row.pattern, row.script]);
		}

		write!(f, "{}", table)
	}
}
