pub mod exec;
pub mod template;
pub mod serv_function;

pub use serv_function::{ ServFunction, ServFunctionTrait, FnWrapper };

use crate::ServValue;
use crate::Context;
use crate::Compiler;

use crate::parser;
use crate::parser::{AstNode};

use std::iter::Peekable;
use std::sync::Arc;

use evalexpr::eval;

use crate::Words;

pub struct Reference(pub String);
impl ServFunctionTrait for Reference {
	fn call(&self, input: ServValue, ctx: &mut Context) -> Result<ServValue, &'static str> {
		let word_value = ctx.get(self.0.as_str()).ok_or("tried to lookup word that wasn't defined")?;
		Ok(word_value.call(input, ctx)?)
	}
}

pub struct ReadFile;
impl ServFunctionTrait for ReadFile {
	fn call(&self, input: ServValue, ctx: &mut Context) -> Result<ServValue, &'static str> {
		let file_contents = std::fs::read_to_string(input.as_str()).map_err(|x| "file not found!")?;
		Ok(file_contents.into())
	}
}

pub struct Eval;
impl ServFunctionTrait for Eval {
	fn call(&self, input: ServValue, ctx: &mut Context) -> Result<ServValue, &'static str> {
		let ServValue::Text(ref expr) = input else { return Err("!") };
		let res = eval(expr).unwrap();
		Ok(match res {
			evalexpr::Value::String(s) => ServValue::Text(s),
			evalexpr::Value::Int(x) => ServValue::Int(x),
			evalexpr::Value::Boolean(x) => ServValue::Boolean(x),
			evalexpr::Value::Float(x) => ServValue::Float(x),
			evalexpr::Value::Empty => ServValue::None,
			_ => todo!(),
		})
	}
}

pub struct Identity;
impl ServFunctionTrait for Identity {
	fn call(&self, input: ServValue, ctx: &mut Context) -> Result<ServValue, &'static str> {
		Ok(input)
	}
}

pub struct Where;
impl ServFunctionTrait for Where {
	fn call(&self, input: ServValue, ctx: &mut Context) -> Result<ServValue, &'static str> {
		let text = input.to_string();
		let table = parser::parse_root_from_text(&text).unwrap();
		for decl in table.declarations() {
 			let AstNode::Declaration { kind, key, value} = decl else {panic!()};
			let f = ctx.interpreter().compile(*value).unwrap();
			ctx.push(key, f);
		}
		Ok(ServValue::None)
	}
}

pub struct List(pub Vec<ServFunction>);
impl ServFunctionTrait for List {
	fn call(&self, input: ServValue, ctx: &mut Context) -> Result<ServValue, &'static str> {
		let mut output = Vec::new();
		for func in self.0.iter() {
			output.push(func.call(ServValue::None, ctx)?);
		}

		if let ServValue::Int(i) = input {
			Ok(output.remove(i.try_into().unwrap()))
		} else {
			Ok(ServValue::List(output))
		}
	}
}

pub struct Sum;
impl ServFunctionTrait for Sum {
	fn call(&self, input: ServValue, ctx: &mut Context) -> Result<ServValue, &'static str> {
		if let ServValue::List(members) = input {
			let mut sum = 0;
			for value in members {
				if let ServValue::Int(i) = value {
					sum += i;
				}
			}

			Ok(ServValue::Int(sum))
		}

		else {
			Err("not a list")
		}
	}
}

pub struct Get(pub ServFunction);
impl ServFunctionTrait for Get {
	fn call(&self, input: ServValue, ctx: &mut Context) -> Result<ServValue, &'static str> {
		match input {
			ServValue::List(mut members) => {
				let index_value = self.0.call(ServValue::None, ctx)?;
				let ServValue::Int(i) = index_value else { return Err("!") };
				Ok(members.remove(i.try_into().unwrap()))
			},
			_ => Err("not a list"),
		}
	}
}

pub struct Clamp(pub ServFunction);
impl ServFunctionTrait for Clamp {
	fn call(&self, input: ServValue, ctx: &mut Context) -> Result<ServValue, &'static str> {

		let ServValue::Int(x) = input else { return Err("!") };
		let index_value = self.0.call(ServValue::None, ctx)?;
		let ServValue::Int(y) = index_value else { return Err("!") };

		Ok(ServValue::Int(std::cmp::min(x, y)))

	}
}

pub struct ServEval;
impl ServFunctionTrait for ServEval {
	fn call(&self, input: ServValue, ctx: &mut Context) -> Result<ServValue, &'static str> {
		let text: String = input.try_into().unwrap();
		let expr = crate::parser::parse_expression_from_text(text.as_str()).unwrap();
		let compiler = crate::compiler::Compiler::new();
		let func = compiler.compile(expr).unwrap();
		let res = func.call(ServValue::None, ctx).unwrap();
		Ok(res)
	}
}

pub struct Switch {
    pub select: ServFunction,
    pub options: ServFunction,
}
impl Switch {
	pub fn new(input: &mut Peekable<Words>, compiler: &Compiler) -> Result<ServFunction, &'static str> {
        let select = compiler.compile_single_word(input.next().unwrap(), input)?;
        let options = compiler.compile_single_word(input.next().unwrap(), input)?;
        Ok(ServFunction::new(Self { select, options }))
	}
}

impl ServFunctionTrait for Switch {
	fn call(&self, input: ServValue, ctx: &mut Context) -> Result<ServValue, &'static str> {
        let select: i64 = self.select.call(ServValue::None, ctx)?.try_into().unwrap();
        let mut options: Vec<ServValue> = self.options.call(ServValue::None, ctx)?.try_into().unwrap();

        let chosen = options.swap_remove(select.try_into().unwrap());
		let expr = crate::parser::parse_expression_from_text(chosen.as_str()).unwrap();
        let func = ctx.interpreter().compile(expr).unwrap();
        
        func.call(input, ctx)
	}
}


pub struct Split(ServFunction);
impl Split {
	fn new(input: &mut Peekable<Words>) -> Result<ServFunction, &'static str> {
		let inner = match input.next().ok_or("!")? {
			AstNode::Template { literal, .. } => ServFunction::new(literal),
			_ => todo!(),
		};
		Ok(ServFunction::new(Self(inner)))
	}
}
impl ServFunctionTrait for Split {
	fn call(&self, input: ServValue, ctx: &mut Context) -> Result<ServValue, &'static str> {
		let separator = self.0.call(ServValue::None, ctx).expect("failed to evaluate separator");
		let ServValue::Text(ref t) = input else { return Err("!") };
		let res = ServValue::List(t.split(separator.as_str()).map(|x| ServValue::Text(x.to_owned())).collect());
		Ok(res)
	}
}

pub struct Log;
impl ServFunctionTrait for Log {
	fn call(&self, input: ServValue, ctx: &mut Context) -> Result<ServValue, &'static str> {
        println!("{:?}", input);
        Ok(input)
	}
}
