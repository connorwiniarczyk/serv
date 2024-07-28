use crate::lexer;
use crate::lexer::*;
use crate::lexer::TokenKind::*;
use crate::ast;
use crate::ast::{TemplateElement, Template};
use crate::error::ServError;

struct Cursor<'tokens> {
    input: &'tokens [Token],
    index: usize,
}

impl<'tokens> Cursor<'tokens> {
    fn new(input: &'tokens [Token]) -> Self {
        Self { input, index: 0 }
    }

    fn advance(&mut self) {
        self.index += 1;
    }

    fn current(&mut self) -> Result<&'tokens Token, ServError> {
        if self.index >= self.input.len() { return Err("out of bounds".into()) };

        Ok(&self.input[self.index])
    }

    fn take(&mut self, kind: TokenKind) -> Result<&'tokens Token, ServError> {
		if self.current()?.kind == kind {
    		let output = self.current()?;
    		self.index += 1;
    		Ok(output)
		} else {
    		Err("incorrect kind".into())
		}
    }

    fn expect(&mut self, kind: TokenKind) -> Result<&'tokens Token, ServError> {
        let current = self.current()?;
        if current.kind == kind {
            Ok(current)
        } else {
            Err("failed assertion".into())
        }
    }
}

fn parse_template(cursor: &mut Cursor) -> Result<ast::Template, ServError>  {
    let open = cursor.expect(TokenKind::TemplateOpen)?.clone();
    cursor.advance();
    
	let mut elements: Vec<ast::TemplateElement> = Vec::new();
	while cursor.current()?.kind != TokenKind::TemplateClose {
    	let token = cursor.current()?;
    	match token.kind {
        	TokenKind::TemplateText => {
            	cursor.advance();
            	elements.push(ast::TemplateElement::Text(token.clone()))
        	},
        	TokenKind::Dollar => elements.push({
            	cursor.advance();
            	ast::TemplateElement::Expression(parse_word(cursor)?)
        	}),

        	TokenKind::TemplateOpen => {
            	elements.push(ast::TemplateElement::Template(parse_template(cursor)?));
            	cursor.advance();
        	},
        	TokenKind::TemplateClose => { unreachable!(); },
        	_ => return Err("token not supported in template".into()),
    	}
	}

	let close = cursor.expect(TokenKind::TemplateClose).unwrap().clone();
	Ok(ast::Template { open, close, elements })
}

fn parse_list(cursor: &mut Cursor) -> Result<Vec<ast::Expression>, ServError> {
    let mut output = Vec::new();
    while cursor.current()?.kind != TokenKind::ListEnd {
        output.push(parse_expression(cursor)?);
        if cursor.current()?.kind == TokenKind::Comma {
            cursor.advance();
        }
    }
    Ok(output)
}

fn parse_word(cursor: &mut Cursor) -> Result<ast::Word, ServError> {
    let token = cursor.current()?;
    let output = match token.kind {
        TokenKind::Identifier => ast::Word::Function(token.clone()),
        TokenKind::IntLiteral => ast::Word::Literal(token.contents.parse::<i64>().unwrap().into()),
        TokenKind::TemplateOpen => ast::Word::Template(parse_template(cursor)?.into()),
        TokenKind::ListBegin => {
            cursor.advance();
            ast::Word::List(parse_list(cursor)?)
        },
        TokenKind::LambdaBegin => {
            cursor.advance();
            ast::Word::Parantheses(parse_expression(cursor)?)
        },

        k @ _ => return Err("unhandled token".into()),
    };

    cursor.advance();
    Ok(output)
}


fn parse_expression(cursor: &mut Cursor) -> Result<ast::Expression, ServError> {
    let mut output: Vec<ast::Word> = Vec::new();
    while let Ok(word) = parse_word(cursor) {
        output.push(word);
    }
   
    Ok(ast::Expression(output))
}

fn parse_declaration(cursor: &mut Cursor) -> Result<ast::Declaration, ServError> {
    if let Ok(route) = cursor.take(Route) {
        _ = cursor.take(WideArrow)?;
        let value = parse_expression(cursor)?;
        Ok(ast::Declaration {
            kind: "route".to_string(),
            key:   route.contents.clone(),
            value: value,
        })
    }

    else if cursor.take(At).is_ok() {
        let word = cursor.take(Identifier)?;
        _ = cursor.take(WideArrow)?;
        let value = parse_expression(cursor)?;
        Ok(ast::Declaration {
            kind: "word".to_string(),
            key:   word.contents.clone(),
            value: value,
        })
    }

    else {
        Err(format!("token {} is not a valid way to start a route", cursor.current()?).as_str().into())
    }
}

fn parse_root(cursor: &mut Cursor) -> Result<ast::AstRoot, ServError> {
	let mut output: Vec<ast::Declaration> = Vec::new();
	
	while cursor.current()?.kind != EndOfInput {
    	output.push(parse_declaration(cursor)?)
	}

	Ok(ast::AstRoot(output))
}

pub fn parse_template_from_text(input: &str) -> Result<ast::Template, ServError> {
	let mut tokens = lexer::tokenize(input);
	let mut cursor = Cursor::new(&tokens.0);

	parse_template(&mut cursor)
}

pub fn parse_expression_from_text(input: &str) -> Result<ast::Expression, ServError> {
	let mut tokens = lexer::tokenize(input);
	let mut cursor = Cursor::new(&tokens.0);
	parse_expression(&mut cursor)
}

pub fn parse_root_from_text(input: &str) -> Result<ast::AstRoot, ServError> {
	let tokens = lexer::tokenize(input);
	let mut cursor = Cursor::new(&tokens.0);
	let ast = parse_root(&mut cursor)?;

	Ok(ast)
}
