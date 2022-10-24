use async_trait::async_trait;
use crate::request_state::RequestState;

use tokio::io::AsyncWriteExt;

use futures_util::FutureExt;

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

use tokio::process::Command;
use std::process::Stdio;

use futures_util::StreamExt;


pub struct Exec {
    program: String,
    args: Vec<String>,
}

#[async_trait]
impl Cmd for Exec {
    fn name(&self) -> &str { "exec" }
    fn arg(&self) -> &str { "" }

    fn with_arg(arg: Option<&str>) -> Self where Self: Sized {
        let mut arg_iter = arg.unwrap().split(" ");
        Self {
            program: arg_iter.next().unwrap().to_string(),
            args: arg_iter.map(|s| s.to_string()).collect(),
        }
    }

    async fn run(&self, state: &mut RequestState) {
        let mut process = Command::new(&self.program)
            .args(&self.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .unwrap();

        let mut stdout = process.stdout.take().unwrap();
        let mut stdin = process.stdin.take().unwrap();

        let mut input = std::mem::take(&mut state.body);

        let stream = stream::unfold(stdout, |mut stdout| async move {
            let mut buffer = BytesMut::with_capacity(1024);
            let bytes_read = stdout.read_buf(&mut buffer).await.unwrap();

            // if 0 bytes were read, we've reached the end of the file and should stop the
            // stream by returning None
            if bytes_read == 0 { return None }

            let result: Result<Bytes, std::io::Error> = Ok(buffer.freeze());
            return Some((result, stdout));
        });

        state.body = Body::wrap_stream(stream);

        state.register_task(async move { process.wait().await; });
        state.register_task(async move {
            match input.next().await {
                Some(Ok(value)) => { stdin.write_all(&value).await; },
                None => { stdin.shutdown().await; },
                _ => todo!(),
             };
        });
    }
}

impl Display for Exec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "exec {}", self.program)
    }
}

pub struct Shell(Exec);

#[async_trait]
impl Cmd for Shell {
    fn name(&self) -> &str { "shell" }
    fn arg(&self) -> &str { "" }

    fn with_arg(arg: Option<&str>) -> Self where Self: Sized {
        Self(Exec{
            program: "/bin/sh".to_string(),
            args: vec!["-c".to_string(), arg.unwrap().to_string()],
        })
    }

    async fn run(&self, state: &mut RequestState) {
        self.0.run(state).await;
    }
}

impl Display for Shell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "shell")
    }
}
