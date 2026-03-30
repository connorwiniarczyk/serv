use crate::template::{Template, TemplateElement};
use crate::{ ServValue, ServFn, Label };
use crate::datatypes::module::{ ServModule };
use crate::datatypes::module;
use crate::datatypes::value;
use crate::datatypes::value::ServList;
use crate::engine::dictionary::Address;
use crate::engine;
use crate::Stack;

use super::tokenizer::{TokenKind, TokenKind::*};
use super::tokenizer;
use super::cursor::Token;
use super::ParseError;

use crate::error::ServError;

pub type Parser<'a> = super::walker::Walker<'a, TokenKind>;

fn parse_template(parser: &mut Parser) -> Result<Template, ServError> {
    let open = parser.expect(TokenKind::TemplateOpen)?.to_string();
    parser.incr()?;

	let mut elements: Vec<TemplateElement> = Vec::new();
	while parser.current()?.kind != TokenKind::TemplateClose {
    	let token = parser.current()?.clone();
    	match token.kind {
        	TokenKind::TemplateLineBreak => {
            	elements.push(TemplateElement::Text(token.to_string()));
            	parser.incr();
        	},

        	TokenKind::TemplateText => {
            	elements.push(TemplateElement::Text(token.to_string()));
            	parser.incr();
        	},
        	TokenKind::Dollar => elements.push({
            	parser.incr();
            	TemplateElement::Expression(parse_word(parser, &mut Stack::empty())?)
        	}),

        	TokenKind::TemplateOpen => {
            	elements.push(TemplateElement::Template(parse_template(parser)?));
            	parser.incr();
        	},
        	TokenKind::TemplateClose => { unreachable!(); },
        	_ => return Err("token not supported in template".into()),
    	}
	}

	let close = parser.expect(TokenKind::TemplateClose).unwrap().to_string();
	Ok(Template { open, close, elements })
}

fn constant(parser: &mut Parser, ctx: &mut Stack) -> Result<ServValue, ServError> {
    let mut expr = ServList::new();
    while let Ok(word) = parse_word(parser, ctx) {
        expr.push_back(word);
    }

	return engine::eval(expr, ctx)
}

fn parse_word(parser: &mut Parser, ctx: &mut Stack) -> Result<ServValue, ServError> {
    let token = parser.get(0)?;
    let output = match token.kind {
        TokenKind::Identifier   => ServValue::Ref(token.to_string().as_str().into()),
        TokenKind::Route        => ServValue::Ref(Label::Route(token.to_string()).into()),
        TokenKind::IntLiteral   => ServValue::Int(token.to_string().parse::<i64>().unwrap().into()),
        TokenKind::TemplateOpen => ServValue::Func(ServFn::Template(parse_template(parser)?.into())),
        TokenKind::ModuleOpen   => {
            parser.incr();
            ServValue::Module(parse_module(parser, ctx)?)
        },
        TokenKind::At => {
            parser.incr();
            let func = parser.get(0).unwrap();
            if func.kind != TokenKind::Identifier {
                return Err("invalid parser function".into());
            }

			match func.to_string().as_str() {
    			"const" => { parser.incr(); constant(parser, ctx) },
    			_ => Err("invalid parser func".into()),
			}?

         //    let mut expr = ServList::new();
         //    while let Ok(word) = parse_word(parser, ctx) {
         //        expr.push_back(word);
         //    }

        	// let result = engine::eval(expr, ctx)?;
        	// return Ok(result)

        	// todo!();
        },

        other => return Err("unhandled token".into()),
    };

    parser.incr();
    Ok(output)
}

fn parse_expression(parser: &mut Parser, ctx: &mut Stack) -> Result<ServList, ServError> {
    let mut output = ServList::new();
    while let Ok(word) = parse_word(parser, ctx) {
        output.push_back(word);
    }

    while let Ok(_) = parser.next_if_kind(ModuleSeparator) {}

    Ok(output)
}

fn get_label(mut input: ServList) -> Result<Address, ServError> {
    if input.len() == 0 { return Err(ServError::new(500, "missing label before declaration")) };
    if input.len() >= 2 { return Err(ServError::new(500, "labels must be exactly 1 word")) };

	match input.pop().unwrap() {
    	ServValue::Ref(label) => Ok(label),
    	otherwise => Err(ServError::new(500, "definition did not start with a valid label")),
	}
}

pub fn parse_declaration(parser: &mut Parser, ctx: &mut Stack) -> Result<(Option<Address>, ServList), ServError> {

    // ignore multiple line breaks in a row
    while let Ok(_) = parser.next_if_kind(ModuleSeparator) {}

    let lhs = parse_expression(parser, ctx)?;
    let Ok(_) = parser.next_if_kind(Equals) else {
        return Ok((None, lhs.into()))
    };

    let label = get_label(lhs)?;
    let rhs = parse_expression(parser, ctx)?;
    Ok((Some(label), rhs.into()))
}


fn include(parser: &mut Parser, ctx: &mut Stack) -> Result<(), ServError> {
    todo!();
}

pub fn parse_module(parser: &mut Parser, ctx: &mut Stack) -> Result<ServModule, ServError> {
    let mut output = ServModule::default();

	while parser.get(0).is_ok() {
    	if parser.get(0)?.kind == ModuleClose { break };
    	if parser.get(0)?.kind == Comment { continue };

    	// include statements need to be executed at parsetime
    	if parser.get(0)?.kind == Include {
        	parser.incr();
        	let (label, mut expr) = parse_declaration(parser, ctx)?;
        	if label.is_some() {
            	return Err(ServError::Empty)
            	// return Err(ServError::new(500, ""))
        	};

        	let result = expr.eval(ctx)?;
        	let ServValue::Module(m) = result else {
            	return Err(ServError::new(500, "include statements require a module type"));
        	};

        	output.values.extend(m.values);
    	}

    	let (label, value) = parse_declaration(parser, ctx)?;
    	output.insert_declaration(label, value);
	}

	Ok(output)
}
