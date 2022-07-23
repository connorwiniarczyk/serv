// #![deny(warnings)e
// #![allow(unused_imports, unused_mut, unused_doc_comments, unused_macros, dead_code, unused_results, unused_must_use, unused_variables)]
#![allow(warnings)]

mod config;
use crate::config::Config;
mod route_table;
mod pattern;
mod parser;

mod command;
mod request_state;

use std::convert::Infallible;

use route_table::Route;
// use config::Config;

use clap::clap_app;
use std::env::current_dir;
use std::path::Path;
// use tide;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use hyper::server::conn::AddrIncoming;






// tls stuff
use tls_listener::TlsListener;

use std::sync::Arc;
use tokio_rustls::rustls::{Certificate, PrivateKey, ServerConfig};

const CERT: &[u8] = include_bytes!("tls_config/local.cert");
const PKEY: &[u8] = include_bytes!("tls_config/local.key");

pub type Acceptor = tokio_rustls::TlsAcceptor;

fn tls_acceptor_impl(cert_der: &[u8], key_der: &[u8]) -> Acceptor {
    let key = PrivateKey(cert_der.into());
    let cert = Certificate(key_der.into());
    Arc::new(
        ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(vec![cert], key)
            .unwrap(),
    )
    .into()
}


pub fn tls_acceptor() -> Acceptor {
    tls_acceptor_impl(PKEY, CERT)
}

// mod tls_config;
// use tls_config::tls_acceptor;


#[tokio::main]
async fn main() {

    // Define this programs arguments
    let matches = clap_app!(serv =>
        (version: "0.2")
        (author: "Connor Winiarczyk")
        (about: "A Based Web Server")
        (@arg port: -p --port +takes_value "which tcp port to listen on")
        (@arg host: -h --host +takes_value "which ip addresses to accept connections from")
        (@arg debug: -d ... "Sets the level of debugging information")
        (@arg PATH: "the directory to serve files from")
    ).get_matches();

    let port = matches.value_of("port").unwrap_or("4000");
    let host = matches.value_of("host").unwrap_or("0.0.0.0");

    // Determine the local path to serve files out of 
    let path = Path::new(matches.value_of("PATH").unwrap_or("."));

    // if the path given has a root, ie. /home/www/public, use it as is,
    // if not, ie. server/public join it to the end of the current directory
    let path_abs = match path.has_root() {
        true => path.to_path_buf(),
        false => current_dir().unwrap().join(path),
    }.canonicalize().unwrap();

    println!("");
    println!("Serving Directory: {:?}", path_abs);

    // It is important to cd into the target directory so that shell scripts invoked in that
    // directory will know what directory they are being run from
    std::env::set_current_dir(&path_abs).expect("could not cd into that directory!");

    let config = Config {
        root: current_dir().unwrap(),
        port: port.parse().unwrap(), // parse port value into an integer
        host: host.to_string(),
    };

    let routefile = config.root.join("routes.conf");

    // Need to wrap the Route Table in an ARC so that we can move multiple references to it into
    // the different request handling closures
    let route_table = Arc::new(
        route_table::RouteTable::from_file(&routefile)
    );

    println!("Generated the following Route Table:");
    println!("{}", route_table);

    let listen_addr = ([0,0,0,0], port.parse::<u16>().unwrap_or(4000)).into();

    let service = make_service_fn(move |_| {
        let route_table = route_table.clone();
        async move {
            Ok::<_, hyper::Error>(service_fn(move |req: Request<Body>| {
                let route_table = route_table.clone();
                async move {
                    route_table.resolve(req).await
                }
            }))
        }
    });

    use hyper::server::conn::AddrIncoming;

    let incoming = TlsListener::new(tls_acceptor(), AddrIncoming::bind(&listen_addr).unwrap());
    // let incoming = TlsListener::new(tls_acceptor(), TcpListener::bind(&listen_addr).await.unwrap());

    let server = Server::builder(incoming).serve(service).await;
}


#[derive(Clone, Debug)]
pub struct State{
    route_table: Vec<Route>,
    config: Config,
}

