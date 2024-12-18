use crate::value::ServValue;
use crate::{ Stack, ServResult, ServFn };
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

#[derive(Clone)]
pub enum TemplateElement {
	Text(String),
	Template(Template),
	// Expression(ast::Word),
	Expression(ServValue),
}

impl Display for TemplateElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
    		TemplateElement::Text(t)       => f.write_str(t.as_str()),
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

#[derive(Clone)]
pub struct Template {
    pub open: String,
    pub close: String,
    pub elements: Vec<TemplateElement>,
}

impl Display for Template {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.write_str(self.open.as_str())?;
        for element in self.elements.iter() {
            element.fmt(f)?;
        }
        f.write_str(self.close.as_str())?;
        Ok(())
    }
}


impl Template {
    pub fn literal(&self) -> ServValue {
        ServValue::Text(self.to_string())
    }

    pub fn literal_inner(&self) -> ServValue {
        let empty = crate::StackDictionary::empty();
        let mut renderer = Renderer::new(&empty);
        let mut options = FormatOptions::default();
        options.resolve_functions = false;
        renderer.render(self, options);

		ServValue::Text(std::mem::take(&mut renderer.output))
    }

    pub fn render(&self, ctx: &Stack) -> ServResult {
        let mut renderer = Renderer::new(ctx);
        let options = FormatOptions::default();
        renderer.render(self, options);
		Ok(ServValue::Text(std::mem::take(&mut renderer.output)))
    }

    pub fn render_sql<'scope>(&self, ctx: &'scope Stack) -> (Stack<'scope>, String, Vec<crate::Label>) {
        let mut renderer = Renderer::new(ctx);
        renderer.sql_bindings = Some(Vec::new());
        renderer.new_context = Some(renderer.ctx.make_child());

        let mut options = FormatOptions::default();
        options.sql_mode = true;

        renderer.render(self, options);

        let output = std::mem::take(&mut renderer.output);
        let bindings = std::mem::take(&mut renderer.sql_bindings).unwrap();
        let new_context = std::mem::take(&mut renderer.new_context).unwrap();

		(new_context, output, bindings)
    }
}

struct Renderer<'scope> {
    output: String,
    sql_bindings: Option<Vec<crate::Label>>,
    ctx: &'scope Stack<'scope>,
    new_context: Option<Stack<'scope>>,
}

impl<'scope> Renderer<'scope> {
    fn new(ctx: &'scope Stack) -> Self {
		Self {
    		output: String::new(),
    		ctx: ctx,
    		sql_bindings: None,
    		new_context: None,
		}
    }

	fn render(&mut self, input: &Template, options: FormatOptions) {
        if options.include_brackets { self.output.push_str(&input.open); }
        for elem in input.elements.iter() {
            match elem {
                TemplateElement::Text(t)     => self.output.push_str(t),
                TemplateElement::Template(t) => self.render(t, options.with_brackets()),

                // be careful, order is important here
                TemplateElement::Expression(ServValue::Ref(label)) if options.sql_mode => {
                    self.output.push('?');
                    if let Some(ref mut sql_bindings) = &mut self.sql_bindings {
                        // sql_bindings.push(crate::Label::Name(t.clone()));
                        sql_bindings.push(label.clone());
                    }
                },

                TemplateElement::Expression(word) if options.sql_mode => {
                    let Some(mut ctx)  = self.new_context.as_mut() else { return };
                    // let mut expr = crate::module::Expression::compile(e.clone());

                    self.output.push('?');
                    if let Some(ref mut sql_bindings) = &mut self.sql_bindings {
                        // sql_bindings.push(ctx.insert_anonymous(ServValue::Func(crate::ServFn::Expr(expr, false))));
                        sql_bindings.push(ctx.insert_anonymous(word.clone()));
                    }
                },

                TemplateElement::Expression(t) if options.resolve_functions => {
                    let value = t.call(None, self.ctx).unwrap();
                    self.output.push_str(&value.to_string());
                },
                TemplateElement::Expression(t) => {
                    self.output.push('$');
                    self.output.push('(');
					self.output.push_str(&t.to_string());
                    self.output.push(')');
                },
            }
        }
        if options.include_brackets { self.output.push_str(&input.close); }
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
