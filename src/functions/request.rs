use crate::{Scope, ServValue, ServResult, FnLabel, ServFunction};
use crate::cursor::Tokenizer;

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

fn query_all(input: ServValue, scope: &Scope) -> ServResult {
    let Some(req) = scope.get_request() else { return Ok(ServValue::None) };
    let Some(query) = req.uri.query() else { return Ok(ServValue::None) };
    let table = parse_query_string(&query);
    Ok(table)
}

fn query_get(input: ServValue, scope: &Scope) -> ServResult {
    todo!();
}

pub fn bind(scope: &mut Scope) {
	scope.insert(FnLabel::name("query"), ServFunction::Core(query_all));
}
