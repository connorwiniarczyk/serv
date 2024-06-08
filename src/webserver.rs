
use hyper::service::Service;
use hyper::body::Incoming as IncomingBody;
use hyper::{ Request, Response };
use std::future::Future;
use std::pin::Pin;

use std::net::SocketAddr;

use matchit::Router;

struct Serv {
	routes: Router<Arc<dyn ServFunction>>,
}

impl Service<Request<IncomingBody>> for Serv {
	type Response = Response<ServBody>;
	type Error = hyper::Error;
	type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

	fn call(&self, req: Request<IncomingBody>) -> Self::Future {
		let res = Ok(Response::builder().body(ServValue::from_str("abcd").into()).unwrap());
		Box::pin(async { res })
	}
}


use tokio::net::TcpListener;
use hyper::server::conn;
use hyper_util::rt::TokioIo;

	// let addr = SocketAddr::from(([0,0,0,0], 4000));
	// let listener = TcpListener::bind(addr).await.unwrap();

	// loop {
	// 	let (stream, _) = listener.accept().await.unwrap();
	// 	let io = TokioIo::new(stream);

	// 	tokio::task::spawn(async move {
	// 		conn::http1::Builder::new()
	// 			.serve_connection(io, Serv { routes: Router::new() })
	// 			.await
	// 			.unwrap();
	// 	});

	// }
