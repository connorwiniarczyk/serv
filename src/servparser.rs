use crate::template::{Template, TemplateElement};
use crate::{ ServValue, ServFn, Label };
use crate::datatypes::module::{ ServModule, Element };
use crate::datatypes::module;
use crate::datatypes::value;
use crate::datatypes::value::ServList;

use crate::servlexer::{TokenKind, TokenKind::*};
use crate::servlexer;

use parsetool::cursor::Token;
use parsetool::ParseError;

use crate::error::ServError;

type Parser<'a> = parsetool::parser::Parser<'a, TokenKind>;

fn parse_template(parser: &mut Parser) -> Result<Template, ServError> {
    let open = parser.expect(TokenKind::TemplateOpen)?.to_string();
    parser.incr()?;

	let mut elements: Vec<TemplateElement> = Vec::new();
	while parser.current()?.kind != TokenKind::TemplateClose {
    	let token = parser.current()?.clone();
    	match token.kind {
        	TokenKind::TemplateText => {
            	elements.push(TemplateElement::Text(token.to_string()));
            	parser.incr();
        	},
        	TokenKind::Dollar => elements.push({
            	parser.incr();
            	TemplateElement::Expression(parse_word(parser, &ServModule::empty())?)
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

fn parse_word(parser: &mut Parser, m: &ServModule) -> Result<ServValue, ServError> {
    let token = parser.get(0)?;
    let output = match token.kind {
        TokenKind::Identifier   => ServValue::Ref(Label::Name(token.to_string())),
        TokenKind::IntLiteral   => ServValue::Int(token.to_string().parse::<i64>().unwrap().into()),
        TokenKind::Route        => ServValue::Ref(Label::Route(token.to_string())),
        TokenKind::TemplateOpen => ServValue::Func(ServFn::Template(parse_template(parser)?.into())),
        TokenKind::ModuleOpen   => {
            parser.incr();
            ServValue::Module(parse_module(parser)?)
        },
        TokenKind::At => {
            parser.incr();

            let mut expr = ServList::new();
            while let Ok(word) = parse_word(parser, m) {
                expr.push_back(word);
            }

            // let mut expr = parse_expression(parser, m)?;
            let mut scope = Stack::empty();

        	scope.insert_module(crate::functions::standard_library().values);
        	scope.insert_module(m.values.clone());
        	let result = expr.eval(&mut scope)?;

        	return Ok(result)

        	// println!("{}", result);

			// result
        },
        // TokenKind::OpenParenthesis => panic!("unexpected open paren"),
        // TokenKind::OpenParenthesis => {
        //     parser.incr();
        //     ServValue::Func(ServFn::Expr(parse_expression(parser)?, false))
        // },

        k => return Err("unhandled token".into()),
    };

    parser.incr();
    Ok(output)
}

fn parse_expression(parser: &mut Parser, m: &ServModule) -> Result<ServList, ServError> {
    let mut output = ServList::new();
    while let Ok(word) = parse_word(parser, m) {
        output.push_back(word);
    }

    while let Ok(_) = parser.next_if_kind(ModuleSeparator) {}

    Ok(output)
}

fn get_label(mut input: ServList) -> Result<Label, ServError> {
    if input.len() == 0 { return Err(ServError::new(500, "missing label before declaration")) };
    if input.len() >= 2 { return Err(ServError::new(500, "labels must be exactly 1 word")) };

	match input.pop().unwrap() {
    	ServValue::Ref(label) => Ok(label),
    	otherwise => Err(ServError::new(500, "definition did not start with a valid label")),
	}

}

fn parse_declaration(parser: &mut Parser, m: &ServModule) -> Result<(Option<Label>, ServList), ServError> {

    // ignore multiple line breaks in a row
    while let Ok(_) = parser.next_if_kind(ModuleSeparator) {}

    let lhs = parse_expression(parser, &ServModule::empty())?;
    let Ok(_) = parser.next_if_kind(Equals) else {
        return Ok((None, lhs.into()))
    };

    let label = get_label(lhs)?;
    let rhs = parse_expression(parser, m)?;
    Ok((Some(label), rhs.into()))
}

use crate::Stack;

fn parse_module(parser: &mut Parser) -> Result<ServModule, ServError> {
    let mut output = ServModule::default();

	while parser.get(0).is_ok() {
    	if parser.get(0)?.kind == ModuleClose { break };
    	if parser.get(0)?.kind == Comment { continue };

    	// include statements need to be executed at parsetime
    	if parser.get(0)?.kind == Include {
        	parser.incr();
        	let (label, mut expr) = parse_declaration(parser, &output)?;
        	if label.is_some() { return Err(ServError::new(500, "")) };

        	let mut scope = Stack::empty();
        	scope.insert_module(crate::functions::standard_library().values);
        	scope.insert_module(output.values.clone());

        	let result = expr.eval(&mut scope)?;
        	let ServValue::Module(m) = result else {
            	return Err(ServError::new(500, "include statements require a module type"));
        	};

        	output.values.extend(m.values);
    	}

    	let (label, value) = parse_declaration(parser, &output)?;
    	output.insert_declaration(label, value);
	}

	Ok(output)
}

pub fn parse_template_from_text(input: &str, brackets: bool) -> Result<Template, ServError> {
    todo!();
    // let chars: Vec<char> = input.chars().collect();
    // let tokens = servlexer::tokenize_template(&chars);
    // let mut parser = Parser::new(&tokens);
    // let ast = parse_template(&mut parser)?;

    // Ok(ast)
}

pub fn parse_expression_from_text(input: &str) -> Result<ServValue, ServError> {
    todo!();
    // let chars: Vec<char> = input.chars().collect();
    // let tokens = servlexer::tokenize_serv(&chars);
    // let mut parser = Parser::new(&tokens);
    // let ast = parse_expression(&mut parser)?;

    // Ok(ast)
}

pub fn parse_root_from_text(input: &str) -> Result<ServModule, ServError> {
    let chars: Vec<char> = input.chars().collect();
    let tokens = servlexer::tokenize(&chars);
    let mut parser = Parser::new(&tokens);
    let ast = parse_module(&mut parser)?;

    Ok(ast)
}
