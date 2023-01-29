use std::fmt;
use prettytable::{ Table, row, cell };
use itertools::Itertools;
use crate::pattern::{Pattern};
use crate::parser;
use crate::request_state::{RequestState};
use std::cell::RefCell;

use std::sync::Mutex;
// use std::borrow::Borrow;

use rhai::{Engine, EvalAltResult, Scope, NativeCallContext};

// use crate::commands::cmd::Cmd;
use hyper::{Request, Response};
use std::collections::HashMap;
// use std::pin::Pin;

use std::sync::Arc;

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

	pub async fn resolve(&self, mut req: Request<hyper::Body>) -> Result<Response<hyper::Body>, hyper::Error> {
		println!("incoming http request: {}", req.uri());
		for route in self.table.iter() {

			match route.pattern.compare(&req) {
				Ok(_) => return route.resolve(req),
				Err(_) => continue,
			};

			// check to see if the request matches
			// if so, store the result in vars, otherwise continue
			// let vars = match route.pattern.compare(&req) {
			// 	Ok(vars) => vars,
			// 	Err(e) => continue,
			// };

			// // if the request matches, resolve it
			// let mut state = RequestState::new(&route, req, &self);
			// for (key, value) in vars { state.variables.insert(key, value); }

			// let mut engine = Engine::new();

			// engine.run(route.script.as_str()).unwrap();


			// let futures = std::mem::take(&mut state.futures);
			// tokio::spawn(async move {
			// 	futures_util::future::join_all(futures).await;
			// 	println!("done");
			// });

			// return Ok(state.into());
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

#[derive(Clone)]
pub struct Route {
	pub pattern: Pattern,
	pub script: String,
}

#[derive(Clone)]
struct State(RefCell<String>);

impl State {
	fn new() -> Self {
		Self(RefCell::new(String::new()))
	}
	fn push(&mut self, x: &str) {
		todo!();
		// self.0.push_str(x);
	}
}

use std::rc::Rc;

impl Route {
	pub fn resolve(&self, mut request: Request<hyper::Body>) -> Result<Response<hyper::Body>, hyper::Error> {
		let vars = self.pattern.compare(&request).unwrap();
		let mut scope = Scope::new();
		for (key, value) in vars {
			scope.push(key, value);	
		}

		let mut output = Rc::new(RefCell::new(String::new()));
		let mut output_cl = output.clone();

		let mut engine = Engine::new();

		engine.register_fn("echo", move |x: &str| {
			output_cl.borrow_mut().push_str(x);
		});

		let ast = engine.compile(self.script.as_str()).unwrap();

		engine.run_ast_with_scope(&mut scope, &ast).unwrap();

		println!("{}", output.borrow());

		// let test = output.borrow_mut();
		let inner = output.replace(String::new());
        let mut out = hyper::Response::builder().status(200);
		Ok(out.body(inner.into()).unwrap())
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
