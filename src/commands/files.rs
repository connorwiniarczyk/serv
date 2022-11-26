use async_trait::async_trait;
use crate::request_state::RequestState;

use multer::Multipart;

use std::fmt::Display;
use std::fmt;

use hyper::Body;

use std::sync::Arc;

use super::Cmd;
use std::path::PathBuf;
use tokio::fs::File;

use std::task::Poll;
use futures_util::stream::poll_fn;
use futures_util::stream;

use std::str::FromStr;

use bytes::BytesMut;
use bytes::Bytes;
use tokio::io::AsyncReadExt;

pub struct ReadFile {
    path: String,
}

#[async_trait]
impl Cmd for ReadFile {
    fn name(&self) -> &str { "read" }
    fn arg(&self) -> &str { self.path.as_str() }

    fn with_arg(arg: Option<&str>) -> Self where Self: Sized {
        return Self {
            path: arg.unwrap().to_string()
            // path: PathBuf::from_str(arg.unwrap()).unwrap()
        };
    }

    async fn run(&self, state: &mut RequestState) {

        let path = Self::substitute_vars(&self.path, &state);
        let mut file = File::open(path).await.unwrap();
        let metadata = file.metadata().await.unwrap();

        // if the file length is under a certain size, just read the whole thing into memory and
        // write it out all at once. Not sure if I'll keep this or if so what the ideal size would
        // be
        if metadata.len() < 1000 * 10 {
            let mut buffer: Vec<u8> = vec![];
            file.read_to_end(&mut buffer).await;
            state.body = buffer.into();
        }

        // if the file is over a certain size, build a Stream that read and outputs the file in
        // buffered increments
        else {
            let stream = stream::unfold(file, |mut file| async move {
                let mut buffer = BytesMut::with_capacity(1024);
                let bytes_read = file.read_buf(&mut buffer).await.unwrap();

                // if 0 bytes were read, we've reached the end of the file and should stop the
                // stream by returning None
                if bytes_read == 0 { return None }

                let result: Result<Bytes, std::io::Error> = Ok(buffer.freeze());
                return Some((result, file));
            });

            state.body = Body::wrap_stream(stream);
        }
    }
}

impl Display for ReadFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // write!(f, "read {}", self.path.to_str().unwrap())
        write!(f, "read {}", self.path)
    }
}


pub struct WriteFile {
    path: String,
}

use futures_util::StreamExt;
use tokio::io::{AsyncWrite, AsyncWriteExt};
use std::path::Path;

#[async_trait]
impl Cmd for WriteFile {
    fn name(&self) -> &str { "write" }
    fn arg(&self) -> &str { self.path.as_str() }

    fn with_arg(arg: Option<&str>) -> Self where Self: Sized {
        return Self {
            path: arg.unwrap().to_string()
        };
    }

    async fn run(&self, state: &mut RequestState) {
        let path = Self::substitute_vars(&self.path, &state);

        let boundary = &state.parts.headers
            .get(hyper::header::CONTENT_TYPE)
            .and_then(|ct| ct.to_str().ok())
            .and_then(|ct| multer::parse_boundary(ct).ok())
            .unwrap();

        let mut body = std::mem::take(&mut state.body);
        let mut multipart = Multipart::new(body, boundary);

        // open file for writing
        let mut file = File::create(&path).await.unwrap();

        state.register_task(async move {

            while let Some(mut field) = multipart.next_field().await.unwrap() {
                // Process the field data chunks e.g. store them in a file.
                let mut field_bytes_len = 0;
                while let Some(field_chunk) = field.chunk().await.unwrap() {
                    file.write_all(&field_chunk).await;
                }
            }
        });
    }
}

impl Display for WriteFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "write {}", self.path)
    }
}
