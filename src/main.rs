#![allow(unused_mut, unused_imports, unused_doc_comments, unused_macros, dead_code, unused_results, unused_must_use, unused_variables)]
#![allow(warnings)]

mod config;
// mod pattern;
// mod command;
mod request_state;
mod body;

mod route_table;

mod parser;
mod routetree;
mod ast;

extern crate pest;
#[macro_use]
extern crate pest_derive;

use std::sync::Arc;
use std::fs::File;

use clap::clap_app;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Server};
use hyper::server::conn::AddrIncoming;
use hyper::server::accept;

use futures_util::stream::StreamExt;
use futures_util::future::ready;
use tls_listener::TlsListener;
use tokio_rustls::TlsAcceptor;

use route_table::RouteTable;
use config::{Config};

#[tokio::main]
async fn main() -> hyper::Result<()> {

    // Define this programs arguments
    let matches = clap_app!(serv =>
        (version: "0.4")
        (author: "Connor Winiarczyk")
        (about: "A DSL for HTTP servers")
        (@arg port: -p --port +takes_value "which tcp port to listen on")
        (@arg host: -h --host +takes_value "which ip addresses to accept connections from")
        (@arg cert: -c --cert +takes_value "path to ssl certificate")
        (@arg key:  -k --key  +takes_value "path to matching rsa key")
        (@arg debug: -d ... "Sets the level of debugging information")
        (@arg PATH: "the directory to serve files from")
    ).get_matches();

    let mut config = Config::from_args(&matches);

    // It is important to cd into the target directory so that shell scripts invoked in that
    // directory will know what directory they are being run from
    std::env::set_current_dir(&config.root).expect("could not cd into that directory!");
    println!("\nServing Directory: {:?}\n", config.root);


    // Generate the Route Table
    let route_table: Arc<RouteTable> = {
        let routefile = config.root.join("test.serv");
        let tree = match File::open(&routefile) {
            Ok(file) => parser::parse(file).expect("syntax error:"),
            Err(_) => todo!(),
            // Err(_) => RouteTable::default(),
        };

        let output = RouteTable::new(tree);


        // The Route Table needs to be behind an Arc smart pointer because it will be shared
        // between multiple async processes. We do not need a Mutex here because once generated,
        // the Route Table can not be mutated
        Arc::new(output)
    };

    println!("Generated the following Route Table:");
    // println!("{}", route_table);

    // config = config.from_routes(&route_table.clone());


    //run the on-start commands if they are specified
    // if let Some(_) = route_table.get("onstart") {
    //     let route_table = route_table.clone();
    //     std::thread::spawn(move || {
    //         let route = route_table.get("onstart").unwrap();
    //         let dummy_request = Request::new(hyper::Body::empty());
    //         let mut state = request_state::RequestState::new(&route, dummy_request, &route_table);
    //         // for command in &route.commands {
    //         //     command.run(&mut state);
    //         // }
    //     });
    // }

    // Start the server
    let keypair = config.keypair.clone();
    match keypair.map(|keypair| keypair.into_tls_acceptor()) {
        None => start_unencrypted(route_table.clone(), &config).await?,
        Some(Err(e)) => {
            println!("failed to load valid certificates and keys");
            println!("error: {:?}", e);
            println!("falling back to unencrypted mode...");
            start_unencrypted(route_table.clone(), &config).await?
        }
        Some(Ok(ref acceptor)) => start_encrypted(route_table.clone(), &config, acceptor.clone()).await?,
    };

    Ok(())
}

// async fn start_encrypted(route_table: Arc<RouteTable>, config: &Config, keypair: KeyReader) -> hyper::Result<()>{
async fn start_encrypted(route_table: Arc<RouteTable>, config: &Config, acceptor: TlsAcceptor) -> hyper::Result<()>{

    println!("starting encrypted server\n");

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

    let listen_addr = ([0,0,0,0], config.port).into();
    let addr = AddrIncoming::bind(&listen_addr)?;
    // let incoming = TlsListener::new(keypair.into_tls_acceptor().unwrap(), addr).filter(|conn|{
    let incoming = TlsListener::new(acceptor, addr).filter(|conn|{
        if let Err(err) = conn {
            println!("Error: {:?}", err);
            ready(false)
        } else {
            ready(true)
        }
    });

    Server::builder(accept::from_stream(incoming)).serve(service).await;
    Ok(())
}

async fn start_unencrypted(route_table: Arc<RouteTable>, config: &Config) -> hyper::Result<()> {
    println!("starting unencrypted server\n");
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

    let listen_addr = ([0,0,0,0], config.port).into();
    let addr = AddrIncoming::bind(&listen_addr)?;
    Server::builder(addr).serve(service).await; 
    Ok(())
    // Server::bind(&listen_addr).serve(service).await;
}
