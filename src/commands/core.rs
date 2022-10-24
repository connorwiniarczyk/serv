use async_trait::async_trait;
use crate::request_state::RequestState;

use std::fmt::Display;
use std::fmt;

use hyper::Body;

use std::sync::Arc;

use super::Cmd;

use hyper::body::HttpBody;


pub struct Echo {
    value: String,
}

#[async_trait]
impl Cmd for Echo {
    fn name(&self) -> &str { "echo" }
    fn arg(&self) -> &str { &self.value }

    fn with_arg(arg: Option<&str>) -> Self where Self: Sized {
        return Self { value: arg.unwrap_or("").to_string() };
    }

    async fn run(&self, state: &mut RequestState) {
        let value = Self::substitute_vars(&self.value, &state);
        state.body = value.into();
    }
}

pub struct SetVar {
    key: String,
    value: String,
}

#[async_trait]
impl Cmd for SetVar {
    fn name(&self) -> &str { "set" }
    fn arg(&self) -> &str { &self.value }

    fn with_arg(arg: Option<&str>) -> Self where Self: Sized {
        let mut iter = arg.unwrap().split(' ');
        Self {
            key: iter.next().unwrap().to_string(),
            value: iter.next().unwrap().to_string(),
        }
    }

    async fn run(&self, state: &mut RequestState) {
        let value = Self::substitute_vars(&self.value, &state);

        if self.key.as_str().contains("header.") {
            let key = self.key.strip_prefix("header.").unwrap();
            state.headers.insert(key.to_string(), self.value.to_string());
        }

        match self.key.as_str() {
            "mimetype" => state.mime = Some(value),
            key => state.set_variable(key, &value),
        }
    }
}

use json::JsonValue;

/// Parses the Body into distinct variables that can be used by later commands
pub struct ParseBody;

#[async_trait]
impl Cmd for ParseBody {
    fn name(&self) -> &str { "parse body" }
    fn arg(&self) -> &str { "" }

    fn with_arg(arg: Option<&str>) -> Self where Self: Sized {
        Self
    }

    async fn run(&self, state: &mut RequestState) {

        // instruct the state to wait for all of its tasks to complete so that the body can be
        // complete when it is parsed.
        state.wait().await;

        let mut body = std::mem::take(&mut state.body);
        let bytes = hyper::body::to_bytes(body).await.unwrap();
        let json = json::parse(std::str::from_utf8(&bytes).unwrap()).unwrap();       

        fn insert_value(parent_prefix: &str, value: JsonValue, state: &mut RequestState) {
            match value {
                JsonValue::Short(value) => state.variables.insert(format!("{}", parent_prefix), value.as_str().to_string()),
                JsonValue::String(value) => state.variables.insert(format!("{}", parent_prefix), value.as_str().to_string()),
                JsonValue::Number(value) => {
                    let number: f32 = value.clone().into();
                    state.variables.insert(format!("{}", parent_prefix), number.to_string())
                },
                JsonValue::Object(ref _object) => {
                    for (key, value) in value.entries() {
                        insert_value(&format!("{}.{}", parent_prefix, key), value.clone(), state);
                    }
                    None
                },

                JsonValue::Array(ref _array) => {
                    for (key, value) in value.members().enumerate() {
                        insert_value(&format!("{}.{}", parent_prefix, key), value.clone(), state);
                    }
                    None
                },

                _ => todo!(),
            };
        }

        insert_value("body", json, state);
    }
}


pub struct Jump {
    route: String,
}

#[async_trait]
impl Cmd for Jump {
    fn name(&self) -> &str { "jump" }
    fn arg(&self) -> &str { &self.route }

    fn with_arg(arg: Option<&str>) -> Self where Self: Sized {
        Self { route: arg.unwrap().to_string() }
    }

    async fn run(&self, state: &mut RequestState) {
        let route = state.table.get(&self.route).expect("that route does not exist");
        
        for command in &route.commands {
            command.run(state).await;
        }
    }
}

