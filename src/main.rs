use http;
use std::thread;
use std::net::{ TcpListener, TcpStream, Shutdown };
use std::io::{ Read, Write };

fn handle_client(mut stream: TcpStream){
	let mut data = [0 as u8; 50];
	while match stream.read(&mut data) {
		Ok(size) => {
			let s = match std::str::from_utf8(&data) {
				Ok(v) => v,
				Err(e) => panic!(),
			};
			println!("{}", s);
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
