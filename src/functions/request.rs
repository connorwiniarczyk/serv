use crate::ServValue;
use crate::ServResult;
use crate::Stack;
use crate::servparser;
use crate::{Label, ServFn};

use parsetool::cursor::Tokenizer;
use std::collections::HashMap;

fn parse_query_string(input: &str) -> ServValue {
    let mut output: HashMap<String, ServValue> = HashMap::new();

    let chars: Vec<char> = input.chars().collect();
    let mut cursor = Tokenizer::new(&chars);

    while !cursor.is_done() {
        cursor.incr_while(|c| c != '=');
        let key = cursor.emit(()).to_string();
        cursor.skip();
        cursor.incr_while(|c| c != '&');
        let value = cursor.emit(()).to_string();
        cursor.skip();
        output.insert(key, ServValue::Text(value));
    }

    ServValue::Table(output)
}

fn headers(input: ServValue, scope: &Stack) -> ServResult {
    let Some(req) = scope.get_request() else { return Ok(ServValue::None) };

    println!("{:#?}", req.headers);
    todo!();
}

fn query_all(input: ServValue, scope: &Stack) -> ServResult {
    let Some(req) = scope.get_request() else { return Ok(ServValue::None) };
    let Some(query) = req.uri.query() else { return Ok(ServValue::None) };
    let table = parse_query_string(&query);
    Ok(table)
}

fn query_get(input: ServValue, scope: &Stack) -> ServResult {
    todo!();
}

pub fn bind(scope: &mut Stack) {
	scope.insert_name("query", ServValue::Func(ServFn::Core(query_all)));
	scope.insert_name("req.headers", ServValue::Func(ServFn::Core(headers)));
}
