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
mod request;
mod math;
pub mod json;

fn serv_try(arg: ServValue, input: ServValue, scope: &Stack) -> ServResult {
	let ServValue::Module(m) = arg else { return Err(ServError::expected_type("module", input)) };

	let mut child = scope.make_child();
    for (label, expr) in m.definitions {
		child.insert(label, ServValue::Func(ServFn::Expr(expr, false)));
    }

	let mut output: ServResult = Err(ServError::new(500, "empty try statement"));
    for mut expr in m.statements {
        output = expr.eval(&mut child);
        match &output {
			Err(_) => (),
			Ok(ServValue::None) => (),
			Ok(_) => return output,
        };
    }

	output
}

fn print(input: ServValue, scope: &Stack) -> ServResult {
    println!("{}", input);
    Ok(input)
}


fn uppercase(input: ServValue, scope: &Stack) -> ServResult {
    Ok(input.to_string().to_uppercase().into())
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
    Ok(ServValue::Text(output.into()))
}


fn drop(arg: ServValue, input: ServValue, scope: &Stack) -> ServResult {
    Ok(input)
}

fn using(mut input: ServList, scope: &mut Stack) -> ServResult {
    let mut arg = input.next().ok_or("using expects an arg")?;
    arg = arg.call(None, scope)?;

	let ServValue::Module(m) = arg else { return Err(ServError::expected_type("module", arg))};

	for (key, value) in m.definitions {
        scope.insert(key, value.as_expr());
	}

	input.eval(scope)
}

fn as_template(input: ServValue, scope: &Stack) -> ServResult {
    let template = crate::servparser::parse_template_from_text(&input.to_string(), false).unwrap();
    template.render(&scope)
}

pub fn apply(input: ServValue, scope: &Stack) -> ServResult {
    let ServValue::Module(m) = input else { return Err(ServError::expected_type("Module", input)) };
    m.call(None, &mut scope.make_child())
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

fn choose(mut arg: ServValue, input: ServValue, scope: &Stack) -> ServResult {
    let ServValue::Module(m) = arg else { return Err(ServError::expected_type("module", arg))};

    let mut child = scope.make_child();
    let mut index = input.clone();

    for (label, mut expr) in m.definitions {
        if label == Label::Name("i".to_string()) {
            expr.push_back(input.clone());
			index = expr.eval(&mut child)?;
        }

        child.insert(label, ServValue::Func(ServFn::Expr(expr, false)));
    }

    let mut path = match index {
        ServValue::None => m.statements.get(0).unwrap().clone(),
        ServValue::Bool(b) if b == true => m.statements.get(0).unwrap().clone(),
        ServValue::Bool(b)  => m.statements.get(1).unwrap().clone(),
        ServValue::Int(i)   => m.statements.get(i as usize).unwrap().clone(),

        other => panic!("not supported {:?}", other),
    };

    path.push_back(input);
    path.eval(&mut child)
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
}


fn parse_module(input: ServValue, scope: &Stack) -> ServResult {
    let module = servparser::parse_root_from_text(&input.to_string()).unwrap();
    Ok(ServValue::Module(module))
}

pub fn assert(arg: ServValue, input: ServValue, scope: &Stack) -> ServResult {
    let test = arg.call(Some(input.clone()), scope)?;

    if test.is_truthy() {
        return Ok(input)
    } else {
        return Err(ServError::new(500, "Assertion Failed"))
    }
}

pub fn bind_standard_library(scope: &mut crate::Stack) {

	// error handling
	scope.insert_name("try",          ServFn::ArgFn(serv_try).into());
	scope.insert_name("assert",       ServFn::ArgFn(assert).into());

	scope.insert_name("print",        ServFn::Core(print).into());
	scope.insert_name("[",            ServFn::Core(dequote).into());
	scope.insert_name("]",            ServFn::Meta(quote).into());
	scope.insert_name("using",        ServFn::Meta(using).into());
	scope.insert_name("let",          ServFn::Meta(using).into());
	scope.insert_name("!",            ServFn::ArgFn(drop).into());
	scope.insert_name("?",            ServFn::ArgFn(choose).into());
	scope.insert_name("switch",       ServFn::ArgFn(switch).into());
	scope.insert_name("include",      ServFn::Meta(include).into());
	scope.insert_name("*",            ServFn::Core(apply).into());
	scope.insert_name("uppercase",    ServFn::Core(uppercase).into());
	scope.insert_name("markdown",     ServFn::Core(markdown).into());
	scope.insert_name("with.header",  ServFn::Meta(with_headers).into());
	scope.insert_name("~",            ServFn::Core(as_template).into());
	scope.insert_name("@",            ServFn::Core(parse_module).into());


	math::bind(scope);
	list::bind(scope);
	host::bind(scope);
	json::bind(scope);
	sql::bind(scope);
	request::bind(scope);
}
