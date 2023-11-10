use crate::parser::{FromSyntax, ParseError, Tree};
use std::collections::HashMap;
use std::convert::TryFrom;
use hyper;

use crate::value::{Table, Response, Value};

use crate::template::Template;

pub enum Operator {
	Html,
	File,
}

impl TryFrom<&str> for Operator {
	type Error = &'static str;

	fn try_from(value: &str) -> Result<Self, Self::Error> {
		let output = match value {
			"html" => Operator::Html,
			"file" => Operator::File,
			_ => return Err("invalid operator name"),
		};

		Ok(output)
	}
}

pub enum AstNode {
	Placeholder,
	Template(Template),
	Text(String),
	Prefix {
		op: Operator,
		options: HashMap<String, String>,
		value: Option<Box<AstNode>>
	},
}

pub struct ExecutionEngine {
	vars: Table,
}

impl ExecutionEngine {

	pub fn new(input: Table) -> Self {
		Self {
			vars: input,
		}
	}

	pub fn resolve_expression(&mut self, expression: &AstNode) -> Result<Value, ()> {

		let output: Value = match expression {
			AstNode::Text(s) => Value::Text(s.trim().to_owned()),
			AstNode::Template(t) => Value::Text(t.render(&self.vars)),
			AstNode::Prefix { op, options, value } => { 
				let input: Option<Value> = match value {
					Some(v) => Some(self.resolve_expression(v)?),
					None => None,
				};

				match op {
					Operator::Html => {
						let mut output = String::new();
						output.push_str("<html>\n<body>\n");
						if let Some(v) = input {
							output.push_str(&v.to_string());
						}
						output.push_str("\n</body>\n</html>");

						Value::Text(output)
					},
					Operator::File => {
						let path_v = input.ok_or(())?;
						let path = path_v.to_string();
						let body = std::fs::read_to_string(path.trim()).map_err(|_| ())?;
						Value::Text(body)
					},
					_ => todo!(),
				}
			},

			_ => todo!(),
		};

		Ok(output)
	}
}

#[cfg(test)]
mod test {
	
	#[test]
	fn test() {}
}
