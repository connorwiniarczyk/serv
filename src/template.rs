use crate::lexer::{Token, TokenKind};
use crate::value::ServValue;
use crate::ast;
use crate::{ Scope, ServResult };
use std::fmt::Display;

#[derive (Clone)]
struct FormatOptions {
	include_brackets: bool,
	resolve_functions: bool,
	sql_mode: bool,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
        	include_brackets: false,
        	resolve_functions: true,
        	sql_mode: false,
        }
    }
}

impl FormatOptions {
    fn with_brackets(&self) -> Self {
        let mut output = self.clone();
        output.include_brackets = true;
        output
    }
}

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
        let mut renderer = Renderer { output: String::new(), ctx, sql_bindings: None };
        let options = FormatOptions::default();
        renderer.render(self, options);
		Ok(ServValue::Text(std::mem::take(&mut renderer.output)))
    }

    pub fn render_sql(&self, ctx: &Scope) -> (String, Vec<crate::FnLabel>) {
        let mut renderer = Renderer::new(ctx);
        renderer.sql_bindings = Some(Vec::new());

        let mut options = FormatOptions::default();
        options.sql_mode = true;

        renderer.render(self, options);

        let output = std::mem::take(&mut renderer.output);
        let bindings = std::mem::take(&mut renderer.sql_bindings).unwrap();

		(output, bindings)
    }
}

struct Renderer<'scope> {
    output: String,
    ctx: &'scope Scope<'scope>,
    sql_bindings: Option<Vec<crate::FnLabel>>,
}

impl<'scope> Renderer<'scope> {
    fn new(ctx: &'scope Scope) -> Self {
		Self {
    		output: String::new(),
    		ctx: ctx,
    		sql_bindings: None,
		}
    }
    fn resolve_function(&mut self, expression: &ast::Word) -> ServResult {
        match expression {
            ast::Word::Function(token) => {
                let value = self.ctx.get(&token.contents.clone().into())
                    .ok_or("does not exist")?
                    .call(ServValue::None, &self.ctx)?
                    .to_string();

                self.output.push_str(&value);
            },

            ast::Word::Parantheses(words) => {
                let mut child = self.ctx.make_child();
            	let func = crate::compile(words.0.clone(), &mut child);
            	let value = func.call(self.ctx.get_str("in")?.call(ServValue::None, &child)?, &child)?;
                self.output.push_str(value.to_string().as_str());
            },

            _ => unreachable!(),
        }

        Ok(ServValue::None)
    }

	fn render(&mut self, input: &Template, options: FormatOptions) {
        if options.include_brackets { self.output.push_str(&input.open.contents); }
        for elem in input.elements.iter() {
            match elem {
                TemplateElement::Text(t)     => self.output.push_str(&t.contents),
                TemplateElement::Template(t) => self.render(t, options.with_brackets()),
                TemplateElement::Expression(t) if options.resolve_functions => {
                    self.resolve_function(t).unwrap();
                },
                TemplateElement::Expression(ast::Word::Function(t)) if options.sql_mode => {
                    self.output.push('?');
                    if let Some(ref mut sql_bindings) = &mut self.sql_bindings {
                        sql_bindings.push(crate::FnLabel::Name(t.contents.clone()));
                    }
                },
                TemplateElement::Expression(t) => {
                    self.output.push('$');
                    self.output.push('(');
					self.output.push_str(&t.to_string());
                    self.output.push(')');
                },
            }
        }
        if options.include_brackets { self.output.push_str(&input.close.contents); }
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
