#![allow(dead_code, unused_results, unused_must_use, unused_variables)]

mod server;

use async_std;

use tide::Request;
use tide::Response;

/// Just brainstorming, in a list of routes, each route would need to keep
/// information about how it should be resolved. For example, text files,
/// Markdown files, and Executable files would all be treated differently.
// #[derive(Debug)]
// enum Resolver {
//     Text, Html, Markdown, Directory, Executable, Embedded
// }

#[derive(Debug, Clone)]
pub enum Resolver {
    Text(Path)
}

impl Resolver {
    async fn resolve(&self) -> tide::Result {
        let text = "imagine this is a text file".to_string();
        let response = Response::builder(200)
            .body(text)
            .header("content-type", "text")
            .build();
        Ok(response)
    }
}

#[derive(Debug, Clone)]
pub struct Path(String);

// impl PartialEq for Path {
//     fn eq(&self, other: &Self) -> bool {
        
//     }
// }

#[derive(Debug, Clone)]
pub struct Route(Path, Resolver);

impl Route {
    fn new(path: &str, resolver: Resolver) -> Self {
        Self(Path(path.to_string()), resolver)
    }
}

#[async_std::main]
async fn main() {
    println!("main");

    let mut route_table: Vec<Route> = vec![];

    route_table.push(Route::new("/", Resolver::Text(Path("/index.html".to_string()))));

    println!("{:?}", route_table);

    let server_instance = server::init(route_table);
    server_instance.listen("0.0.0.0:8080").await;
}
