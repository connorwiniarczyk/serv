use crate::ast;
use crate::Template;

use crate::servlexer::{TokenKind, TokenKind::*};
use crate::servlexer;
use parsetool::cursor::Token;
use parsetool::parser::ServError;

type Parser<'a> = parsetool::parser::Parser<'a, TokenKind>;

fn parse_template(parser: &mut Parser) -> Result<Template, ServError> {

    let open = parser.expect(TokenKind::TemplateOpen)?.to_string();
    parser.incr()?;

	let mut elements: Vec<ast::TemplateElement> = Vec::new();
	while parser.current()?.kind != TokenKind::TemplateClose {
    	let token = parser.current()?.clone();
    	match token.kind {
        	TokenKind::TemplateText => {
            	elements.push(ast::TemplateElement::Text(token.to_string()));
            	parser.incr();
        	},
        	TokenKind::Dollar => elements.push({
            	parser.incr();
            	ast::TemplateElement::Expression(parse_word(parser)?)
        	}),

        	TokenKind::TemplateOpen => {
            	elements.push(ast::TemplateElement::Template(parse_template(parser)?));
            	parser.incr();
        	},
        	TokenKind::TemplateClose => { unreachable!(); },
        	_ => return Err("token not supported in template".into()),
        	// a => panic!("token {:?} not supported in template", a),
    	}
	}

	let close = parser.expect(TokenKind::TemplateClose).unwrap().to_string();
	Ok(ast::Template { open, close, elements })
}

fn parse_word(parser: &mut Parser) -> Result<ast::Word, ServError> {
    let token = parser.get(0)?;
    let output = match token.kind {
        TokenKind::Identifier   => ast::Word::Function(token.to_string()),
        TokenKind::ListEnd      => ast::Word::Function(token.to_string()),
        TokenKind::IntLiteral   => ast::Word::Literal(token.to_string().parse::<i64>().unwrap().into()),
        TokenKind::TemplateOpen => ast::Word::Template(parse_template(parser)?.into()),
        TokenKind::OpenParenthesis => {
            parser.incr();
            ast::Word::Parantheses(parse_expression(parser)?)
        },

        k @ _ => return Err("unhandled token".into()),
    };

    parser.incr();
    Ok(output)
}

fn parse_expression(parser: &mut Parser) -> Result<ast::Expression, ServError> {
    let mut output: Vec<ast::Word> = Vec::new();
    while let Ok(word) = parse_word(parser) {
        output.push(word);
    }

    fn is_meta(words: &Vec<ast::Word>) -> bool {
        match words.last() {
            Some(ast::Word::Function(t)) if t == "]" => true,
            Some(ast::Word::Parantheses(ast::Expression(e, is_meta))) => *is_meta,
            otherwise => false,
        }
    }

    let meta = is_meta(&output);

    while let Ok(_) = parser.next_if_kind(Semicolon) {}
    
    Ok(ast::Expression(output, meta))
}

fn parse_declaration(parser: &mut Parser) -> Result<ast::Declaration, ServError> {
    while let Ok(_) = parser.next_if_kind(Semicolon) {}

    if let Ok(_) = parser.next_if_kind(At) {};

	let next = parser.get(0)?;
	match next.kind {
    	Route => {
        	let route = parser.next_if_kind(Route).unwrap();
            _ = parser.next_if_kind(Equals)?;
            Ok(ast::Declaration {
                kind: "route".to_string(),
                key:   route.to_string(),
                value: parse_expression(parser)?,
            })
    	},

    	Identifier if next.value == "include" => {
        	let _ = parser.next_if_kind(Identifier)?;
            Ok(ast::Declaration {
                kind: "include".to_string(),
                key:  "include".to_string(),
                value: parse_expression(parser)?,
            })
    	},

    	Identifier => {
        	let ident = parser.next_if_kind(Identifier).unwrap();
            _ = parser.next_if_kind(Equals)?;
            Ok(ast::Declaration {
                kind: "word".to_string(),
                key:   ident.to_string(),
                value: parse_expression(parser)?,
            })
    	},

    	otherwise => panic!("unexpected {:?}", next),
	}



    // if let Ok(route) = parser.next_if_kind(Route) {
    //     _ = parser.next_if_kind(Equals)?;
    //     Ok(ast::Declaration {
    //         kind: "route".to_string(),
    //         key:   route.to_string(),
    //         value: parse_expression(parser)?,
    //     })
    // }

    // else if let Ok(word) = parser.next_if_kind(Identifier) {
    //     _ = parser.next_if_kind(Equals)?;
    //     Ok(ast::Declaration {
    //         kind: "word".to_string(),
    //         key:   word.to_string(),
    //         value: parse_expression(parser)?,
    //     })
    // }

    // else {
    //     Err(format!("token {:?} is not a valid way to start a route", parser.get(0)).as_str().into())
    // }
}

fn parse_root(parser: &mut Parser) -> Result<ast::AstRoot, ServError> {
	let mut output: Vec<ast::Declaration> = Vec::new();

	while parser.get(0).is_ok() {
    	if parser.get(0)?.kind == Comment { continue };
    	output.push(parse_declaration(parser)?);
	}

	Ok(ast::AstRoot(output))
}

pub fn parse_template_from_text(input: &str, brackets: bool) -> Result<ast::Template, ServError> {
    let chars: Vec<char> = input.chars().collect();
    let tokens = servlexer::tokenize_template(&chars);
    let mut parser = Parser::new(&tokens);
    let ast = parse_template(&mut parser)?;

    Ok(ast)
}

pub fn parse_expression_from_text(input: &str) -> Result<ast::Expression, ServError> {
    let chars: Vec<char> = input.chars().collect();
    let tokens = servlexer::tokenize_serv(&chars);
    let mut parser = Parser::new(&tokens);
    let ast = parse_expression(&mut parser)?;

    Ok(ast)
}

pub fn parse_root_from_text(input: &str) -> Result<ast::AstRoot, ServError> {
    let chars: Vec<char> = input.chars().collect();
    let tokens = servlexer::tokenize_serv(&chars);
    let mut parser = Parser::new(&tokens);
    let ast = parse_root(&mut parser)?;

    Ok(ast)
}
