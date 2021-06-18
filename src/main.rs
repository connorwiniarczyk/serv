#![allow(dead_code, unused_results, unused_must_use, unused_variables)]
use async_std;
use tide::Request;
use clap::clap_app;
use std::env::current_dir;
use std::path::Path;

mod resolver;
mod config;
mod route_table;
mod request;

use route_table::Route;
use resolver::Resolver;
use config::Config;

pub async fn handler(request: Request<State>) -> tide::Result {
    let state = request.state();

    // get the requested path by taking the route parameter and prepending /
    // let mut path = request.param("route").unwrap().to_string();
    let mut route = request.param("route").unwrap_or("").to_string();
    route = ["/", &route].join("");

    // look for a route in the route table that satisfies the request
    for Route { request, resolver } in &state.route_table {
        println!("{:#?}", request);
        if request == &route.as_str() {
            println!("{:#?}", resolver);
            
            let out = resolver.resolve(&state.config).await;
            return out
        }
    }

    panic!("failure")
}

#[derive(Clone, Debug)]
pub struct State{
    route_table: Vec<Route>,
    config: Config,
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

    println!("{:?}", path_abs);

    let config = Config {
        root: path_abs,
        port: port.parse().unwrap(), // parse port value into an integer
        host: host.to_string(),
    };

    let routefile = config.root.join("routes");
    let route_table = route_table::RouteTable::from_file(&routefile);

    let listen_addr = format!("{}:{}", &config.host, &config.port);

    println!("{:#?}", route_table.table);
	let mut server = tide::with_state(State{route_table: route_table.table, config});

    // let server_instance = server::init(route_table);
    server.at("*route").get(handler);
    server.at("").get(handler);
    server.listen(listen_addr).await;
}
