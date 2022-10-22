use async_trait::async_trait;
use crate::request_state::RequestState;

use std::fmt::Display;
use std::fmt;

use hyper::Body;

use std::sync::Arc;

use super::Cmd;


pub struct Write {
    value: String,
}

#[async_trait]
impl Cmd for Write {
    fn with_arg(arg: Option<&str>) -> Self where Self: Sized {
        return Self { value: arg.unwrap_or("").to_string() };
    }


    async fn run(&self, state: &mut RequestState) {
        // let value: Result<_, std::io::Error> = Ok(self.value.clone());

        state.body = self.value.clone().into()
        // let stream = futures_util::stream::once( async { value } );
        // state.body = Body::wrap_stream(stream);
    }
}

impl Display for Write {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}
