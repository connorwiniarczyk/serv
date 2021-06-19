/// A Resolver is a type that can be turned into a tide HTTP response by
/// calling its resolve() method. Resolver types are defined as an enum

use tide::Response;
use std::fs;
use async_process::Command;

use crate::config::Config;
use crate::path_expression::*;

use std::path::{PathBuf,Path};


// pub enum PreProcessor {
//     Exec (  )
// }

// pub struct Resource {
//     path: PathExpr,

// }

#[derive(Debug, Clone)]
pub enum Resolver {
    File { path: PathExpr },
    Exec { path: String },
}

pub fn resolve_file(file: &Path, config: &Config) -> tide::Result {
    let path = config.root.as_path().join(&file.strip_prefix("/").unwrap());
    println!("resolve path {:?}", path);
    let text = fs::read_to_string(path)?;
    println!("{}", text);
    
    let response = Response::builder(200)
        .body(text)
        .header("content-type", "text")
        .build();
    Ok(response)
}

pub async fn resolve_exec(file: &str, config: &Config) -> tide::Result {
    let path = config.root.as_path().join(&file);
    let output_raw = Command::new(path).output().await?;
    let output = std::str::from_utf8(&output_raw.stdout)?;

    let response = Response::builder(200)
        .body(output)
        .header("content-type", "text")
        .build();
    Ok(response)
}

impl Resolver {
    pub async fn resolve(&self, path_match: PathMatch, config: &Config) -> tide::Result {
       match self {
            Self::File { path } => {
                let resolved_path = path_match.to_path(&path);
                resolve_file(&resolved_path, config)
            },
            Self::Exec { path } => resolve_exec(path, config).await,
        }
    }

    pub fn file(path: &str) -> Self {
        Self::File { path: PathExpr::new(path) }
    }

    pub fn exec(path: &str) -> Self {
        Self::Exec { path: path.to_string() }
    }
}

