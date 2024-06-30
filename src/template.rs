use crate::lexer::{Token, TokenKind};
use crate::value::ServValue;
use crate::ast;

use crate::{ Scope, ServResult };

#[derive(Debug, Clone)]
pub enum TemplateElement {
	Text(Token),
	Variable(Token),
	Template(Template),
	Expression(ast::Word),
}

#[derive(Debug, Clone)]
pub struct Template {
    pub open: Token,
    pub close: Token,
    pub elements: Vec<TemplateElement>,
}

impl Template {

    pub fn literal(&self) -> ServValue {
		let mut output = String::new();
		output.push_str(self.open.contents.as_str());
		for e in self.elements.iter() {
    		match e {
        		TemplateElement::Text(t)     => output.push_str(t.contents.to_string().as_str()),
                TemplateElement::Template(t) => output.push_str(t.literal().to_string().as_str()),
        		_ => todo!(),
    		}
		}
		output.push_str(self.close.contents.as_str());

		ServValue::Text(output)
    }

    pub fn render(&self, ctx: &Scope) -> ServResult {
        let mut output = String::new();
        for elem in self.elements.iter() {
            match elem {
                TemplateElement::Text(t) => output.push_str(&t.contents),
                TemplateElement::Variable(v) => {
                    let value = ctx.get(&v.contents.clone().into()).ok_or("does not exist")?.call(ServValue::None, ctx)?;
                    output.push_str(value.to_string().as_str())
                },
                TemplateElement::Expression(t) => {
                    match t {
                        ast::Word::Function(token) => {
                            let value = ctx.get(&token.contents.clone().into()).ok_or("does not exist")?.call(ServValue::None, ctx)?;
                            output.push_str(value.to_string().as_str());
                        },
                        ast::Word::Parantheses(words) => {
                            // println!("expr {:?}", words);
                            let mut child = ctx.make_child();
                        	let func = crate::compile(words.0.clone(), &mut child);
                        	let value = func.call(ctx.get_str("in")?.call(ServValue::None, &child)?, &child)?;
                            output.push_str(value.to_string().as_str());
                        },
                        _ => todo!(),

                    }
                },
                TemplateElement::Template(t) => {
                    output.push_str(t.literal().to_string().as_str())
                },
            }
        }

		Ok(ServValue::Text(output))
    }
}
