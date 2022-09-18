#![allow(unused_mut, unused_doc_comments, unused_macros, dead_code, unused_results, unused_must_use, unused_variables)]
// #![allow(warnings)]

mod config;
mod route_table;
mod pattern;
mod parser;
mod command;
mod request_state;
mod body;

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

use route_table::RouteTable;
use config::{Config, KeyPair};

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

    let config = Config::from_args(&matches);

    // It is important to cd into the target directory so that shell scripts invoked in that
    // directory will know what directory they are being run from
    std::env::set_current_dir(&config.root).expect("could not cd into that directory!");
    println!("\nServing Directory: {:?}\n", config.root);

    // let config = Config {
    //     root: current_dir().unwrap(),
    //     port: port.parse().unwrap(), // parse port value into an integer
    //     host: host.to_string(),
    //     keypair,
    // };

    let route_table = {
        let routefile = config.root.join("routes.conf");
        let output = match File::open(&routefile) {
            Ok(file) => parser::parse(file).expect("syntax error:"),
            Err(_) => RouteTable::default(),
        };

        Arc::new(output)
    };

    println!("Generated the following Route Table:");
    println!("{}", route_table);

    //run the on-start commands if they are specified
    if let Some(_) = route_table.get("onstart") {
        let route_table = route_table.clone();
        std::thread::spawn(move || {
            let route = route_table.get("onstart").unwrap();
            let dummy_request = Request::new(hyper::Body::empty());
            let mut state = request_state::RequestState::new(&route, &dummy_request, &route_table);
            for command in &route.commands {
                command.run(&mut state);
            }
        });
    }

    match config.keypair {
        None => start_unencrypted(route_table.clone(), &config).await?,
        Some(ref keypair) => start_encrypted(route_table.clone(), &config, keypair.clone()).await?,
    };

    Ok(())
}

async fn start_encrypted(route_table: Arc<RouteTable>, config: &Config, keypair: KeyPair) -> hyper::Result<()>{

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
    let incoming = TlsListener::new(keypair.into_tls_acceptor(), addr).filter(|conn|{
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
