#![allow(dead_code, unused_results, unused_must_use, unused_variables)]
use async_std;
use tide::Request;
use clap::clap_app;
use std::env::current_dir;
use std::path::Path;
use async_process::Command;

mod options;
mod config;
mod route_table;
mod path_expression;
mod error;
mod parser;

use route_table::Route;
use config::Config;

use tide::Response;
use options::Access;
use std::fs;
use options::Arg;

use tide::http::Url;
use std::collections::HashMap;

pub struct HttpQuery {
    inner: HashMap<String, String>,
    // inner: Vec<(String, String)>,

}

impl HttpQuery {
    pub fn from_url(url: &Url) -> Self {

        let mut output: HashMap<String, String> = HashMap::new();
        let pairs = url.query_pairs();
        for ( left, right ) in pairs {
            output.insert(left.into_owned(), right.into_owned());
        }
        Self { inner: output }
    }

    pub fn get(&self, key: &str) -> Option<&str> {
       self.inner.get(key).and_then(|x| Some(x.as_str()))
    }
}

pub async fn handler(http_request: Request<State>) -> tide::Result {
    let state = http_request.state();

    // get the requested path by taking the route parameter and prepending /
    // let mut path = request.param("route").unwrap().to_string();
    let mut route = http_request.param("route").unwrap_or("").to_string();
    route = ["/", &route].join("");

    println!("{:?}", route);

    for route in state.route_table.iter() {
        if let Some(result) = route.resolve(&http_request).await {
            let mut output = Response::builder(200).body(result.body);

            //TODO: this is gross, also the status method can't accept a u32 for some reason
            output = result.headers.iter().fold(output, |acc, (key, value)| acc.header(key.as_str(), value.as_str()));
            return Ok(output.build())
        }
    }

    // TODO: boy I'd like to replace the above for loop with a call to find_map, but the asyncness
    // of it makes it tricky. I think I need to use a stream, but I'm not sure how to turn my
    // Iterator of Futures into a Stream properly
    // &state.route_table.iter().map(|x| x.resolve(&http_request)).find_map(|x| x.await);

    let response = Response::builder(400)
        .body("not found")
        .build();

    return Ok(response)
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

    // It is important to cd into the target directory so that shell scripts invoked in that
    // directory will know what directory they are being run from
    std::env::set_current_dir(&path_abs).expect("could not cd into that directory!");

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
