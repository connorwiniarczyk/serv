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

use hyper::server::conn::http1;

use std::io::{BufReader, Read, Write};

use crate::{ServValue, Label};
use crate::Stack;
use crate::VecDeque;
use crate::SocketAddr;
use crate::TcpListener;

pub struct ServBody(Option<VecDeque<u8>>);

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

use http_body_util::BodyExt;

#[derive(Clone)]
struct Serv(Arc<Stack<'static>>);

impl Service<Request<IncomingBody>> for Serv {
	type Response = Response<ServBody>;
	type Error = hyper::Error;
	type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

	fn call(&self, mut req: Request<IncomingBody>) -> Self::Future {
    	let root = self.0.clone();
    	let output = async move {
        	let router = root.router.as_ref().unwrap();
        	let (parts, body) = req.into_parts();
        	let parts_a = parts.clone();
        	let Ok(matched) = router.at(parts_a.uri.path()) else {
            	let text ="<h1>Error 404: Page Not Found</h1>".to_string();
            	let res = Response::builder()
                	.status(404)
                	.body(ServValue::Text(text).into())
                	.unwrap();
            	return Ok(res)
        	};

    		let mut scope = root.make_child();
        	for (k, v) in matched.params.iter() {
    			scope.insert(Label::name(k), ServValue::Text(v.to_string()));
        	}

        	let body: bytes::Bytes = body.collect().await.unwrap().to_bytes();
        	scope.insert(Label::name("req.body"), ServValue::Raw(body.into()));
        	scope.request = Some(parts);

        	// let result = matched.value.call(ServValue::None, &mut scope).unwrap();
        	let result = matched.value.call(None, &mut scope).unwrap();
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
    		Ok(response_sender)
    	};

    	Box::pin(output)
	}

	// fn call(&self, req: Request<IncomingBody>) -> Self::Future {
 //    	let router = self.0.router.as_ref().unwrap();
 //    	let Ok(matched) = router.at(req.uri().path()) else {
 //        	let text ="<h1>Error 404: Page Not Found</h1>".to_string();
 //        	let res = Response::builder()
 //            	.status(404)
 //            	.body(ServValue::Text(text).into())
 //            	.unwrap();
 //        	return Box::pin(async {Ok(res)})
 //    	};

	// 	let mut scope = self.0.make_child();
 //    	for (k, v) in matched.params.iter() {
	// 		scope.insert(Label::name(k), ServFunction::Literal(ServValue::Text(v.to_string())));
 //    	}

 //    	let result = matched.value.call(ServValue::None, &mut scope).unwrap();
	// 	let mut response = Response::builder();

	// 	if let Some(data) = result.get_metadata() {
 //    		if let Some(status) = data.get("status") {
 //        		let code = status.expect_int().unwrap().clone();
 //        		response = response.status(u16::try_from(code).unwrap());
 //    		}

 //    		if let Some(ServValue::List(headers)) = data.get("headers") {
 //        		for header in headers {
 //            		let text = header.to_string();
 //                	let mut iter = text
 //                    	.split("=")
 //                    	.map(|x| x.trim());

 //                	let key = iter.next().unwrap(); // .ok_or("invalid header syntax")?;
 //                	let value = iter.next().unwrap(); // .ok_or("invalid header syntax")?;
 //            		response = response.header(key, value);
 //        		}
 //    		}
	// 	}

	// 	let response_sender = response.body(result.into()).unwrap();
	// 	Box::pin(async { Ok(response_sender) })
	// }
}

fn get_port(scope: &Stack) -> Result<u16, &'static str> {
    let port = scope.get("serv.port").ok_or("serv.port not defined")?.call(None, scope)?.expect_int()?;
    Ok(port.try_into().unwrap())
}

fn get_tls_info(scope: &Stack<'static>) -> Option<Arc<rustls::ServerConfig>> {
    let key = scope.get("serv.tlskey")?.call(None, scope).expect("Failed running serv.tlskey").to_string();
    let mut reader = BufReader::new(key.as_bytes());
    let key = rustls_pemfile::private_key(&mut reader)
        .expect("failed to parse private key")
        .expect("failed to find private key");

    let cert = scope.get("serv.tlscert")?.call(None, scope).expect("Failed running serv.tlscert").to_string();
    let mut reader_cert = BufReader::new(cert.as_bytes());
    let certs = rustls_pemfile::certs(&mut reader_cert)
        .map(|cert| cert.expect("failed to parse cert"))
    	.collect();

	let output = rustls::ServerConfig::builder()
    	.with_no_client_auth()
    	.with_single_cert(certs, key)
    	.expect("failed to build config");

	Some(Arc::new(output))
}

pub async fn run_webserver(scope: Stack<'static>) {
    let port: u16 = get_port(&scope).unwrap_or(4000);
	let addr = SocketAddr::from(([0,0,0,0], port));
	let listener = TcpListener::bind(addr).await.unwrap();
	let scope_arc = Arc::new(scope);

	if let Some(config) = get_tls_info(&scope_arc) {
    	println!("starting encrypted server");
    	println!("listening on port: {}", port);
        let tls_acceptor = tokio_rustls::TlsAcceptor::from(config);

    	loop {
    		let (tcp_stream, _) = listener.accept().await.unwrap();
    		let scope_arc = scope_arc.clone();
    		let tls_acceptor = tls_acceptor.clone();

    		tokio::task::spawn(async move {
        		let Ok(tls_stream) = tls_acceptor.accept(tcp_stream).await else { panic!() };
    			http1::Builder::new()
    				.serve_connection(TokioIo::new(tls_stream), Serv(scope_arc))
    				.await
    				.unwrap();
    		});
    	}

	} else {
    	println!("starting unencrypted server");
    	println!("listening on port: {}", port);
    	loop {
    		let (tcp_stream, _) = listener.accept().await.unwrap();
    		let scope_arc = scope_arc.clone();

    		tokio::task::spawn(async move {
    			http1::Builder::new()
    				.serve_connection(TokioIo::new(tcp_stream), Serv(scope_arc))
    				.await
    				.unwrap();
    		});
    	}
	}
}
