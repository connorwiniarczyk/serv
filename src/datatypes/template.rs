use crate::value::ServValue;
use crate::{ Stack, ServResult, ServFn };
use std::fmt::Display;
use crate::value::Serializer;

type Buffer<'a> = dyn std::fmt::Write + 'a;

pub trait Renderer {
    fn render<'b>(&mut self, input: &Template, dest: &'b mut Buffer<'b>);
}

#[derive(Clone)]
pub struct DefaultRenderer<'scope, S: Serializer + Clone> {
    include_brackets: bool,
    resolve_expressions: bool,
	serializer: S,
	scope: &'scope Stack<'scope>
}

use crate::functions::json;

impl<'a> DefaultRenderer<'a, json::JsonSerializer<'a>> {
    pub fn new(scope: &'a Stack<'a>) -> Self {
        Self {
            include_brackets: false,
            resolve_expressions: true,
            serializer: json::serializer(scope),
            scope: scope,
        }
    }
}

impl<'scope, S: Serializer + Clone> Renderer for DefaultRenderer<'scope, S> {
    fn render<'b>(&mut self, input: &Template, dest: &'b mut Buffer<'b>) {
        if self.include_brackets {
            dest.write_str(&input.open);
        }

        for elem in input.elements.iter() {
            match elem {
                TemplateElement::Text(t)     => dest.write_str(t).unwrap(),
                TemplateElement::Template(t) => {
                    let mut child = self.clone();
                    child.include_brackets = true;
                    child.render(t, dest);
                },

                TemplateElement::Expression(t) if self.resolve_expressions => {
                    let input = self.scope.get("in").ok();
                    let value = t.call(input, self.scope).unwrap();
                    dest.write_str(&value.to_string());
                },

                TemplateElement::Expression(t) => {
                    dest.write_str("$(");
    				dest.write_str(&t.to_string());
                    dest.write_str(")");
                },
            };
        }

        if self.include_brackets {
            dest.write_str(&input.close);
        }
    }
}

#[derive(Clone)]
pub struct LiteralRenderer {
    include_brackets: bool,
}

impl Renderer for LiteralRenderer {
    fn render<'b>(&mut self, input: &Template, dest: &'b mut Buffer<'b>) {
        if self.include_brackets {
            dest.write_str(&input.open);
        }

        for elem in input.elements.iter() {
            match elem {
                TemplateElement::Text(t)     => dest.write_str(t).unwrap(),
                TemplateElement::Template(t) => {
                    let mut child = self.clone();
                    child.include_brackets = true;
                    child.render(t, dest);
                },
                TemplateElement::Expression(t) => {
                    dest.write_str("$(");
    				dest.write_str(&t.to_string());
                    dest.write_str(")");
                },
            };
        }

        if self.include_brackets {
            dest.write_str(&input.close);
        }
    }
}


#[derive(Clone)]
pub enum TemplateElement {
	Text(String),
	Template(Template),
	Expression(ServValue),
}


#[derive(Clone)]
pub struct Template {
    pub open: String,
    pub close: String,
    pub elements: Vec<TemplateElement>,
}

impl Template {
    pub fn render(&self, ctx: &Stack) -> ServResult {
        let mut output = String::new();
        let mut renderer = DefaultRenderer::new(ctx);

        renderer.render(self, &mut output);
		Ok(ServValue::Text(output.into()))
    }
}

impl Display for Template {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let mut r = LiteralRenderer { include_brackets: true };
        r.render(self, f);
        Ok(())
    }
}
