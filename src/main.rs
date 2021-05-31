#![allow(dead_code, unused_results, unused_must_use, unused_variables)]
use async_std;
use tide::Request;
use tide::Response;

mod resolver;
mod config;
mod routes_parser;

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
            println!("{:#?}", path);
            
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

#[derive(Clone)]
pub struct Route{
    path: String,
    resolver: Resolver,
}

impl Route {
    fn new(path: &str, resolver: Resolver) -> Self {
        Self{ path: path.to_string(), resolver }
    }
}

use std::fmt;

impl fmt::Debug for Route {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let resolver_desc = match &self.resolver {
            Resolver::File{ path } => format!("file: {}", path),
            Resolver::Exec{ path } => format!("exec: *{}", path),
            _ => "other".to_string(),
        };

        write!(f, "{:<8} {:^5} {}", self.path, "-->", resolver_desc)
    }
}

impl fmt::Display for Route {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let resolver_desc = match &self.resolver {
            Resolver::File{ path } => "file",
            Resolver::Exec{ path } => "exec",
            _ => "other",
        };

        write!(f, "({} --> {})", self.path, resolver_desc)
    }
}

#[async_std::main]
async fn main() {

    let config = Config {
        root: "/home/connor/projects/serv/public".to_string(),
        port: 8080,
    };

    let routefile = format!("{}/{}", &config.root, "routes");
    let mut route_table = routes_parser::parse(&routefile);

    // add extra routes manually
    route_table.push(Route::new("/",        Resolver::file("index.html")));
    route_table.push(Route::new("/shell",   Resolver::exec("shell_example")));
    route_table.push(Route::new("/date",    Resolver::exec("date")));

    let listen_addr = format!("0.0.0.0:{}", &config.port);

    println!("{:#?}", route_table);
	let mut server = tide::with_state(State{route_table, config});

    // let server_instance = server::init(route_table);
    server.at("*route").get(handler);
    server.at("").get(handler);
    server.listen(listen_addr).await;
}
