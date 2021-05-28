use tide::Request;
use tide::Response;

use async_std;

pub async fn test(request: Request<()>) -> tide::Result{
    let response = Response::builder(200)
        .body("test")
        .header("content-type", "text/html")
        .build();

    Ok(response)
}

pub fn init() -> tide::Server<()> {
	let mut server = tide::new();
    server.at("/").get(test);
    return server
}

#[async_std::main]
async fn main() {
    println!("main");
    let server = init();
    server.listen("0.0.0.0:8080").await;
}
