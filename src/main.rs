#![allow(dead_code, unused_results, unused_must_use, unused_variables)]
use async_std;
use tide::Request;
use tide::Response;

mod resolver;
mod config;
mod route_table;

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
    for Route { path, resolver } in &state.route_table {
        println!("{:#?}", path);
        if path == &route {
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

    let path = "/home/connor/projects/serv/public";
    let path_absolute = std::fs::canonicalize(path).unwrap();

    let config = Config {
        root: path_absolute,
        port: 8080,
    };

    let mut routefile = config.root.clone();
    routefile.push("routes");

    // let mut route_table = route_table::RouteTable::with_root(&config.root);
    let route_table = route_table::RouteTable::from_file(&routefile);

    let listen_addr = format!("0.0.0.0:{}", &config.port);

    println!("{:#?}", route_table.table);
	let mut server = tide::with_state(State{route_table: route_table.table, config});

    // let server_instance = server::init(route_table);
    server.at("*route").get(handler);
    server.at("").get(handler);
    server.listen(listen_addr).await;
}
