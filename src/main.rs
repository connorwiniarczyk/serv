use http;
use std::thread;
use std::net::{ TcpListener, TcpStream, Shutdown };
use std::io::{ Read, Write };

use std::collections::HashMap;

struct HttpResponse {
	pub status: u16,
	pub reason_phrase: String,
	pub headers: HashMap<String, String>,
	pub body: String,
}

impl HttpResponse {
	fn to_string(&self) -> String {
		let header_string = "";
		let output = format!(
			"HTTP/1.1 {} {}\r\n{}\r\n{}",
			&self.status, &self.reason_phrase,
			header_string,
			&self.body,
		);

		output
	}
}

fn handle_client(mut stream: TcpStream){
	
	let response = HttpResponse{
		status: 200,
		reason_phrase: "OK".to_string(),
		headers: HashMap::new(),
		body: "this is some text".to_string(),
	};

	let mut data = [0 as u8; 1024];
	while match stream.read(&mut data) {
		Ok(size) => {
			let s = match std::str::from_utf8(&data) {
				Ok(v) => v,
				Err(e) => panic!(),
			};
			println!("{}", response.to_string());

			stream.write(response.to_string().as_bytes()).unwrap();
			// stream.write("HTTP/1.1 200 OK\r\n\r\n".as_bytes()).unwrap();
			stream.flush().unwrap();
			// stream.write(&data[0..size]).unwrap();
			// stream.write(&data[0..size]).unwrap();
			true
		},
		Err(_) => false,
	} {}
}

fn main() {
	let listener = TcpListener::bind("0.0.0.0:3333").unwrap();
	println!("Server listening on port 3333");
	for stream in listener.incoming() {
		match stream {
			Ok(stream) => {
				println!("new connection: {}", stream.peer_addr().unwrap());
				thread::spawn(move || {handle_client(stream)});
			}
			Err(e) => {
				println!("error");
			}
		}
	}
	drop(listener)
}
