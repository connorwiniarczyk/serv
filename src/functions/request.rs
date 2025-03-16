use crate::ServValue;
use crate::ServResult;
use crate::ServError;
use crate::Stack;
use crate::servparser;
use crate::datatypes::servlist::ServList;
use crate::{Label, ServFn};

use crate::ServModule;

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
        output.insert(key, value.into());
    }

    ServValue::Table(output)
}

fn parse_cookie(input: &str) -> ServValue {
    let mut output: HashMap<String, ServValue> = HashMap::new();

    let chars: Vec<char> = input.chars().collect();
    let mut cursor = Tokenizer::new(&chars);

    while !cursor.is_done() {
        cursor.incr_while(|c| c != '=');
        let key = cursor.emit(()).to_string().trim().to_string();
        cursor.skip();
        cursor.incr_while(|c| c != ';');
        let value = cursor.emit(()).to_string().trim().to_string();
        cursor.skip();
        output.insert(key, value.into());
    }

    ServValue::Table(output)
}

fn query_all(input: ServValue, scope: &Stack) -> ServResult {
    let Some(req) = scope.get_request() else { return Ok(ServValue::None) };
    let Some(query) = req.uri.query() else { return Ok(ServValue::None) };
    let table = parse_query_string(&query);
    Ok(table)
}

fn get_cookies(input: ServValue, scope: &Stack) -> ServResult {
    let Some(req) = scope.get_request() else { return Ok(ServValue::None) };
	let cookie = req.headers.get("Cookie").ok_or(ServError::new(500, "expected a cookie"))?;

	Ok(parse_cookie(cookie.to_str().unwrap()))
}

fn set_cookie(mut input: ServList, scope: &mut Stack) -> ServResult {
    let mut arg = input.pop()?;
	arg = arg.call(None, scope)?;

    scope.insert("res.cookie", arg);
    input.eval(scope)
}

fn with_headers(mut input: ServList, scope: &mut Stack) -> ServResult {
    let mut arg = input.pop()?;
	arg = arg.call(None, scope)?;

    scope.insert("res.headers", arg);
    input.eval(scope)
}


pub fn get_module() -> ServModule {
    let mut output = ServModule::empty();

	output.insert("req.query",    ServFn::Core(query_all).into());
	output.insert("cookies",      ServFn::Core(get_cookies).into());
	output.insert("cookie.set",   ServFn::Meta(set_cookie).into());
	output.insert("with.headers", ServFn::Meta(with_headers).into());

	output
}
