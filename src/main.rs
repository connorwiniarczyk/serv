#![allow(unused_mut, dead_code, unused_results, unused_must_use, unused_variables)]
#![allow(warnings)]

mod options;
mod config;
mod route_table;
mod route_patterns;
mod error;
mod parser;

mod command;
mod request_state;

use route_table::Route;
use config::Config;
use tide::Response;

use async_std;
use clap::clap_app;
use std::env::current_dir;
use std::path::Path;
use tide;


pub async fn handler(mut http_request: Request) -> tide::Result {
    println!("incoming http request: {}", http_request.url());

    // get the body string if there is one
    let body = http_request.body_string().await.ok();

    let state = http_request.state();
    let route = http_request.param("route").unwrap_or("").to_string();

    for route in state.route_table.iter() {
        if let Ok(result) = route.resolve(&http_request, &body).await {
            println!();
            return Ok(result)
        }
    }

    println!("\t failed to find a matching route, serving 404 page instead");
    let response = Response::builder(404).body("not found").build();
    return Ok(response)
}



#[async_std::main]
async fn main() {

    // Define this programs arguments
    let matches = clap_app!(serv =>
        (version: "0.1")
        (author: "")
        (about: "A Web Server")
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
        root: path_abs,
        port: port.parse().unwrap(), // parse port value into an integer
        host: host.to_string(),
    };

    let routefile = config.root.join("routes.conf");
    // let route_table = route_table::RouteTable::from_file(&routefile);
    let route_table = route_table::RouteTable::default();

    println!("Generated the following Route Table:");
    println!("{}", route_table);

    let listen_addr = format!("{}:{}", &config.host, &config.port);

	let mut server = tide::with_state(State{route_table: route_table.table, config});

    // let server_instance = server::init(route_table);
    server.at("*route").get(handler);
    server.at("").get(handler);
    server.at("*route").post(handler);
    server.at("").post(handler);

    let result = server.listen(listen_addr).await;

    match result {
        Ok(_) => (),
        Err(e) => println!("Server Terminating: {}", e),
    };
}


type Request = tide::Request<State>;

#[derive(Clone, Debug)]
pub struct State{
    route_table: Vec<Route>,
    config: Config,
}

