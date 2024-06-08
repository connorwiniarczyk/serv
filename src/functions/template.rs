use std::iter::Peekable;
use crate::parser::AstNode;

use crate::serv_value::ServValue;
// use crate::expression::Context;

use crate::Context;
use super::ServFunctionTrait;

#[derive(Debug, Clone)]
pub enum TemplateElement {
	Text(String),
	Var(String),
}

#[derive(Debug, Clone)]
pub struct Template {
	original: String,
	elements: Vec<TemplateElement>,
}

impl Template {
	pub fn render(&self, input: ServValue, ctx: &mut Context) -> ServValue {
		let text = input.to_string();
		let mut output = String::new();
		for elem in self.elements.iter() {
			match elem {
				TemplateElement::Text(s) => output.push_str(&s),
				TemplateElement::Var(ref name) if name == "" => output.push_str(&text),
				TemplateElement::Var(ref name) => {
					let word = ctx.get(name).unwrap();
					// output.push_str(word.call(ServValue::None, &mut ctx.clone()).unwrap().to_string().as_str());
					output.push_str(word.call(ServValue::None, ctx).unwrap().to_string().as_str());
				},
			};
		}
		ServValue::Text(output.chars().collect())
	}

	pub fn from_elements(literal: String, input: Vec<AstNode>) -> Result<Self, &'static str> {
		let elements: Vec<TemplateElement> = input.into_iter().map(|x| match x{
			AstNode::TemplateText(s) => TemplateElement::Text(s),
			AstNode::TemplateVar(s) => TemplateElement::Var(s),
			_ => todo!(), //return Err("invalid ast node type"),
		}).collect();

		Ok(Self {
			original: literal,
			elements: elements,
		})
	}

	pub fn as_literal(&self) -> String {
		todo!();
	}
}

impl ServFunctionTrait for Template {
	fn call(&self, input: ServValue, ctx: &mut Context) -> Result<ServValue, &'static str> {

		if let ServValue::List(elements) = input {
			let new_elements = elements.into_iter().map(|e| self.call(e, ctx).unwrap()).collect();
			Ok(ServValue::List(new_elements))
		}

		else {
			let text = input.to_string();
			let mut output = String::new();
			for elem in self.elements.iter() {
				match elem {
					TemplateElement::Text(s) => output.push_str(&s),
					TemplateElement::Var(ref name) if name == "" => output.push_str(&text),
					TemplateElement::Var(ref name) => {
						let word = ctx.get(name).unwrap();
						output.push_str(word.call(ServValue::None, ctx).unwrap().to_string().as_str());
					},
				};
			}

			Ok(ServValue::Text(output.chars().collect()))
		}

	}
}
