use std::fmt;
use prettytable::{ Table, row, cell };
use itertools::Itertools;
use crate::pattern::{Pattern};
use crate::parser;
use crate::request_state::{RequestState};
use std::cell::RefCell;

use std::future::Future;
use std::pin::Pin;

use std::sync::Mutex;
use std::sync::Arc;

use pollster::FutureExt as _;

use rhai::{Engine, EvalAltResult, Scope, NativeCallContext};

// use crate::commands::cmd::Cmd;
use hyper::{Request, Response};
use std::collections::HashMap;

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
				Ok(_) => return route.resolve(req).await,
				Err(_) => continue,
			};

			// check to see if the request matches
			// if so, store the result in vars, otherwise continue
			// let vars = match route.pattern.compare(&req) {
			//	Ok(vars) => vars,
			//	Err(e) => continue,
			// };

			// // if the request matches, resolve it
			// let mut state = RequestState::new(&route, req, &self);
			// for (key, value) in vars { state.variables.insert(key, value); }

			// let mut engine = Engine::new();

			// engine.run(route.script.as_str()).unwrap();


			// let futures = std::mem::take(&mut state.futures);
			// tokio::spawn(async move {
			//	futures_util::future::join_all(futures).await;
			//	println!("done");
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

// type Task = Pin<Box<dyn Sync + Send + Future<Output = ()>>>;
type Task = Pin<Box<dyn Sync + Send + Future<Output = ()>>>;

use hyper::Body;

#[derive(Default)]
struct State {
	tasks: Vec<Task>,
	body: Body,

	test: String,
}

impl State {
	fn new() -> Self{
		Self::default()
	}
	fn arc(self) -> ArcState {
		let arc = Arc::new(Mutex::new(self));
		ArcState(arc)
	}
}

use std::ops::Deref;
use futures_util::StreamExt;
use futures_util::stream;

#[derive(Clone)]
struct ArcState(Arc<Mutex<State>>);

impl ArcState {

	fn body(&self) -> &Body {
		todo!();
	}

	fn body_mut(&self) -> &mut Body {
		todo!();
	}

	fn append(&self, value: &str) {
		let test = &mut self.0.lock().unwrap().test;
		test.push_str(value);
		// let body = &mut self.0.lock().unwrap().body;
		// let mut result = hyper::body::to_bytes(body).block_on().unwrap();


		// *body = Body::wrap_stream(body.chain(next));

		// *body = Body::from(value.to_owned());


		// let next = body.chain(Body::from(value.to_owned()));
		// *body = Body::wrap_stream(next);
	}

	fn push(&mut self, x: &str) {
		todo!();
		// self.0.push_str(x);
	}

	fn inner(self) -> State {
		todo!();
	}

    fn register_task<T>(&self, task: T)
    where T: Future<Output = ()> + Sync + Send + 'static {
		self.0.lock().unwrap().tasks.push(Box::pin(task));
    }
}

use hyper::body::Bytes;
// use tokio::fs::File;
use bytes::BytesMut;
// use tokio::io::AsyncReadExt;

use std::fs::File;
use std::io::Read;

impl Route {
	pub async fn resolve(&self, mut request: Request<hyper::Body>) -> Result<Response<hyper::Body>, hyper::Error> {
		let vars = self.pattern.compare(&request).unwrap();
		let script = self.script.clone();
		let output = State::new().arc();

        let (sender, body) = Body::channel();
        let sender_mux = Arc::new(Mutex::new(sender));

		let worker = std::thread::spawn(move || {
            let output = output.clone();
			let mut engine = Engine::new();
			let ast = engine.compile(script).unwrap();

			let mut scope = Scope::new();
			for (key, value) in vars {
				scope.push(key, value);	
			}

            {
                let sender_mux = sender_mux.clone();
                let output = output.clone();
                engine.register_fn("read", move |path: &str| {
                    let mut file = File::open(path).unwrap();
                    let mut buffer = [0u8; 1024];
                    loop {
                        let n = file.read(&mut buffer).unwrap();
                        if n == 0 { break }

                        let lock = &mut sender_mux.lock().unwrap();
                        lock.send_data(Bytes::copy_from_slice(&buffer[0..n])).block_on();
                    }
                });
            }

            {
                let sender_mux = sender_mux.clone();
                engine.register_fn("echo", move |x: &str| {
                    sender_mux.lock().unwrap().send_data(Bytes::from(x.to_owned())).block_on();

                    // let next_element: Result<Bytes, hyper::Error> = Ok(Bytes::from(x.to_owned()));
                    // let next_element_str = stream::once(async {next_element});

                    // let current_body = &mut output_cl.0.lock().unwrap().body;
                    // let current_stream = std::mem::take(current_body);

                    // let sum = current_stream.chain(next_element_str);	
                    // *current_body = Body::wrap_stream(sum);
                });

            }

			engine.run_ast_with_scope(&mut scope, &ast).unwrap();
		});

        // tokio::spawn(state)

		// worker.join().unwrap();

		// let body = std::mem::take(&mut output.0.lock().unwrap().body);
		// let body = Body::from(output.0.lock().unwrap().test.clone());
		let mut out = hyper::Response::builder().status(200);
		Ok(out.body(body).unwrap())
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
