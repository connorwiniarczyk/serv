use crate::lexer::{Token, TokenKind};
use crate::value::ServValue;
use crate::ast;
use crate::dictionary::Scope;

use crate::eval;

use crate::{ Context };

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
    pub fn render(&self, ctx: &Context) -> Result<ServValue, &'static str>{
        let mut output = String::new();
        for elem in self.elements.iter() {
            match elem {
                TemplateElement::Text(t) => output.push_str(&t.contents),
                TemplateElement::Variable(v) => {
                    ctx.get(&v.contents).ok_or("does not exist")?.call(ServValue::Int(0), ctx)?;
                    todo!();
                },
                TemplateElement::Expression(t) => output.push_str("exp"),
                TemplateElement::Template(t) => {
                    let ServValue::Text(inner) = t.render(ctx)? else { unreachable!() };
                    output.push_str(inner.as_str())
                },
            }
        }

		Ok(ServValue::Text(output))
    }
}
