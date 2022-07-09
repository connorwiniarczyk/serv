// #![deny(warnings)e
// #![allow(unused_imports, unused_mut, unused_doc_comments, unused_macros, dead_code, unused_results, unused_must_use, unused_variables)]
#![allow(warnings)]

mod config;
mod route_table;
mod pattern;
mod parser;

mod command;
mod request_state;

use route_table::Route;
use config::Config;
// use tide::Response;

// use async_std;
use clap::clap_app;
use std::env::current_dir;
use std::path::Path;
// use tide;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};

pub async fn handler(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    println!("incoming http request: {}", req.uri());

    // let state = http_request.state().clone();
    // let route = http_request.param("route").unwrap_or("").to_string();

    // // get the body if there is one
    // let body = http_request.body_bytes().await.ok();

    // for route in state.route_table.iter() {
    //     if let Ok(result) = route.resolve(&mut http_request, &body).await {
    //         println!();
    //         return Ok(result)
    //     }
    // }

    // println!("\t failed to find a matching route, serving 404 page instead");
    // let response = Response::builder(404).body("not found").build();
    // return Ok(response)

    // todo!();


    Ok(Response::new(Body::from("test")))

}


/// This is our service handler. It receives a Request, routes on its
/// path, and returns a Future of a Response.
//async fn echo(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
//    match (req.method(), req.uri().path()) {
//        // Serve some instructions at /
//        (&Method::GET, "/") => Ok(Response::new(Body::from(
//            "Try POSTing data to /echo such as: `curl localhost:3000/echo -XPOST -d 'hello world'`",
//        ))),

//        // Simply echo the body back to the client.
//        (&Method::POST, "/echo") => Ok(Response::new(req.into_body())),

//        // TODO: Fix this, broken in PR #2896
//        // Convert to uppercase before sending back to client using a stream.
//        // (&Method::POST, "/echo/uppercase") => {
//        // let chunk_stream = req.into_body().map_ok(|chunk| {
//        //     chunk
//        //         .iter()
//        //         .map(|byte| byte.to_ascii_uppercase())
//        //         .collect::<Vec<u8>>()
//        // });
//        // Ok(Response::new(Body::wrap_stream(chunk_stream)))
//        // }

//        // Reverse the entire body before sending back to the client.
//        //
//        // Since we don't know the end yet, we can't simply stream
//        // the chunks as they arrive as we did with the above uppercase endpoint.
//        // So here we do `.await` on the future, waiting on concatenating the full body,
//        // then afterwards the content can be reversed. Only then can we return a `Response`.
//        (&Method::POST, "/echo/reversed") => {
//            let whole_body = hyper::body::to_bytes(req.into_body()).await?;

//            let reversed_body = whole_body.iter().rev().cloned().collect::<Vec<u8>>();
//            Ok(Response::new(Body::from(reversed_body)))
//        }

//        // Return the 404 Not Found for other routes.
//        _ => {
//            let mut not_found = Response::default();
//            *not_found.status_mut() = StatusCode::NOT_FOUND;
//            Ok(not_found)
//        }
//    }
//}

use std::sync::Arc;

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
//     let addr = ([127, 0, 0, 1], 3000).into();

//     // let state = Arc::new("abcd");
//     let state = "abcd";

//     let service = make_service_fn(|_| async move {
//         Ok::<_, hyper::Error>(service_fn(move |req: Request<Body>| async move {
//             let output = state.clone();
//             Ok::<_, hyper::Error>(Response::new(Body::from(output)))
//         }))
//     });

//     let server = Server::bind(&addr).serve(service);

//     println!("Listening on http://{}", addr);

//     server.await?;

//     Ok(())
// }


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
        root: current_dir().unwrap().join(Path::new("../examples/blog")),
        port: port.parse().unwrap(), // parse port value into an integer
        host: host.to_string(),
    };

    let routefile = config.root.join("routes.conf");
    let route_table = Arc::new(
        route_table::RouteTable::from_file(&routefile)
    );

    println!("Generated the following Route Table:");
    println!("{}", route_table);

    let listen_addr = ([127, 0, 0, 1], 3000).into();

    let route_table = route_table.clone();
    let service = make_service_fn(move |_| {

        let route_table = route_table.clone();
        async move {
            Ok::<_, hyper::Error>(service_fn(move |req: Request<Body>| {
                let route_table = route_table.clone();
                async move {
                    // println!("{}", route_table);
                    route_table.resolve(req).await
                    // Ok::<_, hyper::Error>(Response::new(Body::from("test")))
                    //
                }
            }))
        }
    });

    let server = Server::bind(&listen_addr).serve(service).await;

	// let mut server = tide::with_state(State{route_table: route_table.table, config});

    // // let server_instance = server::init(route_table);
    // server.at("*route").get(handler);
    // server.at("").get(handler);
    // server.at("*route").post(handler);
    // server.at("").post(handler);

    // let result = server.listen(listen_addr).await;

    // match result {
        // Ok(_) => (),
        // Err(e) => println!("Server Terminating: {}", e),
    // };
}


#[derive(Clone, Debug)]
pub struct State{
    route_table: Vec<Route>,
    config: Config,
}

