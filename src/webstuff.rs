// pub struct GlobalContext {
// 	routes: Router<ServFunction>,
// 	interpreter: Compiler,
// 	values: HashMap<String, ServFunction>,
// }

// impl GlobalContext {
// 	pub fn new(interpreter: Compiler) -> Self {
// 		Self {
// 			interpreter,
// 			routes: Router::new(),
// 			values: HashMap::new(),
// 		}
// 	}
// 	pub fn insert_word(&mut self, name: &str, value: ServFunction) {
// 		self.values.insert(name.to_string(), value);
// 	}

// 	pub fn insert_route(&mut self, name: &str, value: ServFunction) {
// 		self.routes.insert(name.to_string(), value);
// 	}
// }

// #[derive(Clone)]
// pub struct Context {
// 	global: Arc<GlobalContext>,
// 	values: HashMap<String, ServFunction>,
// }


// impl Context {

// 	pub fn from(global: &Arc<GlobalContext>) -> Self {
// 		Self {
// 			global: global.clone(),
// 			values: HashMap::new(),
// 		}
// 	}

// 	pub fn get(&self, input: &str) -> Option<ServFunction> {
// 		self.values.get(input).or(self.global.values.get(input)).map(|x| x.clone())
// 	}

// 	pub fn push(&mut self, input: String, value: ServFunction) {
// 		self.values.insert(input, value);
// 	}

// 	pub fn push_str(&mut self, name: &str, value: &str) {
// 		self.values.insert(name.to_string(), ServFunction::new(value.to_owned()));
// 	}

// 	pub fn interpreter(&self) -> &Compiler {
// 		&self.global.interpreter
// 	}
// }


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
