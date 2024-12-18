use crate::template::{Template, TemplateElement};
use crate::{ ServValue, ServFn, Label };
use crate::module::{ ServModule, Expression };
use crate::module;

use crate::servlexer::{TokenKind, TokenKind::*};
use crate::servlexer;
use parsetool::cursor::Token;
use parsetool::parser::ServError;

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
            	TemplateElement::Expression(parse_word(parser)?)
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

fn parse_word(parser: &mut Parser) -> Result<ServValue, ServError> {
    let token = parser.get(0)?;
    let output = match token.kind {
        TokenKind::Identifier   => ServValue::Ref(Label::Name(token.to_string())),
        TokenKind::ListEnd      => ServValue::Ref(Label::Name(token.to_string())),
        TokenKind::IntLiteral   => ServValue::Int(token.to_string().parse::<i64>().unwrap().into()),
        TokenKind::Route        => ServValue::Func(ServFn::Route(token.to_string())),
        TokenKind::TemplateOpen => ServValue::Func(ServFn::Template(parse_template(parser)?.into())),
        TokenKind::OpenParenthesis => {
            parser.incr();
            ServValue::Func(ServFn::Expr(parse_expression(parser)?, false))
        },
        // TokenKind::Identifier   => ast::Word::Function(token.to_string()),
    //     TokenKind::ListEnd      => ast::Word::Function(token.to_string()),
    //     TokenKind::IntLiteral   => ast::Word::Literal(token.to_string().parse::<i64>().unwrap().into()),
    //     TokenKind::Route        => ast::Word::Literal(ServValue::Func(ServFn::Route(token.to_string()))),
    //     TokenKind::TemplateOpen => ast::Word::Template(parse_template(parser)?.into()),
        // TokenKind::OpenParenthesis => {
        //     parser.incr();
        //     ast::Word::Parantheses(parse_expression(parser)?)
        // },

        k @ _ => return Err("unhandled token".into()),
    };

    parser.incr();
    Ok(output)
}

fn parse_expression(parser: &mut Parser) -> Result<module::Expression, ServError> {
    let mut output: Vec<ServValue> = Vec::new();
    while let Ok(word) = parse_word(parser) {
        output.push(word);
    }

    // fn is_meta(words: &Vec<ast::Word>) -> bool {
    //     match words.last() {
    //         Some(ast::Word::Function(t)) if t == "]" => true,
    //         Some(ast::Word::Parantheses(ast::Expression(e, is_meta))) => *is_meta,
    //         otherwise => false,
    //     }
    // }

    // let meta = is_meta(&output);

    while let Ok(_) = parser.next_if_kind(Semicolon) {}
    
    Ok(module::Expression(output.into(), false))
}

fn get_pattern(mut input: Expression) -> Option<ServValue> {
    if input.0.len() == 0 { return None };
    if input.0.len()  > 1 { return Some(ServValue::Func(ServFn::Expr(input, false))) };

    match input.next().unwrap() {
        route @ ServValue::Func(ServFn::Route(_)) => Some(route),
        label @ ServValue::Ref(_) => Some(label),
        otherwise => Some(ServValue::Func(ServFn::Expr(input, false))),
    }
}

fn parse_declaration(parser: &mut Parser) -> Result<module::Element, ServError> {
    while let Ok(_) = parser.next_if_kind(Semicolon) {}

    if let Ok(_) = parser.next_if_kind(At) {};

    let lhs = parse_expression(parser)?;
    let Ok(_) = parser.next_if_kind(Equals) else {
        return Ok(module::Element { pattern: None, action: lhs })
    };

    let rhs = parse_expression(parser)?;

    Ok(module::Element{ pattern: get_pattern(lhs), action: rhs })
}

fn parse_module(parser: &mut Parser) -> Result<ServModule, ServError> {
	let mut output: Vec<module::Element> = Vec::new();

	while parser.get(0).is_ok() {
    	if parser.get(0)?.kind == Comment { continue };
    	output.push(parse_declaration(parser)?);
	}

	Ok(ServModule::from_elements(output.into_iter()))
}

pub fn parse_template_from_text(input: &str, brackets: bool) -> Result<Template, ServError> {
    todo!();
    // let chars: Vec<char> = input.chars().collect();
    // let tokens = servlexer::tokenize_template(&chars);
    // let mut parser = Parser::new(&tokens);
    // let ast = parse_template(&mut parser)?;

    // Ok(ast)
}

pub fn parse_expression_from_text(input: &str) -> Result<module::Expression, ServError> {
    todo!();
    // let chars: Vec<char> = input.chars().collect();
    // let tokens = servlexer::tokenize_serv(&chars);
    // let mut parser = Parser::new(&tokens);
    // let ast = parse_expression(&mut parser)?;

    // Ok(ast)
}

pub fn parse_root_from_text(input: &str) -> Result<ServModule, ServError> {
    let chars: Vec<char> = input.chars().collect();
    let tokens = servlexer::tokenize_serv(&chars);
    let mut parser = Parser::new(&tokens);
    let ast = parse_module(&mut parser)?;

    Ok(ast)
}
