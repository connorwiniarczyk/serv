use super::ServFunctionTrait;

use crate::serv_value::ServValue;
use crate::Context;
use std::sync::Arc;

use crate::AstNode;
use crate::ServFunction;
use crate::Compiler;
use crate::Words;

use std::process::{Command, Stdio};

use std::io::Write;
use std::io::Read;

pub struct Exec;
impl ServFunctionTrait for Exec {
	fn call(&self, input: ServValue, ctx: &mut Context) -> Result<ServValue, &'static str> {

		let mut args = input.as_str().split_whitespace();
		let mut cmd = Command::new(args.next().unwrap());
		cmd.args(args);

		let output = cmd.output().expect("!");
		Ok(ServValue::Bin(output.stdout))
	}
}


pub struct PipeInto(pub ServFunction);
impl ServFunctionTrait for PipeInto {
	fn call(&self, input: ServValue, ctx: &mut Context) -> Result<ServValue, &'static str> {

		let arg_result = self.0.call(ServValue::None, ctx)?;
		let mut args = arg_result.as_str().split_whitespace();
		let mut cmd = Command::new(args.next().unwrap());
		cmd.args(args);

		match input {
			ServValue::None => {
				let output = cmd.output().expect("!");
				Ok(ServValue::Bin(output.stdout))
			},
			i @ _ => {
				cmd.stdin(Stdio::piped());
				cmd.stdout(Stdio::piped());
				let out = cmd.spawn().expect("!");
				out.stdin.unwrap().write_all(i.as_bytes());

				let mut bytes = Vec::new();
				out.stdout.unwrap().read_to_end(&mut bytes);
				Ok(ServValue::Bin(bytes))
			},
		}
	}
}

impl PipeInto {
	pub fn new(arg: AstNode, compiler: &Compiler) -> Self {
		let func = compiler.compile_single_word(arg, &mut Words::empty().peekable()).unwrap();
		Self(func)
	}
}
