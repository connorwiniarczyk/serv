use crate::lexer::{Token, TokenKind};
use crate::value::ServValue;
use crate::ast;
use crate::{ Scope, ServResult };

use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum TemplateElement {
	Text(Token),
	Template(Template),
	Expression(ast::Word),
}

impl Display for TemplateElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
    		TemplateElement::Text(t)       => f.write_str(t.contents.as_str()),
            TemplateElement::Template(t)   => t.fmt(f),
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
                TemplateElement::Template(t) => output.push_str(t.literal().to_string().as_str()),
                // TemplateElement::Template(t) => output.push_str(t.render(ctx)?.to_string().as_str()),
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
            }
        }

		let formatted = format_text(output.as_str());
		Ok(ServValue::Text(formatted))
    }
}

pub fn format_text(input: &str) -> String {
    let mut output = String::new();
    let mut iter = input.lines().skip_while(|line| *line == "").peekable();

	let indent_level = iter.clone().map(|line| {
    	let mut chars = line.chars();
    	let mut level = 0;
    	while chars.next() == Some(' ') { level += 1 };
    	level
	}).min().unwrap_or(0);

	while let Some(line) = iter.next()  {
    	let mut chars = line.chars();
    	chars.skip(indent_level).for_each(|c| output.push(c));
    	if iter.peek().is_some() { output.push('\n') };
	}

	output
}
