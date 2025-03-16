use crate::ServValue;
use crate::ServResult;
use crate::ServModule;

use crate::Label;
use crate::error::ServError;
use crate::Stack;
use crate::servparser;

use std::collections::HashMap;

use crate::value::ServFn;
use crate::value::ServList;

fn math_expr(input: ServValue, scope: &Stack) -> ServResult {
    let expression = input.to_string();
	let res = evalexpr::eval(expression.as_str()).unwrap();
	Ok(match res {
		evalexpr::Value::String(s)  => s.into(),
		evalexpr::Value::Int(x)     => ServValue::Int(x),
		evalexpr::Value::Boolean(x) => ServValue::Bool(x),
		evalexpr::Value::Float(x)   => ServValue::Float(x),
		evalexpr::Value::Empty      => ServValue::None,
		_ => todo!(),
	})
}

fn incr(input: ServValue, scope: &Stack) -> ServResult {
    Ok(ServValue::Int(input.expect_int()? + 1))
}

fn decr(input: ServValue, scope: &Stack) -> ServResult {
    Ok(ServValue::Int(input.expect_int()? - 1))
}

fn yes(input: ServValue, scope: &Stack) -> ServResult {
    Ok(ServValue::Bool(true))
}

fn equals(arg: ServValue, input: ServValue, scope: &Stack) -> ServResult {
    Ok(ServValue::Bool(match (arg.call(None, scope)?, input) {
        (ServValue::Int(a), ServValue::Int(b)) => a == b,
        _ => todo!(),
    }))
}

pub fn get_module() -> ServModule {
    let mut output = ServModule::empty();

	output.insert("+",  ServFn::Core(incr).into());
	output.insert("-",  ServFn::Core(decr).into());
	output.insert("%",  ServFn::Core(math_expr).into());
	output.insert("eq", ServFn::ArgFn(equals).into());
	output.insert("true",  ServFn::Core(yes).into());
	output.insert("else",  ServFn::Core(yes).into());

	output
}

pub fn bind(scope: &mut Stack) {
	scope.insert_name("+",  ServFn::Core(incr).into());
	scope.insert_name("-",  ServFn::Core(decr).into());
	scope.insert_name("%",  ServFn::Core(math_expr).into());
	scope.insert_name("eq", ServFn::ArgFn(equals).into());

	scope.insert_name("true",  ServFn::Core(yes).into());
	scope.insert_name("else",  ServFn::Core(yes).into());
}
