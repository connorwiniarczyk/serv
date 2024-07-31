use crate::ServValue;
use crate::ServResult;
use crate::Scope;
use crate::Words;
use crate::parser;
use crate::compile;
use std::io::Read;

mod host;
mod list_operations;
pub use host::*;
pub use list_operations::*;

use std::process::{Command, Stdio};
use std::io::Write;
use std::io::{BufWriter};
use evalexpr::eval;
use std::collections::VecDeque;
use sqlite;

use std::collections::HashMap;

pub fn sql_exec(input: ServValue, scope: &Scope) -> ServResult {
    let connection = sqlite::open("serv.sqlite").unwrap();
    connection.execute(input.to_string()).unwrap();
    Ok(ServValue::None)
}

pub fn sql(input: ServValue, scope: &Scope) -> ServResult {
    let mut output: HashMap<String, ServValue> = HashMap::new();
    let connection = sqlite::open("serv.sqlite").unwrap();
    let mut statement = connection.prepare(input.to_string()).unwrap();

    while let Ok(sqlite::State::Row) = statement.next() {
        for (index, name) in statement.column_names().iter().enumerate() {
            let value = match statement.column_type(index).unwrap() {
                sqlite::Type::Binary  => {
                    let v: i64 = statement.read(index).unwrap();
                    ServValue::Bool(if v == 0 {false} else {true})
                },
                sqlite::Type::Float   => ServValue::Float(statement.read(index).unwrap()),
                sqlite::Type::Integer => ServValue::Int(statement.read(index).unwrap()),
                sqlite::Type::String  => ServValue::Text(statement.read(index).unwrap()),
                sqlite::Type::Null    => ServValue::None,

            };
			output.insert(name.clone(), value);
        }
    }

    Ok(ServValue::Table(output))
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

pub fn decr(input: ServValue, scope: &Scope) -> ServResult {
    Ok(ServValue::Int(input.expect_int()? - 1))
}

pub fn read_file(input: ServValue, scope: &Scope) -> ServResult {
    let path = input.to_string();
    let contents = std::fs::read_to_string(path).map_err(|e| "failed to open file")?;
    Ok(ServValue::Text(contents))
}

pub fn read_file_raw(input: ServValue, scope: &Scope) -> ServResult {
    let path = input.to_string();
    let contents = std::fs::read(path).map_err(|e| "failed to open file")?;
    Ok(ServValue::Raw(contents))
}

pub fn read_dir(input: ServValue, scope: &Scope) -> ServResult {
    let paths = std::fs::read_dir(input.to_string()).map_err(|_| "invalid path")?;
    let mut output = VecDeque::new();
    for path in paths {
        if let Ok(p) = path { output.push_back(ServValue::Text(p.path().display().to_string())); }
    }
    Ok(ServValue::List(output))
}

pub fn math_expr(input: ServValue, scope: &Scope) -> ServResult {
    let expression = input.to_string();
	let res = eval(expression.as_str()).unwrap();
	Ok(match res {
		evalexpr::Value::String(s) => ServValue::Text(s),
		evalexpr::Value::Int(x) => ServValue::Int(x),
		evalexpr::Value::Boolean(x) => ServValue::Bool(x),
		evalexpr::Value::Float(x) => ServValue::Float(x),
		evalexpr::Value::Empty => ServValue::None,
		_ => todo!(),
	})
}

pub fn drop(words: &mut Words, input: ServValue, scope: &Scope) -> ServResult {
    _ = words.0.pop_front();
    words.eval(input, scope)
}


pub fn render(words: &mut Words, input: ServValue, scope: &Scope) -> ServResult {
    let path = words.take_next(scope)?.to_string();

    let mut f = std::fs::File::open(&path).map_err(|e| "could not open file")?;
    let mut template_str = String::new();
    template_str.push('{');
    f.read_to_string(&mut template_str).map_err(|e| "could not read file")?;
    template_str.push('}');

	let template = parser::parse_template_from_text(&template_str).unwrap();

    let rest = words.eval(input, scope)?;

	let mut child = scope.make_child();
    child.insert_name("in", ServFunction::Literal(rest.clone()));

    template.render(&child)
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

pub fn with_header(words: &mut Words, mut input: ServValue, scope: &Scope) -> ServResult {
    let arg = words.take_next(scope)?;
	let mut output = words.eval(input, scope)?;
	let headers: &mut ServValue = output.metadata()
    	.entry("headers".to_owned())
    	.or_insert(ServValue::List(VecDeque::new()));

	let ServValue::List(list) = headers else {panic!()};
	list.push_back(arg);
	Ok(output)
}

pub fn with_status(words: &mut Words, input: ServValue, scope: &Scope) -> ServResult {
    let arg = words.take_next(scope)?;
    let mut output = words.eval(input, scope)?;
	output.metadata().insert("status".to_owned(), arg);

	Ok(output)
}

use crate::error::ServError;
fn parse_key_value(input: &str) -> Result<(String, String), &'static str> {
    let mut iter = input.split("=").map(str::trim).map(str::to_string);
    Ok((iter.next().ok_or("invalid option")?, iter.next().ok_or("invalid_option")?))
}

pub fn with_option(words: &mut Words, input: ServValue, scope: &Scope) -> ServResult {
    let arg = words.take_next(scope)?;
    let (key, value) = parse_key_value(&arg.to_string())?;
    let mut output = words.eval(input, scope)?;
	output.metadata().insert(key, ServValue::Text(value));
	Ok(output)
}


use crate::ServFunction;
use crate::FnLabel;

pub fn bind_standard_library(scope: &mut Scope) {
	scope.insert(FnLabel::name("hello"),     ServFunction::Core(hello_world));
	scope.insert(FnLabel::name("uppercase"), ServFunction::Core(uppercase));
	scope.insert(FnLabel::name("incr"),      ServFunction::Core(incr));
	scope.insert(FnLabel::name("decr"),      ServFunction::Core(decr));
	scope.insert(FnLabel::name("%"),         ServFunction::Core(math_expr));
	scope.insert(FnLabel::name("sum"),       ServFunction::Core(sum));
	scope.insert(FnLabel::name("read"),      ServFunction::Core(read_file));
	scope.insert(FnLabel::name("read.raw"),      ServFunction::Core(read_file_raw));
	scope.insert(FnLabel::name("file"),      ServFunction::Core(read_file));
	scope.insert(FnLabel::name("file.raw"),      ServFunction::Core(read_file_raw));
	scope.insert(FnLabel::name("inline"),    ServFunction::Core(inline));
	scope.insert(FnLabel::name("exec"),    ServFunction::Core(exec));
	scope.insert(FnLabel::name("markdown"),    ServFunction::Core(markdown));
	scope.insert(FnLabel::name("sql"),    ServFunction::Core(sql));
	scope.insert(FnLabel::name("sqlexec"),    ServFunction::Core(sql_exec));
	scope.insert(FnLabel::name("ls"),    ServFunction::Core(read_dir));
	scope.insert(FnLabel::name("count"),    ServFunction::Core(count));

	scope.insert(FnLabel::name("!"),         ServFunction::Meta(drop));
	scope.insert(FnLabel::name("map"),       ServFunction::Meta(map));
	scope.insert(FnLabel::name("using"),     ServFunction::Meta(using));
	scope.insert(FnLabel::name("let"),       ServFunction::Meta(using));
	scope.insert(FnLabel::name("choose"),    ServFunction::Meta(choose));
	scope.insert(FnLabel::name("*"),         ServFunction::Meta(apply));
	scope.insert(FnLabel::name("exec.pipe"),         ServFunction::Meta(exec_pipe));
	scope.insert(FnLabel::name("with_header"),         ServFunction::Meta(with_header));
	scope.insert(FnLabel::name("with_status"),         ServFunction::Meta(with_status));
	scope.insert(FnLabel::name("fold"),    ServFunction::Meta(fold));
	scope.insert(FnLabel::name("get"),    ServFunction::Meta(get));
	scope.insert(FnLabel::name("switch"),    ServFunction::Meta(switch));
	scope.insert(FnLabel::name("render"),    ServFunction::Meta(render));
	scope.insert(FnLabel::name("join"),       ServFunction::Meta(join));
	scope.insert(FnLabel::name("split"),    ServFunction::Meta(split));
}
