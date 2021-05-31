/// parses the routes file

use super::*;

use std::fs::File;
use std::io::{BufRead, BufReader};

use regex::Regex;
use lazy_static::lazy_static;

use super::Route;

fn parseline(line: &str, index: usize) -> Option<Route> {

    lazy_static! {
       static ref COMMENT: Regex = Regex::new(r"#.*").unwrap();
       static ref LINE: Regex = Regex::new(r"([^\s]+)\s+([^\s]+)").unwrap();
    }
     
    // remove comments
    let stripped_line = COMMENT.replace(line, "");

    if let Some(captures) = LINE.captures(&stripped_line) {

        let output = Route::new(&captures[1], Resolver::file(&captures[2]));
        println!("{:?}", output);
        
        return Some(output)
    } else {
        return None
    }
}


pub fn parse(path: &str) -> Vec<Route> {

    let file = File::open(path).unwrap(); 
    let reader = BufReader::new(file);

    for (index, line) in reader.lines().enumerate() {
        let line = line.unwrap();
        // println!("{}. {}", index, line);
        parseline(&line, index);
    }


    println!("");


    let mut out: Vec<Route> = vec![];
    out
}
