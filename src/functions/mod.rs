use crate::ServValue;
use crate::ServResult;

use crate::Label;
use crate::error::ServError;
use crate::Stack;
use crate::servparser;

use std::collections::HashMap;

use crate::value::ServFn;
use crate::value::ServList;

use std::io::Read;

mod host;
mod list;
mod sql;
pub mod json;

mod request;

fn print(input: ServValue, scope: &Stack) -> ServResult {
    println!("{}", input);
    Ok(input)
}

fn yes(input: ServValue, scope: &Stack) -> ServResult {
    Ok(ServValue::Bool(true))
}

fn hello_world(input: ServValue, scope: &Stack) -> ServResult {
    Ok(ServValue::Text("hello world".to_owned()))
}

fn uppercase(input: ServValue, scope: &Stack) -> ServResult {
    Ok(ServValue::Text(input.to_string().to_uppercase()))
}

fn inline(input: ServValue, scope: &Stack) -> ServResult {
    Ok(ServValue::Text(input.to_string().lines().collect()))
}

fn incr(input: ServValue, scope: &Stack) -> ServResult {
    Ok(ServValue::Int(input.expect_int()? + 1))
}

fn decr(input: ServValue, scope: &Stack) -> ServResult {
    Ok(ServValue::Int(input.expect_int()? - 1))
}

fn markdown(input: ServValue, scope: &Stack) -> ServResult {
    let compile_options = markdown::CompileOptions {
        allow_dangerous_html: true,
        allow_dangerous_protocol: true,
        gfm_tagfilter: false,
        ..markdown::CompileOptions::default()
    };

    let options = markdown::Options {
        compile: compile_options,
        ..markdown::Options::gfm()
    };

    let output = markdown::to_html_with_options(input.to_string().as_str(), &options).unwrap();
    Ok(ServValue::Text(output))
}

pub fn math_expr(input: ServValue, scope: &Stack) -> ServResult {
    let expression = input.to_string();
	let res = evalexpr::eval(expression.as_str()).unwrap();
	Ok(match res {
		evalexpr::Value::String(s)  => ServValue::Text(s),
		evalexpr::Value::Int(x)     => ServValue::Int(x),
		evalexpr::Value::Boolean(x) => ServValue::Bool(x),
		evalexpr::Value::Float(x)   => ServValue::Float(x),
		evalexpr::Value::Empty      => ServValue::None,
		_ => todo!(),
	})
}

fn drop(arg: ServValue, input: ServValue, scope: &Stack) -> ServResult {
    Ok(input)
}

fn using(mut input: ServList, scope: &mut Stack) -> ServResult {
   todo!();
 //    let arg = input.pop_front().ok_or("using expects an arg")?;
 //    let text = arg.call(None, &scope)?.to_string();

	// let ast = servparser::parse_root_from_text(&text).unwrap();
	// let mut new_scope = scope.make_child();

	// for declaration in ast.0 {
 //    	if declaration.kind == "word" {
 //        	let key = declaration.key();
 //        	let func = crate::compile(declaration.value, &mut new_scope);
 //        	// new_scope.insert(declaration.key.to_owned().into(), func);
 //        	new_scope.insert(key.into(), func);
 //    	}
	// }

 //    eval(input, &mut new_scope)
}

fn as_template(input: ServValue, scope: &Stack) -> ServResult {
    let template = crate::servparser::parse_template_from_text(&input.to_string(), false).unwrap();
    template.render(&scope)
}

pub fn apply(input: ServValue, scope: &Stack) -> ServResult {
    let ServValue::Module(m) = input else { return Err(ServError::expected_type("Module", input)) };
	let mut child = scope.make_child();
	let mut result = ServValue::default();

	for mut expr in m.statements {
    	result = expr.eval(&mut child)?;
	}

	Ok(result)
}

fn with_headers(mut input: ServList, scope: &mut Stack) -> ServResult {
    let mut arg = input.pop()?;
	arg = arg.call(None, scope)?;

    let ServValue::Module(m) = &arg else {
        return Err(ServError::expected_type("Module", arg))
    };

    scope.insert_name("res.headers", arg);

    input.eval(scope)
}

fn dequote(input: ServValue, scope: &Stack) -> ServResult {
	if let ServValue::List(mut expr) = input {
    	expr.eval(&mut scope.make_child())
	}

	else {
    	Ok(input)
	}
}

fn quote(input: ServList, scope: &mut Stack) -> ServResult {
    Ok(ServValue::List(input))
}

fn choose(mut input: ServList, scope: &mut Stack) -> ServResult {
    let if_true  = input.pop().unwrap_or(ServValue::None);
    let if_false = input.pop().unwrap_or(ServValue::None);

	let value = input.eval(scope)?;

    match &value {
        ServValue::None        => if_false,
        ServValue::Bool(false) => if_false,
        ServValue::Int(0)      => if_false,

        otherwise => if_true,
    }.call(Some(value), scope)
}

fn include(mut input: ServList, scope: &mut Stack) -> ServResult {
    let val = input.eval(scope)?;
    let ServValue::Module(m) = val else { return Err(ServError::expected_type("Module", val)) };

    for (label, expr) in m.definitions {
		scope.insert(label, expr.as_expr());
    }

    Ok(ServValue::None)
}

fn switch(mut arg: ServValue, input: ServValue, scope: &Stack) -> ServResult {
	arg = arg.call(None, scope)?;
    let ServValue::Module(m) = arg else { return Err(ServError::expected_type("Module", arg)) };
    for (p, a) in m.equalities {
        let pattern: ServValue = p.into();
        let action: ServValue = a.into();
		if pattern.call(Some(input.clone()), scope).unwrap().is_truthy() {
    		return Ok(action.call(Some(input), scope).unwrap())
		}
    }

    Ok(ServValue::None)

 //    let mut expr = input.pop_front().unwrap_or(ServValue::None);
 //    if let ServValue::Ref(label) = expr { expr = scope.get(label).unwrap() };

 //    let text = match expr {
 //        ServValue::Func(ServFn::Template(t)) => t.literal_inner(),
 //        ref otherwise => expr.call(None, scope)?,
 //    }.to_string();

	// let ast = servparser::parse_root_from_text(&text).expect("failed to parse switch expression");
 //    let value = crate::value::eval(input, scope)?;

 //    let mut child = scope.make_child();

	// for declaration in ast.0 {
 //    	if let crate::ast::Pattern::Expr(pattern) = declaration.key {
 //        	match crate::compile(pattern, &mut child).call(Some(value.clone()), &child)? {
 //            	ServValue::None        => continue,
 //            	ServValue::Bool(false) => continue,
 //            	ServValue::Int(0)      => continue,
 //            	otherwise => {
 //                	return crate::compile(declaration.value, &mut child).call(Some(value.clone()), &child)
 //            	},
 //        	}
 //    	}
	// }

	// Ok(ServValue::None)
}

fn equals(arg: ServValue, input: ServValue, scope: &Stack) -> ServResult {
    Ok(ServValue::Bool(match (arg.call(None, scope)?, input) {
        (ServValue::Int(a), ServValue::Int(b)) => a == b,
        _ => todo!(),
    }))
}

fn parse_module(input: ServValue, scope: &Stack) -> ServResult {
    let module = servparser::parse_root_from_text(&input.to_string()).unwrap();
    Ok(ServValue::Module(module))
}

pub fn bind_standard_library(scope: &mut crate::Stack) {

	scope.insert_name("print",       ServValue::Func(ServFn::Core(print)));
	scope.insert_name("[",           ServValue::Func(ServFn::Core(dequote)));
	scope.insert_name("]",           ServValue::Func(ServFn::Meta(quote)));
	scope.insert_name("using",       ServValue::Func(ServFn::Meta(using)));
	scope.insert_name("let",         ServValue::Func(ServFn::Meta(using)));
	scope.insert_name("!",           ServValue::Func(ServFn::ArgFn(drop)));
	scope.insert_name("choose",      ServValue::Func(ServFn::Meta(choose)));

	scope.insert_name("switch",      ServValue::Func(ServFn::ArgFn(switch)));

	scope.insert_name("include",     ServValue::Func(ServFn::Meta(include)));
	scope.insert_name("+",           ServValue::Func(ServFn::Core(incr)));
	scope.insert_name("-",           ServValue::Func(ServFn::Core(decr)));
	scope.insert_name("eq",          ServValue::Func(ServFn::ArgFn(equals)));
	scope.insert_name("%",           ServValue::Func(ServFn::Core(math_expr)));
	scope.insert_name("*",           ServValue::Func(ServFn::Core(apply)));
	scope.insert_name("hello",       ServValue::Func(ServFn::Core(hello_world)));
	scope.insert_name("uppercase",   ServValue::Func(ServFn::Core(uppercase)));
	scope.insert_name("inline",      ServValue::Func(ServFn::Core(inline)));
	scope.insert_name("markdown",    ServValue::Func(ServFn::Core(markdown)));
	scope.insert_name("with.header", ServValue::Func(ServFn::Meta(with_headers)));
	scope.insert_name("~",           ServValue::Func(ServFn::Core(as_template)));

	scope.insert(Label::name("true"), ServValue::Func(ServFn::Core(yes)));
	scope.insert(Label::name("else"), ServValue::Func(ServFn::Core(yes)));

	scope.insert_name("@", ServValue::Func(ServFn::Core(parse_module)));

	list::bind(scope);
	host::bind(scope);
	json::bind(scope);
	sql::bind(scope);
	request::bind(scope);

	// request::bind(scope);
}
