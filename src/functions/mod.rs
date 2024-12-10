use crate::ServValue;
use crate::ServResult;

use crate::Label;
use crate::value::ServFn;
use crate::value::eval;
use crate::error::ServError;
use crate::Stack;
use crate::servparser;
use std::collections::HashMap;
use std::collections::VecDeque;

// use crate::compile;
use std::io::Read;

mod host;
mod list;
mod sql;
mod json;
// mod request;

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

fn using(mut input: VecDeque::<ServValue>, scope: &mut Stack) -> ServResult {
    let arg = input.pop_front().ok_or("using expects an arg")?;
    let text = arg.call(None, &scope)?.to_string();

	let ast = servparser::parse_root_from_text(&text).unwrap();
	let mut new_scope = scope.make_child();

	for declaration in ast.0 {
    	if declaration.kind == "word" {
        	let func = crate::compile(declaration.value, &mut new_scope);
        	new_scope.insert(declaration.key.to_owned().into(), func);
    	}
	}

    eval(input, &mut new_scope)
}

fn as_template(input: ServValue, scope: &Stack) -> ServResult {
    let template = crate::servparser::parse_template_from_text(&input.to_string(), false).unwrap();
    template.render(&scope)
}

pub fn apply(input: ServValue, scope: &Stack) -> ServResult {
	let mut child = scope.make_child();
	let text = input.to_string();
	let Ok(ast) = servparser::parse_expression_from_text(&text) else {
    	panic!("failed to parse serv text in apply statement: {:?}", text);
	};

	let func = crate::compile(ast, &mut child);
	func.call(None, &child)
	// eval(expr, &mut child)

    // let rest = words.eval(input, scope)?;

    // func.call(rest, &new_scope)
}

fn with_option(arg: ServValue, mut input: ServValue, scope: &Stack) -> ServResult {
    fn parse_key_value(input: &str) -> Result<(String, String), &'static str> {
        let mut iter = input.split("=").map(str::trim).map(str::to_string);
        Ok((iter.next().ok_or("invalid option")?, iter.next().ok_or("invalid_option")?))
    }

    let text = arg.call(None, scope)?.to_string();
    let (key, value) = parse_key_value(&text)?;
	input.metadata().insert(key, ServValue::Text(value));
	Ok(input)
}

fn with_status(arg: ServValue, mut input: ServValue, scope: &Stack) -> ServResult {
	input.metadata().insert("status".to_owned(), arg);
	Ok(input)
}

fn with_header(arg: ServValue, mut input: ServValue, scope: &Stack) -> ServResult {
	let headers: &mut ServValue = input.metadata()
    	.entry("headers".to_owned())
    	.or_insert(ServValue::List(VecDeque::new()));

	let ServValue::List(list) = headers else {panic!()};
	list.push_back(arg.call(None, scope)?);
	Ok(input)
}

fn dequote(input: ServValue, scope: &Stack) -> ServResult {
    match input {
		ServValue::List(words) => crate::value::eval(words, &mut scope.make_child()),
		i => Ok(i),
    }
}

fn quote(input: VecDeque::<ServValue>, scope: &mut Stack) -> ServResult {
    Ok(ServValue::List(input))
}

fn choose(mut input: VecDeque<ServValue>, scope: &mut Stack) -> ServResult {
    let if_true  = input.pop_front().unwrap_or(ServValue::None);
    let if_false = input.pop_front().unwrap_or(ServValue::None);

    let value = crate::value::eval(input, scope)?;

    match &value {
        ServValue::None        => if_false,
        ServValue::Bool(false) => if_false,
        ServValue::Int(0)      => if_false,

        otherwise => if_true,
    }.call(Some(value), scope)
}

pub fn bind_standard_library(scope: &mut crate::Stack) {

	scope.insert(Label::name("["),           ServValue::Func(ServFn::Core(dequote)));
	scope.insert(Label::name("]"),           ServValue::Func(ServFn::Meta(quote)));
	scope.insert(Label::name("using"),       ServValue::Func(ServFn::Meta(using)));
	scope.insert(Label::name("let"),         ServValue::Func(ServFn::Meta(using)));
	scope.insert(Label::name("!"),           ServValue::Func(ServFn::ArgFn(drop)));
	scope.insert(Label::name("choose"),      ServValue::Func(ServFn::Meta(choose)));
	scope.insert(Label::name("+"),           ServValue::Func(ServFn::Core(incr)));
	scope.insert(Label::name("-"),           ServValue::Func(ServFn::Core(decr)));
	scope.insert(Label::name("%"),           ServValue::Func(ServFn::Core(math_expr)));
	scope.insert(Label::name("*"),           ServValue::Func(ServFn::Core(apply)));
	scope.insert(Label::name("hello"),       ServValue::Func(ServFn::Core(hello_world)));
	scope.insert(Label::name("uppercase"),   ServValue::Func(ServFn::Core(uppercase)));
	scope.insert(Label::name("inline"),      ServValue::Func(ServFn::Core(inline)));
	scope.insert(Label::name("markdown"),    ServValue::Func(ServFn::Core(markdown)));
	scope.insert(Label::name("~"),           ServValue::Func(ServFn::Core(as_template)));
	scope.insert(Label::name("with_header"), ServValue::Func(ServFn::ArgFn(with_header)));
	scope.insert(Label::name("with_status"), ServValue::Func(ServFn::ArgFn(with_status)));
	scope.insert(Label::name("with_option"), ServValue::Func(ServFn::ArgFn(with_option)));
	// scope.insert(Label::name("*"),            ServFunction::Meta(apply));

	list::bind(scope);
	host::bind(scope);
	json::bind(scope);
	sql::bind(scope);

	// request::bind(scope);
	// json::bind(scope);
}
