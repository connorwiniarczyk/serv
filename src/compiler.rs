use crate::ast::AstNode;
use crate::functions::{ ServFunctionTrait, ServFunction, FnWrapper };
use crate::functions::template::Template;
use crate::functions::Reference;
use crate::functions::exec::{Exec, PipeInto};

use crate::Context;
use crate::serv_value::ServValue;

use std::sync::Arc;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::iter::Peekable;


trait WordCompiler: Send + Sync {
	fn compile(&self, words: &mut Peekable<Words>) -> Result<ServFunction, &'static str>;
}


impl WordCompiler for ServFunction {
	fn compile(&self, words: &mut Peekable<Words>) -> Result<ServFunction, &'static str> {
		Ok(self.clone())
	}
}

impl<C> WordCompiler for C
where C: Send + Sync + Fn(&mut Peekable<Words>) -> Result<ServFunction, &'static str> {
	fn compile(&self, words: &mut Peekable<Words>) -> Result<ServFunction, &'static str> {
		(self)(words)
	}
}

use crate::functions;

pub struct Compiler(HashMap<String, Arc<dyn WordCompiler>>);
impl Compiler {
	pub fn new() -> Self {
		let mut output = Self(HashMap::new());
		Self(HashMap::new())
	}

	pub fn compile_single_word(&self, input: AstNode, iter: &mut Peekable<Words>) -> Result<ServFunction, &'static str> {
		if let Word(ref name) = input {
			let output = match name.as_str() {
				"hello" => ServFunction::new("Hello World!"),
				"ident" => ServFunction::new(functions::Identity),
				"drop" => { _ = iter.next(); ServFunction::new(functions::Identity)},

				"incr" => FnWrapper::new(|x: i64| x + 1),
				"decr" => FnWrapper::new(|x: i64| x - 1),
				"clamp" => {
					let max_word = iter.next().ok_or("!")?;
					let max_val = self.compile_single_word(max_word, iter)?;
					ServFunction::new(functions::Clamp(max_val))
				},

				"uppercase" => FnWrapper::new(|x: String| x.to_uppercase()),
				"lowercase" => FnWrapper::new(|x: String| x.to_lowercase()),

				"eval" => ServFunction::new(functions::ServEval),

				"read" => ServFunction::new(crate::functions::ReadFile),
				"where" => ServFunction::new(crate::functions::Where),
				"%" => ServFunction::new(crate::functions::Eval),
				"sum" => ServFunction::new(functions::Sum),

				"log" => ServFunction::new(functions::Log),

				"switch" => functions::Switch::new(iter, self)?,

				"get" => {
					let arg = iter.next().ok_or("missing argument for get")?;
					ServFunction::new(functions::Get(self.compile_single_word(arg, iter)?))
				},

				"exec" => ServFunction::new(Exec),
				"pipe" => {
					let arg = iter.next().ok_or("missing argument for exec")?;
					ServFunction::new(PipeInto::new(arg, &self))
				},

				"markdown" => FnWrapper::new(|x: String| {
					markdown::to_html_with_options(x.as_str(), &markdown::Options::gfm()).unwrap()
				}),

				name @ _ => ServFunction::new(Reference(name.to_string())),
			};

			Ok(output)
		} else if let AstNode::Template {literal, elements} = input {
			Template::from_elements(literal, elements).map(ServFunction::new)
		} else if let AstNode::IntLiteral(i) = input {
			Ok(ServFunction::new(i))

		} else if let AstNode::ListLiteral(list) = input {
			let mut inner = Vec::new();
			for word in list {
				inner.push(self.compile_single_word(word, &mut Words::empty().peekable())?);
			}
			Ok(ServFunction::new(functions::List(inner)))
		} else {
			todo!()
		}
	} 

	pub fn compile(&self, expression: AstNode) -> Result<ServFunction, &'static str> {
		let mut words = expression.words().peekable();
		let mut output: Vec<ServFunction> = Vec::new();

		while let Some(word) = words.next() {
			output.push(self.compile_single_word(word, &mut words)?);
		}

		Ok(ServFunction::new(output))
	}
}

