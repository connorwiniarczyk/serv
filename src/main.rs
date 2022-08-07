// #![deny(warnings)e
// #![allow(unused_imports, unused_mut, unused_doc_comments, unused_macros, dead_code, unused_results, unused_must_use, unused_variables)]
#![allow(warnings)]

mod config;
mod route_table;
mod pattern;
mod parser;

mod command;
mod request_state;

use std::convert::Infallible;
use route_table::Route;

use clap::clap_app;
use std::env::current_dir;
use std::path::Path;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use hyper::server::conn::AddrIncoming;

use std::path::PathBuf;

use futures_util::stream::StreamExt;
use futures_util::future::ready;
use hyper::server::accept;

use rustls_pemfile::{Item, read_all, read_one};

use std::io::BufReader;
use std::fs::File;

use tls_listener::TlsListener;
use std::sync::Arc;
use tokio_rustls::rustls::{Certificate, PrivateKey, ServerConfig};

pub type Acceptor = tokio_rustls::TlsAcceptor;



#[derive(Clone, Debug)]
struct KeyPair {
    key: PathBuf,
    cert: PathBuf,
}

impl KeyPair {
    pub fn into_tls_acceptor(&self) -> Acceptor {
        let mut key_file = BufReader::new(File::open(&self.key).unwrap());
        let key_der = match read_one(&mut key_file).unwrap().unwrap() {
            Item::RSAKey(key) => key,
            _ => panic!("not a valid key"),
        };

        let mut cert_file = BufReader::new(File::open(&self.cert).unwrap());
        let cert_der = match read_one(&mut cert_file).unwrap().unwrap() {
            Item::X509Certificate(cert) => cert,
            _ => panic!("not a valid certificate"),
        };

        let key = PrivateKey(key_der.into());
        let cert = Certificate(cert_der.into());

        let server_config = ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(vec![cert], key)
            .unwrap();
         
        Arc::new(server_config).into()
    }
}

#[derive(Clone, Debug)]
pub struct Config {
    // route_table: RouteTable,
    root: PathBuf,
    port: u32,
    host: String,

    keypair: Option<KeyPair>,
}


#[tokio::main]
async fn main() -> hyper::Result<()> {

    // Define this programs arguments
    let matches = clap_app!(serv =>
        (version: "0.3")
        (author: "Connor Winiarczyk")
        (about: "A Based Web Server")
        (@arg port: -p --port +takes_value "which tcp port to listen on")
        (@arg host: -h --host +takes_value "which ip addresses to accept connections from")
        (@arg cert: -c --cert +takes_value "path to ssl certificate")
        (@arg key:  -k --key  +takes_value "path to matching rsa key")
        (@arg debug: -d ... "Sets the level of debugging information")
        (@arg PATH: "the directory to serve files from")
    ).get_matches();

    let port = matches.value_of("port").unwrap_or("4000");
    let host = matches.value_of("host").unwrap_or("0.0.0.0");

    let keypair = match (matches.value_of("cert"), matches.value_of("key")) {
        (Some(cert), Some(key)) => Some(KeyPair {
            cert: Path::new(&cert).to_path_buf(),
            key: Path::new(&key).to_path_buf(),
        }),
        (None, None) => None,

        // if one is set but not the other
        _ => panic!("please specify both a key and a cert or neither"),
    };


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

        keypair,

        // keypair: Some(KeyPair {
        //     key: Path::new("local.key").to_path_buf(),
        //     cert: Path::new("local.cert").to_path_buf(),
        // }),

        // cert: Some(PathBuf::new()),
        // key: Some(),
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

    match config.keypair {
        None => {
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

            Server::bind(&listen_addr).serve(service).await;
        },

        Some(keypair) => {
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


            let addr = AddrIncoming::bind(&listen_addr)?;
            // let incoming = TlsListener::new(tls_acceptor(&keypair), addr).filter(|conn|{
            let incoming = TlsListener::new(keypair.into_tls_acceptor(), addr).filter(|conn|{
                if let Err(err) = conn {
                    eprintln!("Error: {:?}", err);
                    ready(false)
                } else {
                    ready(true)
                }
            });

            Server::builder(accept::from_stream(incoming)).serve(service).await;

        }
    };

    Ok(())
}
