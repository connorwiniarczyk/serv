use crate::ServValue;
use crate::ServResult;

use crate::Label;
use crate::value::ServFn;

// use crate::Stack;
use crate::Stack;

// use crate::Words;
use crate::parser;
// use crate::compile;
use std::io::Read;

// mod host;
mod list_operations;
// mod request;
// mod sql;
// mod json;


// pub use host::*;
pub use list_operations::*;

use std::process::{Command, Stdio};
use std::io::Write;
use std::io::{BufWriter};
// use evalexpr::eval;
use std::collections::VecDeque;
use sqlite;

use std::collections::HashMap;

pub fn hello_world(input: ServValue, scope: &Stack) -> ServResult {
    Ok(ServValue::Text("hello world".to_owned()))
}

pub fn uppercase(input: ServValue, scope: &Stack) -> ServResult {
    Ok(ServValue::Text(input.to_string().to_uppercase()))
}

pub fn inline(input: ServValue, scope: &Stack) -> ServResult {
    Ok(ServValue::Text(input.to_string().lines().collect()))
}

pub fn incr(input: ServValue, scope: &Stack) -> ServResult {
    Ok(ServValue::Int(input.expect_int()? + 1))
}

pub fn decr(input: ServValue, scope: &Stack) -> ServResult {
    Ok(ServValue::Int(input.expect_int()? - 1))
}

pub fn markdown(input: ServValue, scope: &Stack) -> ServResult {
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


pub fn read_file(input: ServValue, scope: &Stack) -> ServResult {
    let path = input.to_string();
    let contents = std::fs::read_to_string(path).map_err(|e| "failed to open file")?;
    Ok(ServValue::Text(contents))
}

pub fn read_file_raw(input: ServValue, scope: &Stack) -> ServResult {
    let path = input.to_string();
    let contents = std::fs::read(path).map_err(|e| "failed to open file")?;
    Ok(ServValue::Raw(contents))
}

pub fn read_dir(input: ServValue, scope: &Stack) -> ServResult {
    let paths = std::fs::read_dir(input.to_string()).map_err(|_| "invalid path")?;
    let mut output = VecDeque::new();
    for path in paths {
        if let Ok(p) = path { output.push_back(ServValue::Text(p.path().display().to_string())); }
    }
    Ok(ServValue::List(output))
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

// pub fn drop(words: &mut Words, input: ServValue, scope: &Stack) -> ServResult {
//     _ = words.0.pop_front();
//     words.eval(input, scope)
// }


// pub fn render(words: &mut Words, input: ServValue, scope: &Stack) -> ServResult {
//     let path = words.take_next(scope)?.to_string();

//     let mut f = std::fs::File::open(&path).map_err(|e| "could not open file")?;
//     let mut template_str = String::new();
//     template_str.push('{');
//     f.read_to_string(&mut template_str).map_err(|e| "could not read file")?;
//     template_str.push('}');

// 	let template = parser::parse_template_from_text(&template_str).unwrap();

//     let rest = words.eval(input, scope)?;

// 	let mut child = scope.make_child();
//     child.insert_name("in", ServFunction::Literal(rest.clone()));

//     template.render(&child)
// }

// pub fn apply(words: &mut Words, input: ServValue, scope: &Stack) -> ServResult {
//     let next = words.next().ok_or("not enought arguments")?;
//     let arg = scope.get(&next).ok_or("not found")?.call(input.clone(), scope)?;

// 	let mut new_scope = scope.make_child();
// 	let ast = parser::parse_expression_from_text(arg.to_string().as_str()).unwrap();
// 	let func = compile(ast.0, &mut new_scope);

//     let rest = words.eval(input, scope)?;

//     func.call(rest, &new_scope)
// }


// pub fn using(words: &mut Words, input: ServValue, scope: &Stack) -> ServResult {
//     let arg_name = words.0.pop_front().unwrap();
//     let arg_fn = scope.get(&arg_name).unwrap();
//     let arg = arg_fn.call(input.clone(), scope)?;

// 	let ast = parser::parse_root_from_text(arg.to_string().as_str()).unwrap();
// 	let mut new_scope = scope.make_child();

// 	for declaration in ast.0 {
//     	if declaration.kind == "word" {
//         	let func = compile(declaration.value.0, &mut new_scope);
//         	new_scope.insert(declaration.key.to_owned().into(), func);
//     	}
// 	}

//     words.eval(input, &new_scope)
// }

// pub fn with_header(words: &mut Words, mut input: ServValue, scope: &Stack) -> ServResult {
//     let arg = words.take_next_with_input(scope, input.clone())?;
// 	let mut output = words.eval(input, scope)?;
// 	let headers: &mut ServValue = output.metadata()
//     	.entry("headers".to_owned())
//     	.or_insert(ServValue::List(VecDeque::new()));

// 	let ServValue::List(list) = headers else {panic!()};
// 	list.push_back(arg);
// 	Ok(output)
// }

// pub fn with_status(words: &mut Words, input: ServValue, scope: &Stack) -> ServResult {
//     let arg = words.take_next(scope)?;
//     let mut output = words.eval(input, scope)?;
// 	output.metadata().insert("status".to_owned(), arg);

// 	Ok(output)
// }

// use crate::error::ServError;
// fn parse_key_value(input: &str) -> Result<(String, String), &'static str> {
//     let mut iter = input.split("=").map(str::trim).map(str::to_string);
//     Ok((iter.next().ok_or("invalid option")?, iter.next().ok_or("invalid_option")?))
// }

// pub fn with_option(words: &mut Words, input: ServValue, scope: &Stack) -> ServResult {
//     let arg = words.take_next(scope)?;
//     let (key, value) = parse_key_value(&arg.to_string())?;
//     let mut output = words.eval(input, scope)?;
// 	output.metadata().insert(key, ServValue::Text(value));
// 	Ok(output)
// }

// pub fn get(words: &mut Words, input: ServValue, scope: &Stack) -> ServResult {
//     let arg = words.take_next(scope)?;
// 	let mut input = words.eval(input, scope)?;

// 	let output = match ((arg, input)) {
//     	(ServValue::Int(i),  ServValue::List(mut l)) => l.remove(i.try_into().unwrap()).unwrap_or(ServValue::None),
//     	(ServValue::Text(ref t), ServValue::Table(mut m)) =>  m.remove(t).unwrap_or(ServValue::None),

//     	_ => return Err("not a valid use of get"),
// 	};

// 	Ok(output)
// }


// pub fn bind_standard_library(scope: &mut Scope) {
	// scope.insert(Label::name("hello"),           ServFunction::Core(hello_world));
	// scope.insert(Label::name("uppercase"),       ServFunction::Core(uppercase));
	// scope.insert(Label::name("incr"),            ServFunction::Core(incr));
	// scope.insert(Label::name("decr"),            ServFunction::Core(decr));
	// scope.insert(Label::name("%"),               ServFunction::Core(math_expr));
	// scope.insert(Label::name("sum"),             ServFunction::Core(sum));
	// scope.insert(Label::name("read"),            ServFunction::Core(read_file));
	// scope.insert(Label::name("read.raw"),        ServFunction::Core(read_file_raw));
	// scope.insert(Label::name("file.utf8"),       ServFunction::Core(read_file));
	// scope.insert(Label::name("file.raw"),        ServFunction::Core(read_file_raw));
	// scope.insert(Label::name("file"),            ServFunction::Core(read_file_raw));
	// scope.insert(Label::name("inline"),          ServFunction::Core(inline));
	// scope.insert(Label::name("exec"),            ServFunction::Core(exec));
	// scope.insert(Label::name("markdown"),        ServFunction::Core(markdown));
	// scope.insert(Label::name("ls"),              ServFunction::Core(read_dir));
	// scope.insert(Label::name("count"),           ServFunction::Core(count));

	// scope.insert(Label::name("!"),            ServFunction::Meta(drop));
	// scope.insert(Label::name("map"),             ServFunction::Meta(map));
	// scope.insert(Label::name("&"),               ServFunction::Meta(quote));
	// scope.insert(Label::name("*"),               ServFunction::Meta(deref));
	// scope.insert(Label::name("using"),        ServFunction::Meta(using));
	// scope.insert(Label::name("let"),          ServFunction::Meta(using));
	// scope.insert(Label::name("choose"),       ServFunction::Meta(choose));
	// scope.insert(Label::name("*"),            ServFunction::Meta(apply));
	// scope.insert(Label::name("exec.pipe"),    ServFunction::Meta(exec_pipe));
	// scope.insert(Label::name("with_header"),  ServFunction::Meta(with_header));
	// scope.insert(Label::name("with_status"),  ServFunction::Meta(with_status));
	// scope.insert(Label::name("fold"),         ServFunction::Meta(fold));
	// scope.insert(Label::name("get"),          ServFunction::Meta(get));
	// scope.insert(Label::name("switch"),       ServFunction::Meta(switch));
	// scope.insert(Label::name("render"),       ServFunction::Meta(render));
	// scope.insert(Label::name("join"),         ServFunction::Meta(join));
	// scope.insert(Label::name("split"),        ServFunction::Meta(split));
	// scope.insert(Label::name("get"),          ServFunction::Meta(get));
	// scope.insert(Label::name("."),            ServFunction::Meta(get));
// 	request::bind(scope);
// 	sql::bind(scope);
// 	json::bind(scope);
// }

use std::rc::Rc;
use crate::value::Transform;
use crate::value::eval;

pub fn take(mut input: VecDeque<ServValue>, scope: &mut Stack) -> ServResult {
   	let arg  = input.pop_front().ok_or("word expected")?;
   	let ServValue::Ref(name @ Label::Name(_)) = arg else { panic!() };

   	let rest = eval(input, scope)?;

   	let out = match rest {
       	ServValue::List(mut l) => { scope.insert(name, l.pop_front().ok_or("")?); ServValue::List(l) },
       	element => { scope.insert(name, element); ServValue::None },
   	};
   	// scope.insert(name, ServValue::Int(10));
   	
   	Ok(out)

   	// f(arg, rest, scope)

    // if let ServValue::Ref(Label::Name(ref name)) = arg {
    //     scope.insert("");
    //     let transform = Transform(Rc::new(|s: &mut Stack| { s.insert_name(name, ServValue::None) }));
    //     let out = ServValue::Transform(Box::new(input), transform);
    //     Ok(out)
    // }
    // else { return Ok(input) }
}

pub fn pop(input: ServValue, scope: &Stack) -> ServResult {
    let ServValue::List(mut inner) = input else { return Ok(ServValue::None) };
    _ = inner.pop_front();


    // let arg = expr.pop_front().ok_or("not enough args")?;
    // let target = ServValue::FnLiteral(crate::ServFn::Expr(expr)).eval(None, scope)?;
    // let ServValue::List(list) = target else { return Err("tried to map onto a non list") };

    Ok(ServValue::List(inner))
}

pub fn dequote(input: ServValue, scope: &Stack) -> ServResult {
    match input {
		ServValue::List(words) => crate::value::eval(words, &mut scope.make_child()),
		i => Ok(i),
    }
}

pub fn quote(input: VecDeque::<ServValue>, scope: &mut Stack) -> ServResult {
    Ok(ServValue::List(input))
}

fn generate_list(input: VecDeque::<ServValue>, scope: &mut Stack) -> ServResult {
    let mut output = VecDeque::new();
    for item in input.into_iter() {
        output.push_back(item.call(None, scope)?);
    }
    Ok(ServValue::List(output))
}

fn map(arg: ServValue, input: ServValue, scope: &Stack) -> ServResult {
    let mut output = VecDeque::new();
    let ServValue::List(list) = input else { return arg.call(Some(input), scope) };
    for item in list.into_iter() {
        output.push_back(arg.call(Some(item), scope)?);
    }
    Ok(ServValue::List(output))
}

pub fn bind_standard_library(scope: &mut crate::Stack) {
	// scope.insert(Label::name("hello"),           ServValue::FnLiteral(ServFn::Core(hello_world)));
	// scope.insert(Label::name("uppercase"),       ServValue::FnLiteral(ServFn::Core(uppercase)));
	// scope.insert(Label::name("incr"),            ServValue::FnLiteral(ServFn::Core(incr)));
	// scope.insert(Label::name("decr"),            ServValue::FnLiteral(ServFn::Core(decr)));

	scope.insert(Label::name("+"),            ServValue::Func(ServFn::Core(incr)));
	scope.insert(Label::name("-"),            ServValue::Func(ServFn::Core(decr)));
	scope.insert(Label::name("map"),          ServValue::Func(ServFn::ArgFn(map)));
	// scope.insert(Label::name("take"),         ServValue::Func(ServFn::Meta(take)));

	// scope.insert(Label::name("%"),               ServValue::FnLiteral(ServFn::Core(math_expr)));
	// scope.insert(Label::name("sum"),             ServValue::FnLiteral(ServFn::Core(sum)));
	// scope.insert(Label::name("read"),            ServValue::FnLiteral(ServFn::Core(read_file)));
	// scope.insert(Label::name("read.raw"),        ServValue::FnLiteral(ServFn::Core(read_file_raw)));
	// scope.insert(Label::name("file.utf8"),       ServValue::FnLiteral(ServFn::Core(read_file)));
	// scope.insert(Label::name("file.raw"),        ServValue::FnLiteral(ServFn::Core(read_file_raw)));
	// scope.insert(Label::name("file"),            ServValue::FnLiteral(ServFn::Core(read_file_raw)));
	// scope.insert(Label::name("inline"),          ServValue::FnLiteral(ServFn::Core(inline)));
	// scope.insert(Label::name("exec"),            ServValue::FnLiteral(ServFn::Core(exec)));
	// scope.insert(Label::name("markdown"),        ServValue::FnLiteral(ServFn::Core(markdown)));
	// scope.insert(Label::name("ls"),              ServValue::FnLiteral(ServFn::Core(read_dir)));
	scope.insert(Label::name("count"),           ServValue::Func(ServFn::Core(count)));
	scope.insert(Label::name("pop"),             ServValue::Func(ServFn::Core(pop)));

	// scope.insert(Label::name("!"),            ServFunction::Meta(drop));
	// scope.insert(Label::name("map"),             ServValue::FnLiteral(ServFn::CoreMeta(map)));  // ::Meta(map));

	// scope.insert(Label::name("pop"),             ServValue::FnLiteral(ServFn::Core(pop))); 
	scope.insert(Label::name("["),               ServValue::Func(ServFn::Core(dequote)));
	scope.insert(Label::name("]"),               ServValue::Func(ServFn::Meta(quote)));
	scope.insert(Label::name("|"),               ServValue::Func(ServFn::Meta(generate_list)));
	scope.insert(Label::name("take"),               ServValue::Func(ServFn::Meta(take)));
	// scope.insert(Label::name("&"),               ServFunction::Meta(quote));
	// scope.insert(Label::name("*"),               ServFunction::Meta(deref));
	// scope.insert(Label::name("using"),        ServFunction::Meta(using));
	// scope.insert(Label::name("let"),          ServFunction::Meta(using));
	// scope.insert(Label::name("choose"),       ServFunction::Meta(choose));
	// scope.insert(Label::name("*"),            ServFunction::Meta(apply));
	// scope.insert(Label::name("exec.pipe"),    ServFunction::Meta(exec_pipe));
	// scope.insert(Label::name("with_header"),  ServFunction::Meta(with_header));
	// scope.insert(Label::name("with_status"),  ServFunction::Meta(with_status));
	// scope.insert(Label::name("fold"),         ServFunction::Meta(fold));
	// scope.insert(Label::name("get"),          ServFunction::Meta(get));
	// scope.insert(Label::name("switch"),       ServFunction::Meta(switch));
	// scope.insert(Label::name("render"),       ServFunction::Meta(render));
	// scope.insert(Label::name("join"),         ServFunction::Meta(join));
	// scope.insert(Label::name("split"),        ServFunction::Meta(split));
	// scope.insert(Label::name("get"),          ServFunction::Meta(get));
	// scope.insert(Label::name("."),            ServFunction::Meta(get));

	// request::bind(scope);
	// sql::bind(scope);
	// json::bind(scope);
}
