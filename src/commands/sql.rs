use async_trait::async_trait;
use crate::request_state::RequestState;

use tokio::io::AsyncWriteExt;

use futures_util::FutureExt;

use std::fmt::Display;
use std::fmt;

use hyper::Body;

use std::sync::Arc;

use super::Cmd;
use std::path::PathBuf;
use tokio::fs::File;

use std::task::Poll;
use futures_util::stream::poll_fn;
use futures_util::stream;

use std::str::FromStr;

use bytes::BytesMut;
use bytes::Bytes;
use tokio::io::AsyncReadExt;

use tokio::process::Command;
use std::process::Stdio;

use futures_util::StreamExt;

use sqlite;
use sqlite::{Statement, Cursor};

use lazy_static::lazy_static;
use regex::{Regex, Captures};


pub struct Sql {
    statement: String,
    parameters: Vec<String>,
}

#[async_trait]
impl Cmd for Sql {
    fn name(&self) -> &str { "exec" }
    fn arg(&self) -> &str { &self.statement }

    fn with_arg(arg_opt: Option<&str>) -> Self where Self: Sized {
        lazy_static! {
            /// defines syntax for variables within an argument.
            /// syntax is based on Makefile variable syntax: ie. $(VAR)
            static ref VAR: Regex = Regex::new(r"(?P<precede>\$?)\$\((?P<name>.+?)\)").unwrap();
        }

        let arg = arg_opt.unwrap();
        let mut parameters = Vec::new();

        let statement = VAR.replace_all(arg, |caps: &Captures| {
            let name = caps.name("name").unwrap();
            parameters.push(name.as_str().to_string());
            "?".to_string()
        }).to_string();

        Self { statement, parameters }
    }

    async fn run(&self, state: &mut RequestState) {

        //TODO: custom database paths
        let connection = sqlite::open("serv.sqlite").unwrap();
        let mut query = connection.prepare(&self.statement).unwrap();

        for (index, param) in self.parameters.iter().map(|x| state.get_variable(x).unwrap()).enumerate() {
            query = query.bind(index + 1, param).expect("failed to bind sql parameter");
        }

        state.body = SqlResult::new(query).to_string().into();
    }
}

impl Display for Sql {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "sql")
    }
}


struct SqlResult {
    width: usize,
    column_names: Vec<String>,
    rows: Vec<Vec<String>>,
}

impl SqlResult {
    pub fn new(query: Statement) -> Self {

        let mut cursor = query.into_cursor();
        let width = cursor.column_count();
        let column_names = cursor.column_names().iter().map(|s| s.to_string()).collect();

        let mut output = Self { width, column_names, rows: vec![] };

        while let Some(Ok(row)) = cursor.next() {
            let mut output_row: Vec<String> = Vec::new();

            for i in 0..width {
                let val: sqlite::Value = row.get(i);
                output_row.push({
                    match val {
                        sqlite::Value::String(s) => format!("\"{}\"", s),
                        sqlite::Value::Float(d) => d.to_string(),
                        sqlite::Value::Integer(d) => d.to_string(),
                        sqlite::Value::Binary(bytes) => todo!(),
                        sqlite::Value::Null => "\"\"".to_string(),
                        _ => todo!(),
                    }
                })
            }
            output.rows.push(output_row);
        }
        output
    } 
}

impl Display for SqlResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[\n")?;

        let mut rows_iter = self.rows.iter().peekable();
        while let Some(row) = rows_iter.next() {
            let mut iter = row.iter().zip(self.column_names.iter()).peekable();
            f.write_str("{")?;
            while let Some((value, key)) = iter.next() {
                write!(f, "\"{}\": {}", key, value)?;
                if let Some(_) = iter.peek() { f.write_str(", ")?; }
            }
            f.write_str("}")?;
            if let Some(_) = rows_iter.peek() { f.write_str(",\n")?; }
        }

        f.write_str("]")?;


        Ok(())
    }
}
