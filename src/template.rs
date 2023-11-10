use crate::parser::{ Tree, ParseError, FromSyntax };
use std::sync::Arc;
use crate::parser::Rule;
use std::collections::HashMap;

use crate::value::Value;
use crate::value::Table;

#[derive(Debug)]
enum TemplateElement<'a> {
	Literal(&'a str),
	Variable(&'a str),
}

use TemplateElement::*;

pub struct Template {
	content: &'static str,
	elements: Vec<TemplateElement<'static>>,
}

impl FromSyntax for Template {
	fn from_syntax(input: Tree) -> Result<Self, ParseError> {

		let content: &'static str = input.as_str().to_owned().leak();
		let mut output = Self {
			content,
			elements: Vec::new(),
		};

		let template_start: usize = input.as_span().start();

		for pair in input.into_inner() {
			match pair.as_rule() {
				Rule::template_text => {
					let s = pair.as_span().start() - template_start;
					let e = pair.as_span().end() - template_start;
					output.elements.push(TemplateElement::Literal(&content[s..e]));
				},
				Rule::template_variable => {
					let ident = pair.into_inner().next().unwrap();
					let s = ident.as_span().start() - template_start;
					let e = ident.as_span().end() - template_start;
					output.elements.push(TemplateElement::Variable(&content[s..e]));
				},
				Rule::template_expression => {},
				_ => unreachable!(),
			}
		}

		return Ok(output)
	}
}

impl Template {
	// pub fn render(&self, vars: &HashMap<String, Value>) -> String {
	pub fn render(&self, vars: &Table) -> String {
		let mut output = String::new();

		for piece in self.elements.iter() {
			match piece {
				Literal(l) => { 
					output.push_str(l);
				}, 
				Variable(v) => { 
					let value = vars.get(&v.to_string()).unwrap_or_default();
					output.push_str(&value.to_string());
				}, 
			}
		}

		return output
	}
}

