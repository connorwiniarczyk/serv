use crate::ServValue;
use crate::ServResult;
use crate::Scope;
use crate::Words;
use crate::parser;
use crate::compile;
use std::process::{Command, Stdio};
use std::io::Write;
use std::io::{BufWriter};

use evalexpr::eval;
use std::collections::VecDeque;

pub fn exec(input: ServValue, scope: &Scope) -> ServResult {
    let mut cmd = Command::new(input.to_string())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let output = cmd.wait_with_output().unwrap();
    Ok(ServValue::Text(std::str::from_utf8(&output.stdout).unwrap().to_owned()))
}

pub fn execpipe(words: &mut Words, input: ServValue, scope: &Scope) -> ServResult {
    let arg_name = words.0.pop_front().unwrap();
    let arg_fn = scope.get(&arg_name).unwrap();
    let arg = arg_fn.call(input.clone(), scope)?;
    let rest = words.eval(input, scope)?;

    let mut cmd = Command::new(arg.to_string())
        .stderr(Stdio::null())
        .stdout(Stdio::piped())
        .stdin(Stdio::piped())
        .spawn()
        .unwrap();

    cmd.stdin.as_mut().unwrap().write_all(rest.to_string().as_bytes());
    let output = cmd.wait_with_output().unwrap();

    Ok(ServValue::Text(std::str::from_utf8(&output.stdout).unwrap().to_owned()))
}




pub fn hello_world(input: ServValue, scope: &Scope) -> ServResult {
    Ok(ServValue::Text("hello world".to_owned()))
}

pub fn uppercase(input: ServValue, scope: &Scope) -> ServResult {
    Ok(ServValue::Text(input.to_string().to_uppercase()))
}

pub fn inline(input: ServValue, scope: &Scope) -> ServResult {
    Ok(ServValue::Text(input.to_string().lines().collect()))
}

pub fn incr(input: ServValue, scope: &Scope) -> ServResult {
    Ok(ServValue::Int(input.expect_int()? + 1))
}

pub fn markdown(input: ServValue, scope: &Scope) -> ServResult {
    Ok(ServValue::Text(markdown::to_html(input.to_string().as_str())))
}

pub fn decr(input: ServValue, scope: &Scope) -> ServResult {
    Ok(ServValue::Int(input.expect_int()? - 1))
}

pub fn read_file(input: ServValue, scope: &Scope) -> ServResult {
    let path = input.to_string();
    let contents = std::fs::read_to_string(path).map_err(|e| "failed to open file")?;
    Ok(ServValue::Text(contents))
}

pub fn math_expr(input: ServValue, scope: &Scope) -> ServResult {
    let expression = input.to_string();
	let res = eval(expression.as_str()).unwrap();
	Ok(match res {
		evalexpr::Value::String(s) => ServValue::Text(s),
		evalexpr::Value::Int(x) => ServValue::Int(x),
		// evalexpr::Value::Boolean(x) => ServValue::Boolean(x),
		// evalexpr::Value::Float(x) => ServValue::Float(x),
		evalexpr::Value::Empty => ServValue::None,
		_ => todo!(),
	})
}

pub fn sum(input: ServValue, scope: &Scope) -> ServResult {
    if let ServValue::List(l) = input {
        let mut sum = 0;
        for x in l.into_iter() { sum += x.expect_int()? };
        Ok(ServValue::Int(sum))
    }

    else {
        Ok(ServValue::Int(input.expect_int()?))
    }
}


pub fn drop(words: &mut Words, input: ServValue, scope: &Scope) -> ServResult {
    _ = words.0.pop_front();
    words.eval(input, scope)
}

pub fn map(words: &mut Words, input: ServValue, scope: &Scope) -> ServResult {
    let next = words.next().ok_or("not enough arguments")?;
    let arg = scope.get(&next).ok_or("not found")?;
    let rest = words.eval(input, scope)?;

	let mapped = match rest {
    	ServValue::List(list) => ServValue::List(list.into_iter().map(|a| arg.call(a, scope).unwrap()).collect()),
    	_ => todo!(),
	};

	Ok(mapped)
}

pub fn apply(words: &mut Words, input: ServValue, scope: &Scope) -> ServResult {
    let next = words.next().ok_or("not enought arguments")?;
    let arg = scope.get(&next).ok_or("not found")?.call(input.clone(), scope)?;

	let mut new_scope = scope.make_child();
	let ast = parser::parse_expression_from_text(arg.to_string().as_str()).unwrap();
	let func = compile(ast.0, &mut new_scope);

    let rest = words.eval(input, scope)?;

    func.call(rest, &new_scope)
}

pub fn choose(words: &mut Words, input: ServValue, scope: &Scope) -> ServResult {
    let next = words.next().ok_or("not enought arguments")?;
    let arg = scope.get(&next).ok_or("not found")?.call(input.clone(), scope)?;
    let rest = words.eval(input, scope)?;

	let ServValue::List(list) = rest else { return Err("not a valid list") };

	let index: usize = arg.expect_int()?.try_into().unwrap();

	Ok(list[index.clamp(0, list.len() - 1)].clone())
}

pub fn using(words: &mut Words, input: ServValue, scope: &Scope) -> ServResult {
    let arg_name = words.0.pop_front().unwrap();
    let arg_fn = scope.get(&arg_name).unwrap();
    let arg = arg_fn.call(ServValue::None, scope)?;

	let ast = parser::parse_root_from_text(arg.to_string().as_str()).unwrap();
	let mut new_scope = scope.make_child();

	for declaration in ast.0 {
    	if declaration.kind == "word" {
        	let func = compile(declaration.value.0, &mut new_scope);
        	new_scope.insert(declaration.key.to_owned().into(), func);
    	}
	}

    words.eval(input, &new_scope)
}
