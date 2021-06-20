/// A Resolver is a type that can be turned into a tide HTTP response by
/// calling its resolve() method. Resolver types are defined as an enum

use tide::Response;
use async_process::Command;
use crate::config::Config;
use regex::Regex;
use lazy_static::lazy_static;

// define regular expressions for the module
lazy_static! {
   static ref OPTS: Regex = Regex::new(r"(?m:(?P<option>\w+)(?:\((?P<args>.*?)\))?)+").unwrap();
   static ref ARGS: Regex = Regex::new(r"(?P<arg>[a-zA-Z0-9]+)(?::(?P<value>[a-zA-Z0-9]+))?").unwrap();
}


#[derive(Debug, Clone)]
pub struct Arg {
    pub name: String,
    pub value: Option<String>,
}

impl Arg {
    pub fn new(name: &str, value: &str) -> Self {
        match value {
            "" => Self { name: name.to_string(), value: None },
            _  => Self { name: name.to_string(), value: Some(value.to_string()) },
        }
    }
}


#[derive(Debug, Clone)]
pub enum Access {
    Read,
    Exec(Vec<Arg>),
}

#[derive(Debug, Clone)]
pub struct Options {
    pub access_type: Access,
    pub post_processors: Vec<String>, 
}


impl Options {
    
    fn get_args(input: &str) -> Vec<Arg> {
        ARGS.captures_iter(input).map(|x| { 
            let name = x.name("arg").unwrap().as_str();
            let value = x.name("value").and_then(|x| Some(x.as_str()));
            Arg::new(&name, value.unwrap_or(""))
        }).collect()
    }

    pub fn from_str(input: &str) -> Self {

        let mut captures = OPTS.captures_iter(input); 

        // assume the first option given is the access_type
        let access_type_str = captures.next().unwrap();
        let option = access_type_str.name("option").and_then(|x| Some(x.as_str()));
        let args = access_type_str.name("args").and_then(|x| Some(x.as_str()));
        let access_type = match (option, args) {
            ( Some("exec"), Some(args)) => Access::Exec(Self::get_args(args)),
            ( Some("exec"), _) => Access::Exec(vec![]),
            ( _, _ ) => Access::Read,
        };

        let post_processors = vec![];

        Self { access_type, post_processors }
    }
}


pub async fn exec(file: &str, config: &Config) -> tide::Result {
    let path = config.root.as_path().join(&file);
    let output_raw = Command::new(path).output().await?;
    let output = std::str::from_utf8(&output_raw.stdout)?;

    let response = Response::builder(200)
        .body(output)
        .header("content-type", "text")
        .build();
    Ok(response)
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
       Options::from_str("exec(abcd) abcd(test)");
    }
}
