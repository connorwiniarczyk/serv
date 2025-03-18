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
			ServValue::Ref(ref addr) => ServBody::generate(crate::engine::deref(addr, scope).unwrap(), scope),
			ServValue::Func(_) => ServBody::generate(crate::engine::resolve(input, None, scope).unwrap(), scope),
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

fn get_mime_type<'a>(value: &'a ServValue, scope: &'a Stack) -> Option<String> {
    if let Ok(v) = scope.get("res.mime") {
		return Some(v.call(None, scope).ok()?.to_string());
    }

    if let ServValue::Text(t) = value {
        return Some(t.mime?.to_string())
    }

    None
}

fn response_from_value(input: ServValue, scope: &mut Stack) -> Response<ServBody> {
    let mut response = Response::builder();
	response = response.status(200);

	if let Some(mime) = get_mime_type(&input, &scope) {
    	response = response.header("Content-Type", mime);
	}

	if let Ok(ServValue::Module(m)) = crate::engine::deref(&"res.headers".into(), scope) {
    	// println!("{:?}", m);
    	for (mut p, mut a) in m.values {
        	let mut child = scope.make_child();
        	// let key   = p.eval(scope).unwrap().to_string();
        	let key   = p.to_string();
        	let value = a.call(None, scope).unwrap().to_string();
        	response = response.header(&key, &value);
    	}
	}

	// if let Ok(ServValue::Module(m)) = scope.get("res.cookie") {
 //    	for (mut p, mut a) in m.equalities {
 //        	let mut child = scope.make_child();
 //        	let key   = p.eval(scope).unwrap().to_string();
 //        	let value = a.eval(scope).unwrap();
 //        	let cookie_text = format!("{}={};path=/;SameSite=Strict", key, value );
 //        	response = response.header("Set-Cookie", &cookie_text);
 //    	}
	// }

	response.body(ServBody::generate(input, scope)).unwrap()
}

fn response_from_error(input: ServError, scope: &mut Stack) -> Response<ServBody> {
    let mut response = Response::builder();
	response = response.status(500);

	response.body(ServBody::generate(input.to_string().into(), scope)).unwrap()
}

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
            	let value = ServValue::Text(v.into());
    			scope.insert(k, value);
        	}

        	let body: bytes::Bytes = body.collect().await.unwrap().to_bytes();
        	scope.insert("req.body", ServValue::Text(body.into()));
        	scope.request = Some(parts);

    		let mut response = Response::builder();

        	let result = match matched.value {
            	ServValue::Func(ServFn::Expr(e, _)) => e.clone().eval(&mut scope),
            	value => value.call(None, &scope),
        	};

        	match result {
				Ok(value)  => Ok(response_from_value(value, &mut scope)),
				Err(error) => Ok(response_from_error(error, &mut scope)),
        	}


			// Ok(response_from_value(result, &scope))
    		// let response_sender = response.body(ServBody::generate(result, &scope)).unwrap();
    		// Ok(response_sender)
    	};

    	Box::pin(output)
	}
}

fn get_port(scope: &mut Stack) -> Result<u16, ServError> {
    match scope.get("serv.port") {
        Ok(val) => Ok(val.call(None, &scope)?.expect_int()?.try_into().unwrap()),
        Err(e) => {
            scope.insert("serv.port", 4000.into());
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
