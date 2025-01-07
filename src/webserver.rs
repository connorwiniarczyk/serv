use hyper::service::Service;
use hyper::body::{Body, Frame, Incoming as IncomingBody};
use hyper::{ Request, Response };
use std::sync::Arc;
use std::pin::Pin;
use std::future::Future;
use std::task::{Poll, Context};
use tokio_rustls::rustls;

use crate::{ServFn, ServModule};
use crate::ServError;

use matchit::Router;

use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder;

use hyper::server::conn::http1;

use std::io::{BufReader, Read, Write};

use crate::{ServValue, Label};
use crate::Stack;
use std::net::SocketAddr;
use tokio::net::TcpListener;

use std::collections::VecDeque;

pub struct ServBody(Option<VecDeque<u8>>);

impl ServBody {
    pub fn generate(input: ServValue, scope: &Stack) -> Self {
        match input {
			ServValue::Ref(label) => ServBody::generate(scope.get(label).unwrap(), scope),
			f @ ServValue::Func(_) => ServBody::generate(f.call(None, scope).unwrap(), scope),
			ServValue::Raw(t) => Self(Some(t.into())),
			otherwise => {
    			let mut output = String::new();
				crate::value::DefaultSerializer(scope).write(otherwise, &mut output).unwrap();
				Self(Some(output.bytes().collect()))
			},
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
use crate::value::Serializer;

#[derive(Clone)]
struct Serv(Arc<Stack<'static>>, Arc<Router<ServValue>>);

impl Service<Request<IncomingBody>> for Serv {
	type Response = Response<ServBody>;
	type Error = hyper::Error;
	type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

	fn call(&self, mut req: Request<IncomingBody>) -> Self::Future {
    	let root = self.0.clone();
    	let router = self.1.clone();
    	let output = async move {
        	let (parts, body) = req.into_parts();
        	let parts_a = parts.clone();
        	let Ok(matched) = router.at(parts_a.uri.path()) else {
            	let text ="<h1>Error 404: Page Not Found</h1>".to_string();
            	let res = Response::builder()
                	.status(404)
                	.body(ServBody(Some(text.bytes().collect())))
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

    		let mut response = Response::builder();

        	let result = match matched.value {
            	ServValue::Func(ServFn::Expr(e, _)) => e.clone().eval(&mut scope),
            	value => value.call(None, &scope),
        	}.unwrap();

        	response = response.status(200);

        	if let Ok(mime) = scope.get("res.mime") {
            	let value = mime.call(None, &scope);
            	// println!("{:?}", value.unwrap().to_string());
            	// response = response.header("Content-Type", mime.call(None, &scope).unwrap().to_string());
            	// todo!();
            	response = response.header("Content-Type", value.unwrap().to_string());
        	}

        	if let Ok(ServValue::Module(m)) = scope.get("res.headers") {
            	for (mut p, mut a) in m.equalities {
                	let mut child = scope.make_child();
                	let key   = p.eval(&mut scope).unwrap().to_string();
                	let value = a.eval(&mut scope).unwrap().to_string();
                	response = response.header(&key, &value);
            	}
        	}

			// let mut text = String::new();
			// crate::value::DefaultSerializer(&scope).write(result, &mut text);
			// let body = ServBody(Some(text.bytes().collect()));
    		let response_sender = response.body(ServBody::generate(result, &scope)).unwrap();
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

fn get_port(scope: &mut Stack) -> Result<u16, ServError> {
    match scope.get("serv.port") {
        Ok(val) => Ok(val.call(None, &scope)?.expect_int()?.try_into().unwrap()),
        Err(e) => {
            scope.insert_name("serv.port", 4000.into());
            Ok(4000)
        },
    }
}

fn get_tls_info(scope: &Stack<'static>) -> Option<Arc<rustls::ServerConfig>> {
    let key = scope.get("serv.tlskey").ok()?.call(None, scope).expect("Failed running serv.tlskey").to_string();
    let mut reader = BufReader::new(key.as_bytes());
    let key = rustls_pemfile::private_key(&mut reader)
        .expect("failed to parse private key")
        .expect("failed to find private key");

    let cert = scope.get("serv.tlscert").ok()?.call(None, scope).expect("Failed running serv.tlscert").to_string();
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


pub async fn run_webserver(mut scope: Stack<'static>, router: Router<ServValue>) {
    let port: u16 = get_port(&mut scope).unwrap_or(4000);
	let addr = SocketAddr::from(([0,0,0,0], port));
	let listener = TcpListener::bind(addr).await.unwrap();
	let scope_arc = Arc::new(scope);
	let router_arc = Arc::new(router);

	if let Some(config) = get_tls_info(&scope_arc) {
    	println!("starting encrypted server");
        let tls_acceptor = tokio_rustls::TlsAcceptor::from(config);

    	loop {
    		let (tcp_stream, _) = listener.accept().await.unwrap();
    		let scope_arc = scope_arc.clone();
    		let router_arc = router_arc.clone();
    		let tls_acceptor = tls_acceptor.clone();

    		tokio::task::spawn(async move {
        		let Ok(tls_stream) = tls_acceptor.accept(tcp_stream).await else { panic!() };
    			http1::Builder::new()
    				.serve_connection(TokioIo::new(tls_stream), Serv(scope_arc, router_arc))
    				.await
    				.unwrap();
    		});
    	}

	} else {
    	println!("starting unencrypted server");
    	loop {
    		let (tcp_stream, _) = listener.accept().await.unwrap();
    		let scope_arc = scope_arc.clone();
    		let router_arc = router_arc.clone();

    		tokio::task::spawn(async move {
    			http1::Builder::new()
    				.serve_connection(TokioIo::new(tcp_stream), Serv(scope_arc, router_arc))
    				.await
    				.unwrap();
    		});
    	}
	}
}
