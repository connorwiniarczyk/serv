use tide::Request;
use tide::Response;

use super::*;

pub async fn test(request: Request<State>) -> tide::Result{
    let response = Response::builder(200)
        .body("test")
        .header("content-type", "text/html")
        .build();

    Ok(response)
}

pub async fn handler(request: Request<State>) -> tide::Result {

    let state = request.state();


    // get the requested path by taking the route parameter and prepending /
    // let mut path = request.param("route").unwrap().to_string();
    let mut path = request.param("route").unwrap_or("").to_string();
    path = ["/", &path].join("");

    // look for a route in the route table that satisfies the request
    for Route(route_path, resolver) in &state.route_table {
        println!("{:#?}", path);
        if route_path.0 == path {
            println!("{:#?}", route_path.0);
            
            let out = resolver.resolve().await;
            return out
        }
    }

    panic!("failure")
}

#[derive(Clone, Debug)]
pub struct State{
    route_table: Vec<Route>,
}

pub fn init(route_table: Vec<Route>) -> tide::Server<State> {
	let mut server = tide::with_state(State{route_table});


    // for route in route_table {
    //     let Route(Path(path), resolver) = route;

    // }

    server.at("*route").get(handler);
    server.at("").get(handler);
    return server
}
