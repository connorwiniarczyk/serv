use async_trait::async_trait;
use crate::request_state::RequestState;

use std::fmt::Display;
use std::fmt;

use hyper::Body;

use std::sync::Arc;

use super::Cmd;


pub struct Echo {
    value: String,
}

#[async_trait]
impl Cmd for Echo {
    fn with_arg(arg: Option<&str>) -> Self where Self: Sized {
        return Self { value: arg.unwrap_or("").to_string() };
    }

    async fn run(&self, state: &mut RequestState) {
        let value = Self::substitute_vars(&self.value, &state);
        state.body = value.into();
    }
}

impl Display for Echo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

pub struct SetVar {
    key: String,
    value: String,
}

#[async_trait]
impl Cmd for SetVar {
    fn with_arg(arg: Option<&str>) -> Self where Self: Sized {
        let mut iter = arg.unwrap().split(' ');
        Self {
            key: iter.next().unwrap().to_string(),
            value: iter.next().unwrap().to_string(),
        }
    }

    async fn run(&self, state: &mut RequestState) {
        let value = Self::substitute_vars(&self.value, &state);
        state.set_variable(&self.key, &value);
    }
}

impl Display for SetVar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "set {} {}", self.key, self.value)
    }
}
