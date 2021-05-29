/// A Resolver is a type that can be turned into a tide HTTP response by
/// calling its resolve() method. Resolver types are defined as an enum

use tide::Request;
use tide::Response;
use std::fs;

#[derive(Debug, Clone)]
pub enum Resolver {
    File { path: String }
}

pub fn resolve_file(path: &str) -> tide::Result {
    let text = fs::read_to_string(path)?;
    let response = Response::builder(200)
        .body(text)
        .header("content-type", "text")
        .build();
    Ok(response)
}

impl Resolver {
    pub async fn resolve(&self) -> tide::Result {
        match self {
            Self::File { path } => resolve_file(path),
        }
    }
}

