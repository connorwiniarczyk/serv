use std::fmt;
use prettytable::{ Table, row, cell };
use itertools::Itertools;
// use crate::pattern::{Pattern};
// use crate::parser;
use crate::request_state::{RequestState};
// use crate::commands::cmd::Cmd;
use hyper::{Request, Response};
use std::collections::{HashSet, HashMap};

use std::iter::Peekable;

use std::sync::Arc;

#[derive(Hash, Eq, PartialEq, Clone)]
pub enum NodeType {
	Value(String),
	Wildcard(String),
	DeepWildcard(String),
}

impl NodeType {
	fn compare(&self, input: &str, acc: &mut HashMap<String, String>) -> bool {
		match self {
			Self::Value(s) => return s == input,
			Self::Wildcard(v) => {
				acc.insert(v.clone(), input.to_owned());
				return true
			},
			Self::DeepWildcard(v) => unreachable!(),
		}
	}
}

use NodeType::*;

pub struct PathNode {
	node_type: NodeType,
	expression: String,
	children: Vec<PathNode>,
}

impl PathNode {
	fn new(node_type: NodeType) -> Self {
		Self { node_type, expression: String::new(), children: Vec::new() }
	}

	pub fn add_child(&mut self, child: NodeType) {
		// match child.NodeType
		self.children.push(Self::new(child));
	}

	pub fn resolve<I>(&self, mut path: Peekable<I>, acc: HashMap<String, String>) -> Result<(), ()> 
	where I: Iterator<Item = String> {
		match (&self.node_type, path.next(), path.peek()) {

			(Value(x), None, None) => return Err(()),

			(Value(x), Some(y), None) if x == &y => Ok(()),
			(Value(x), Some(y), None) => Err(()),

			(Value(x), Some(y), Some(z)) if (x == &y) => {
				// for child in self.children {
				//	match => child.resolve(path, acc)
				// }
				todo!();
				// if (self.children.)
			},
			_ => todo!(),
		};

		todo!();
	}
}

pub struct RouteTable {
	pub table: Vec<Route>,
	pub names: HashMap<String, usize>,
}

impl RouteTable {
	pub fn add(&mut self, route: Route ) {
		// if let Some(ref name) = route.pattern.name {
		//	self.names.insert(name.clone(), self.table.len());
		// }
		// self.table.push( route ); 
	}

	pub fn new() -> Self {
		Self { table: Vec::new(), names: HashMap::new() }
	}

	/// get the route with the given assigned name, or None
	pub fn get(&self, name: &str) -> Option<&Route> {
		self.names.get(name).map(|index| &self.table[index.clone()])
	}

	pub async fn resolve(&self, mut req: Request<hyper::Body>) -> Result<Response<hyper::Body>, hyper::Error> {
		todo!();
		// println!("incoming http request: {}", req.uri());
		// for route in self.table.iter() {

		//	// check to see if the request matches
		//	// if so, store the result in vars, otherwise continue
		//	let vars = match route.pattern.compare(&req) {
		//		Ok(vars) => vars,
		//		Err(e) => continue,
		//	};

		//	// if the request matches, resolve it
		//	let mut state = RequestState::new(&route, req, &self);
		//	for (key, value) in vars { state.variables.insert(key, value); }


		//	for command in &route.commands {
		//		command.run(&mut state).await;
		//	}

		//	let futures = std::mem::take(&mut state.futures);
		//	tokio::spawn(async move {
		//		futures_util::future::join_all(futures).await;
		//		println!("done");
		//	});

		//	return Ok(state.into());
		// }

		// return Ok(Response::new(hyper::Body::from("404 errer")))
	}
}

impl Default for RouteTable {
	fn default() -> Self {
		todo!();
		// parser::parse("/**path: read $(path)".as_bytes()).unwrap()
	}
}

impl fmt::Display for RouteTable {
	fn fmt(&self, f: &mut fmt::Formatter<'_>)  -> fmt::Result {
		todo!();

		// let mut table = Table::new();
		// table.add_row(row![ "ROUTE", "COMMANDS"]);
		// for row in self.table.iter() {
		//	let commands = row.commands.iter().map(|command| command.to_string().replace("\n", "")).join("\n");
		//	table.add_row(row![row.pattern, commands]);
		// }

		// write!(f, "{}", table)
	}
}

#[derive(Clone)]
pub struct Route {
	// pub pattern: Pattern,
	// pub commands: Vec<Command>,
	// pub commands: Vec<Arc<dyn Cmd>>,
}

impl fmt::Debug for Route {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		todo!();
		// write!(f, "{:<8}", self.pattern)
	}
}

impl fmt::Display for Route {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		todo!();
		// write!(f, "{:<8}", self.pattern)
	}
}
