use crate::lexer::{Token, TokenKind};
use crate::value::ServValue;
use crate::ast;
// use crate::dictionary::StackDictionary;
// use crate::engine::{ServFn};

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
    pub fn render(&self, ctx: &Scope) -> ServResult {
        let mut output = String::new();
        for elem in self.elements.iter() {
            match elem {
                TemplateElement::Text(t) => output.push_str(&t.contents),
                TemplateElement::Variable(v) => {
                    let value = ctx.get(&v.contents.clone().into()).ok_or("does not exist")?.call(ServValue::None, ctx)?;
                    output.push_str(value.as_str())
                },
                // TemplateElement::Expression(t) => output.push_str("exp"),
                TemplateElement::Expression(t) => {
					println!("{:?}", t);
                    match t {
                        ast::Word::Function(token) => {
                            println!("{:?}", token);
                            let value = ctx.get(&token.contents.clone().into()).ok_or("does not exist")?.call(ServValue::None, ctx)?;
                            output.push_str(value.as_str());
                        },
                        _ => todo!(),
                    }
                },
                TemplateElement::Template(t) => {
                    let ServValue::Text(inner) = t.render(ctx)? else { unreachable!() };
                    output.push_str(inner.as_str())
                },
            }
        }

		Ok(ServValue::Text(output))
    }
}
