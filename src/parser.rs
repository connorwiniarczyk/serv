use crate::lexer;
use crate::lexer::*;
use crate::lexer::TokenKind::*;
use crate::ast;
use crate::ast::{TemplateElement, Template};

struct Cursor<'tokens> {
    input: &'tokens [Token],
    index: usize,
}

impl<'tokens> Cursor<'tokens> {
    fn new(input: &'tokens [Token]) -> Self {
        Self { input, index: 0 }
    }

    fn incr(&mut self, i: usize) {
        self.index += i;
    }

    fn get(&mut self, offset: usize) -> Result<&'tokens Token, &'static str> {
        if self.index + offset < self.input.len() {
            Ok(&self.input[self.index + offset])
        } else {
            Err("out of bounds")
        }
    }

    fn expect(&mut self, kind: TokenKind) -> Result<&'tokens Token, &'static str> {
        let current = self.get(0)?;
        if current.kind == kind {
            Ok(current)
        } else {
            Err("failed assertion")
        }
    }
}

fn parse_template(cursor: &mut Cursor) -> Result<ast::Template, &'static str>  {
    let open = cursor.expect(TokenKind::TemplateOpen)?.clone();
    cursor.incr(1);
    
	let mut elements: Vec<ast::TemplateElement> = Vec::new();
	while cursor.get(0)?.kind != TokenKind::TemplateClose {
    	let token = cursor.get(0)?;
    	match token.kind {
        	TokenKind::TemplateText => {
            	cursor.incr(1);
            	elements.push(ast::TemplateElement::Text(token.clone()))
        	},
        	TokenKind::Dollar => elements.push({
            	cursor.incr(1);
            	ast::TemplateElement::Expression(parse_word(cursor)?)
        	}),

        	TokenKind::TemplateOpen => {
            	cursor.incr(1);
            	elements.push(ast::TemplateElement::Template(parse_template(cursor)?))
        	},
        	TokenKind::TemplateClose => break,
        	// TokenKind::LambdaBegin => elements.push(ast::TemplateElement::)
        	_ => return Err("token not supported in template"),
    	}
	}

	let close = cursor.expect(TokenKind::TemplateClose)?.clone();
	Ok(ast::Template { open, close, elements })
}

fn parse_word(cursor: &mut Cursor) -> Result<ast::Word, &'static str> {
    let token = cursor.get(0)?;
    let output = match token.kind {
        TokenKind::Identifier => ast::Word::Function(token.clone()),
        TokenKind::IntLiteral => ast::Word::Literal(token.contents.parse::<i64>().unwrap().into()),
        TokenKind::TemplateOpen => ast::Word::Literal(parse_template(cursor)?.into()),
        TokenKind::LambdaBegin => {
            cursor.incr(1);
            ast::Word::Parantheses(parse_expression(cursor)?)
        },

        k @ _ => return Err("unhandled token"),
    };

    cursor.incr(1);
    Ok(output)
}


fn parse_expression(cursor: &mut Cursor) -> Result<ast::Expression, &'static str> {
    let mut output: Vec<ast::Word> = Vec::new();
    while let Ok(word) = parse_word(cursor) {
        output.push(word);
    }
   
    Ok(ast::Expression(output))
}


fn parse_root(cursor: &mut Cursor) -> Result<ast::AstRoot, &'static str> {
	let mut output: Vec<ast::Declaration> = Vec::new();
	
	while cursor.get(0)?.kind != EndOfInput {
    	let token = cursor.get(0).unwrap();
    	let (kind, pattern) = match cursor.get(0).unwrap().kind {
        	Route => ("route", token.contents.to_owned()),
        	At    => ("word",  {cursor.incr(1); cursor.expect(Identifier).unwrap().contents.to_owned()} ),
        	_     =>  panic!(),
    	};

    	cursor.incr(1);
    	_ = cursor.expect(WideArrow).unwrap();

    	cursor.incr(1);
    	let value = parse_expression(cursor)?;
    	// let value = parse_expression(cursor).map(Box::new)?;
    	output.push(ast::Declaration {kind: kind.to_owned(), key: pattern.to_owned(), value});
	}

	Ok(ast::AstRoot(output))
}

pub fn parse_expression_from_text(input: &str) -> Result<ast::Expression, &'static str> {
	let mut tokens = lexer::tokenize(input);
	let mut cursor = Cursor::new(&tokens);
	parse_expression(&mut cursor)
}

pub fn parse_root_from_text(input: &str) -> Result<ast::AstRoot, &'static str> {
	let tokens = lexer::tokenize(input);
	// println!("{:#?}", tokens);
	let mut cursor = Cursor::new(&tokens);
	let ast = parse_root(&mut cursor)?;

	// println!("{:#?}", ast);

	Ok(ast)
}
