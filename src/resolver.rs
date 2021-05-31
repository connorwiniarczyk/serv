/// A Resolver is a type that can be turned into a tide HTTP response by
/// calling its resolve() method. Resolver types are defined as an enum

use tide::Request;
use tide::Response;
use std::fs;
use async_process::Command;

use crate::config::Config;

#[derive(Debug, Clone)]
pub enum Resolver {
    File { path: String },
    Exec { path: String },
    // Dir  { path: String },
}
pub fn resolve_file(file: &str, config: &Config ) -> tide::Result {
    let path = format!("{}/{}", config.root, file);
    let text = fs::read_to_string(&path)?;
    let response = Response::builder(200)
        .body(text)
        .header("content-type", "text")
        .build();
    Ok(response)
}

pub async fn resolve_exec(file: &str, args: &Vec<String>, config: &Config) -> tide::Result {
    let path = format!("{}/{}", config.root, file);
    let output_raw = Command::new(path).args(args).output().await?;
    let output = std::str::from_utf8(&output_raw.stdout)?;

    let response = Response::builder(200)
        .body(output)
        .header("content-type", "text")
        .build();
    Ok(response)
}

// Resolving full directories is complicated, I should brainstorm more before deciding on an
// implementation
// pub fn resolve_dir(path: &str, config: &Config) -> tide::Result {
//     resolve_file(path, config)
// }

impl Resolver {
    pub async fn resolve(&self, config: &Config) -> tide::Result {
        match self {
            Self::File { path } => resolve_file(path, config),
            Self::Exec { path } => resolve_exec(path, &vec![], config).await,
            // Self::Dir  { path } => resolve_dir(path, config),
        }
    }

    pub fn file(path: &str) -> Self {
        Self::File{ path: path.to_string() }
    }

    pub fn exec(path: &str) -> Self {
        Self::Exec{ path: path.to_string() }
    }
}

