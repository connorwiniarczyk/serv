use crate::ast;
use crate::ast::{TemplateElement, Template};
use crate::error::ServError;

// use crate::serv_tokenizer::{ServToken as Token, TokenKind};
use crate::serv_tokenizer::TokenKind;
use crate::serv_tokenizer::TokenKind::*;
use crate::cursor::Token;

type ServToken = crate::cursor::Token<TokenKind>;

use std::iter::Peekable;

struct Parser<I: Iterator<Item = ServToken>>(Peekable<I>);

impl<I> Parser<I> where I: Iterator<Item = ServToken> {
    fn new(input: I) -> Self {
		Self(input.peekable())
    }

    fn advance(&mut self) {
        _ = self.0.next();
    }

//     fn get_kind(&mut self) -> TokenKind {
//         let t = self.0.peek().unwrap();
//         let output = t.clone();
//         output.kind
//         // self.0.peek().ok_or("out of bounds".into()).copied()
//     }

    fn current(&mut self) -> Result<ServToken, ServError> {
        self.0.peek().ok_or("out of bounds".into()).cloned()
    }

    fn next_if_kind(&mut self, kind: TokenKind) -> Result<ServToken, ServError> {
        self.0.next_if(|t| t.kind == kind).ok_or("unexpected token".into())
        // if self.0.peek().unwrap().kind == kind {
        // // if self.current()?.kind == kind {
        //     Ok(self.0.next().unwrap())
        // } else {
        //     Err("incorrect kind".into())
        // }
    }

    fn expect(&mut self, kind: TokenKind) -> Result<ServToken, ServError> {
        if self.current()?.kind == kind {
            Ok(self.current()?)
        } else {
            Err("failed assertion!".into())
        }
    }

    // -------------
    // PARSING LOGIC
    // -------------
    fn parse_template(&mut self) -> Result<ast::Template, ServError>  {
        let open = self.expect(TokenKind::TemplateOpen)?.to_string();
        self.advance();
        
    	let mut elements: Vec<ast::TemplateElement> = Vec::new();
    	while self.current()?.kind != TokenKind::TemplateClose {
        	let token = self.current()?.clone();
        	match token.kind {
            	TokenKind::TemplateText => {
                	elements.push(ast::TemplateElement::Text(token.to_string()));
                	self.advance();
            	},
            	TokenKind::Dollar => elements.push({
                	self.advance();
                	ast::TemplateElement::Expression(self.parse_word()?)
            	}),

            	TokenKind::TemplateOpen => {
                	elements.push(ast::TemplateElement::Template(self.parse_template()?));
                	self.advance();
            	},
            	TokenKind::TemplateClose => { unreachable!(); },
            	_ => return Err("token not supported in template".into()),
        	}
    	}

    	let close = self.expect(TokenKind::TemplateClose).unwrap().to_string();
    	Ok(ast::Template { open, close, elements })
    }

    fn parse_word(&mut self) -> Result<ast::Word, ServError> {
        let token = self.0.peek().ok_or("End of input while parsing")?;
        let output = match token.kind {
            TokenKind::Identifier   => ast::Word::Function(token.to_string()),
            TokenKind::ListEnd      => ast::Word::Function(token.to_string()),
            TokenKind::IntLiteral   => ast::Word::Literal(token.to_string().parse::<i64>().unwrap().into()),
            TokenKind::TemplateOpen => ast::Word::Template(self.parse_template()?.into()),
            // TokenKind::ListBegin => {
            //     self.advance();
            //     let mut inner = self.parse_expression()?;
            //     let mut t = vec![ast::Word::Function("[".into())];
            //     for w in inner.0 {
            //         t.push(w);
            //     }
            //     inner.0 = t;
            //     inner.1 = true;

            //     ast::Word::Parantheses(inner)
            // },
            TokenKind::OpenParenthesis => {
                self.advance();
                ast::Word::Parantheses(self.parse_expression()?)
            },

            k @ _ => return Err("unhandled token".into()),
        };

        self.advance();
        Ok(output)
    }

    fn parse_expression(&mut self) -> Result<ast::Expression, ServError> {
        let mut output: Vec<ast::Word> = Vec::new();
        while let Ok(word) = self.parse_word() {
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
        Ok(ast::Expression(output, meta))
    }

    fn parse_list(&mut self) -> Result<Vec<ast::Expression>, ServError> {
        let mut output = Vec::new();
        while self.current()?.kind != TokenKind::ListEnd {
            output.push(self.parse_expression()?);
            if self.current()?.kind == TokenKind::Comma {
                self.advance();
            }
        }
        Ok(output)
    }

    fn parse_declaration(&mut self) -> Result<ast::Declaration, ServError> {
        if let Ok(route) = self.next_if_kind(Route) {
            _ = self.next_if_kind(WideArrow)?;
            Ok(ast::Declaration {
                kind: "route".to_string(),
                key:   route.to_string(),
                value: self.parse_expression()?,
            })
        }

        else if self.next_if_kind(At).is_ok() {
            let word = self.next_if_kind(Identifier)?;
            _ = self.next_if_kind(WideArrow)?;
            Ok(ast::Declaration {
                kind: "word".to_string(),
                key:   word.to_string(),
                value: self.parse_expression()?,
            })
        }

        else {
            Err(format!("token {:?} is not a valid way to start a route", self.0.peek()).as_str().into())
        }
    }

    fn parse_root(&mut self) -> Result<ast::AstRoot, ServError> {
    	let mut output: Vec<ast::Declaration> = Vec::new();

    	while self.0.peek().is_some() {
        	output.push(self.parse_declaration()?);
    	}

    	Ok(ast::AstRoot(output))
    }
}

pub fn parse_template_from_text(input: &str) -> Result<ast::Template, ServError> {
    let chars: Vec<char> = input.chars().collect();
    let tokens = crate::serv_tokenizer::tokenize_serv(&chars);
    let mut parser = Parser::new(tokens.into_iter().filter(|t| t.kind != Comment));
    let ast = parser.parse_template()?;

    Ok(ast)
}

pub fn parse_expression_from_text(input: &str) -> Result<ast::Expression, ServError> {
    let chars: Vec<char> = input.chars().collect();
    let tokens = crate::serv_tokenizer::tokenize_serv(&chars);
    let mut parser = Parser::new(tokens.into_iter().filter(|t| t.kind != Comment));
    let ast = parser.parse_expression()?;

    Ok(ast)
}

pub fn parse_root_from_text(input: &str) -> Result<ast::AstRoot, ServError> {
    let chars: Vec<char> = input.chars().collect();
    let tokens = crate::serv_tokenizer::tokenize_serv(&chars);
    let mut parser = Parser::new(tokens.into_iter().filter(|t| t.kind != Comment));
    let ast = parser.parse_root()?;
    // println!("{:#?}", ast);

	Ok(ast)
}
