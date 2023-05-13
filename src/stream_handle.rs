use rhai::Engine;

use futures_util::{stream, Stream};
use bytes::{Bytes, BytesMut};

use std::sync::Arc;
use std::fs::File;
use std::io::{Read};

// use tokio::io::{AsyncRead, AsyncBufRead};

use tokio::process::Command;
use std::process::Stdio;

use tokio::io::AsyncReadExt;

/// Represents an asynchronous stream that can be manipulated by the rhai script
/// The Clone trait is necessary for it to be used as an rhai type, which is why
/// the stream is wrapped in an Arc pointer.
#[derive(Clone)]
pub struct StreamHandle {
	pub inner: Arc<hyper::Body>,
}

impl StreamHandle {
	pub fn empty() -> Self {
		let body = hyper::Body::empty();
		Self { inner: Arc::new(body) }
	}

	pub fn from_file(path: &str) -> Self {
		let mut file = File::open(path).unwrap();
		let metadata = file.metadata().unwrap();

		let mut buf = String::new();
		file.read_to_string(&mut buf);
		println!("{:?}", buf);

		todo!();

		// if the file is over a certain size, build a Stream that read and outputs the file in
		// buffered increments
		let stream = stream::unfold(file, |mut file| async move {
			let mut buffer = BytesMut::with_capacity(1024);

			let bytes_read: usize = file.read(&mut buffer).unwrap();

			println!("{:?}", buffer);

			// if 0 bytes were read, we've reached the end of the file and should stop the
			// stream by returning None
			if bytes_read == 0 { return None }

			let result: Result<Bytes, hyper::Error> = Ok(buffer.freeze());
			// let result = buffer.freeze();
			return Some((result, file));
		});

		let body = hyper::Body::wrap_stream(stream);
		return Self { inner: Arc::new(body) }
	}

	pub fn exec(path: &str) -> Self {

		let mut process = Command::new(path)
			.stdin(Stdio::piped())
			.stdout(Stdio::piped())
			.kill_on_drop(true)
			.spawn()
			.unwrap();

		let stdout = process.stdout.take().unwrap();

		let stream = stream::unfold((stdout, process), |(mut stdout, mut process)| async move {
			let mut buffer = BytesMut::with_capacity(1024);
			let bytes_read = stdout.read_buf(&mut buffer).await.unwrap();

			if process.try_wait().is_ok() { return None }

			let result: Result<Bytes, std::io::Error> = Ok(buffer.freeze());
			return Some((result, (stdout, process)))
		});

		let body = hyper::Body::wrap_stream(stream);
		return Self { inner: Arc::new(body) }

	}
}
