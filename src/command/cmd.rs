use async_trait::async_trait;
use crate::request_state::RequestState;

use std::fmt::Display;
use std::fmt;

use hyper::Body;

use std::sync::Arc;

#[async_trait]
pub trait Cmd: Display + Send + Sync {
    async fn run(&self, state: &mut RequestState); 
    fn with_arg(arg: Option<&str>) -> Self where Self: Sized;

    fn wrap(self) -> Arc<dyn Cmd> where Self: Sized + 'static {
        Arc::new(self)
    }


}

