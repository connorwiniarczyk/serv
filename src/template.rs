use crate::lexer::{Token, TokenKind};
use crate::value::ServValue;
use crate::ast;
use crate::{ Scope, ServResult };

use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum TemplateElement {
	Text(Token),
	Variable(Token),
	Template(Template),
	Expression(ast::Word),
}

impl Display for TemplateElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
    		TemplateElement::Text(t)     => f.write_str(t.contents.as_str()),
            TemplateElement::Template(t) => t.fmt(f),
            TemplateElement::Variable(t) => f.write_str(t.contents.as_str()),
            TemplateElement::Expression(e) => {
                f.write_str("$(")?;
    			e.fmt(f)?;
                f.write_str(")")?;
                Ok(())
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct Template {
    pub open: Token,
    pub close: Token,
    pub elements: Vec<TemplateElement>,
}

impl Display for Template {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.write_str(self.open.contents.as_str())?;
        for element in self.elements.iter() {
            element.fmt(f)?;
        }
        f.write_str(self.close.contents.as_str())?;
        Ok(())
    }
}

impl Template {
    pub fn literal(&self) -> ServValue {
        ServValue::Text(self.to_string())
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
