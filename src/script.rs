use rhai::Engine;

use futures_util::{stream, Stream};
use bytes::{Bytes, BytesMut};

use std::sync::Arc;
use std::fs::File;
use std::io::{Read, BufRead};


pub fn create_engine() -> rhai::Engine {
	let mut engine = Engine::new();
	engine.register_type_with_name::<StreamHandle>("Stream");
	engine.register_fn("empty", StreamHandle::empty);
	engine.register_fn("file", StreamHandle::from_file);

	return engine
}

#[cfg(test)]
mod test {
	use super::*;


	#[test]
	fn test1() {
		let engine = create_engine();
		let res: StreamHandle = engine.eval("empty()").unwrap();

	}
}

		// let vars = self.pattern.compare(&request).unwrap();
		// let script = self.script.clone();
		// let output = State::new().arc();

        // let (sender, body) = Body::channel();
        // let sender_mux = Arc::new(Mutex::new(sender));

		// let worker = std::thread::spawn(move || {
            // let output = output.clone();
		// 	let mut engine = Engine::new();
		// 	let ast = engine.compile(script).unwrap();

		// 	let mut scope = Scope::new();
		// 	for (key, value) in vars {
		// 		scope.push(key, value);	
		// 	}

            // {
                // let sender_mux = sender_mux.clone();
                // let output = output.clone();
                // engine.register_fn("read", move |path: &str| {
                    // let mut file = File::open(path).unwrap();
                    // let mut buffer = [0u8; 1024];
                    // loop {
                        // let n = file.read(&mut buffer).unwrap();
                        // if n == 0 { break }

                        // let lock = &mut sender_mux.lock().unwrap();
                        // lock.send_data(Bytes::copy_from_slice(&buffer[0..n])).block_on();
                    // }
                // });
            // }

            // {
                // let sender_mux = sender_mux.clone();
                // engine.register_fn("echo", move |x: &str| {
                    // sender_mux.lock().unwrap().send_data(Bytes::from(x.to_owned())).block_on();

                    // // let next_element: Result<Bytes, hyper::Error> = Ok(Bytes::from(x.to_owned()));
                    // // let next_element_str = stream::once(async {next_element});

                    // // let current_body = &mut output_cl.0.lock().unwrap().body;
                    // // let current_stream = std::mem::take(current_body);

                    // // let sum = current_stream.chain(next_element_str);	
                    // // *current_body = Body::wrap_stream(sum);
                // });

            // }

		// 	engine.run_ast_with_scope(&mut scope, &ast).unwrap();
		// });

        // // tokio::spawn(state)

		// // worker.join().unwrap();

		// // let body = std::mem::take(&mut output.0.lock().unwrap().body);
		// // let body = Body::from(output.0.lock().unwrap().test.clone());
		// let mut out = hyper::Response::builder().status(200);
		// Ok(out.body(body).unwrap())
