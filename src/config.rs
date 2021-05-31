/// Config

use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub root: PathBuf,    
    pub port: u32,
}
