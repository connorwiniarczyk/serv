use hyper::service::Service;
use hyper::body::{Body, Frame, Incoming as IncomingBody};
use hyper::{ Request, Response };
use std::sync::Arc;
use std::pin::Pin;
use std::future::Future;
use std::task::{Poll, Context};
use tokio_rustls::rustls;

use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder;

use std::io::{BufReader, Read, Write};

// const CERT: &[u8] = include_bytes!("/home/connor/scratch/ssl/certificate.crt");
// const PKEY: &[u8] = include_bytes!("/home/connor/scratch/ssl/private.key");

use crate::{Scope, ServValue, ServFunction, FnLabel};
use crate::VecDeque;
use crate::SocketAddr;
use crate::TcpListener;

pub struct ServBody(Option<VecDeque<u8>>);

impl ServBody {
	pub fn new() -> Self {
		Self(Some("hello!".bytes().collect()))
	}
}

impl From<ServValue> for ServBody {
	fn from(input: ServValue) -> Self {
    	match input {
			ServValue::Raw(bytes) => Self(Some(bytes.into())),
			_ => Self(Some(input.to_string().bytes().collect())),
    	}
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
    	let Ok(matched) = router.at(req.uri().path()) else {
        	let text ="<h1>Error 404: Page Not Found</h1>".to_string();
        	let res = Response::builder()
            	.status(404)
            	.body(ServValue::Text(text).into())
            	.unwrap();
        	return Box::pin(async {Ok(res)})
    	};

		let mut scope = self.0.make_child();
    	for (k, v) in matched.params.iter() {
			scope.insert(FnLabel::name(k), ServFunction::Literal(ServValue::Text(v.to_string())));
    	}

    	let result = matched.value.call(ServValue::None, &mut scope).unwrap();
		let mut response = Response::builder();

		if let Some(data) = result.get_metadata() {
    		if let Some(status) = data.get("status") {
        		let code = status.expect_int().unwrap().clone();
        		response = response.status(u16::try_from(code).unwrap());
    		}

    		if let Some(ServValue::List(headers)) = data.get("headers") {
        		for header in headers {
            		let text = header.to_string();
                	let mut iter = text
                    	.split("=")
                    	.map(|x| x.trim());

                	let key = iter.next().unwrap(); // .ok_or("invalid header syntax")?;
                	let value = iter.next().unwrap(); // .ok_or("invalid header syntax")?;
            		response = response.header(key, value);
        		}
    		}
		}

		let response_sender = response.body(result.into()).unwrap();
		Box::pin(async { Ok(response_sender) })
	}
}


use hyper_util::rt::TokioIo;
use hyper::server::conn::http1::Builder;

fn get_port(scope: &Scope) -> Result<u16, &'static str> {
    let port_func = scope.get(&FnLabel::Name("port".to_owned())).ok_or("")?;
    let port = port_func.call(ServValue::None, scope)?.expect_int()?;

    Ok(port.try_into().unwrap())
}

const certfile: &str = "/home/connor/scratch/ssl/certificate.crt";
const keyfile: &str = "/home/connor/scratch/ssl/private.key";

pub async fn run_webserver(scope: Scope<'static>) {
    let port: u16 = get_port(&scope).unwrap_or(4000);

    let certs = rustls_pemfile::certs(
        &mut BufReader::new(&mut std::fs::File::open(certfile).unwrap()))
        .map(|x| x.unwrap())
        .collect();

    let key = rustls_pemfile::private_key(
        &mut BufReader::new(&mut std::fs::File::open(keyfile).unwrap())).unwrap().unwrap();

    let config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .unwrap();

    let tls_acceptor = tokio_rustls::TlsAcceptor::from(Arc::new(config));

	let addr = SocketAddr::from(([0,0,0,0], port));
	let listener = TcpListener::bind(addr).await.unwrap();

	let scope_arc = Arc::new(scope);

	println!("listening on port: {}", port);

	loop {
		let (stream, _) = listener.accept().await.unwrap();
		let io = TokioIo::new(stream);

		// let mut conn = rustls::ServerConnection::new(config_arc.clone()).unwrap();
		// conn.complet_io(&stream);

		let scope_arc = scope_arc.clone();
		let tls_acceptor = tls_acceptor.clone();

		tokio::task::spawn(async move {
    		let Ok(tls_stream) = tls_acceptor.accept(io) else { panic!() };
			Builder::new()
				.serve_connection(tls_stream, Serv(scope_arc))
				.await
				.unwrap();
		});
	}
}
