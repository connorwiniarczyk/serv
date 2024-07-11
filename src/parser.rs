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

    fn incr(&mut self, i: usize) {
        self.index += i;
    }

    fn get(&mut self, offset: usize) -> Result<&'tokens Token, ServError> {
        if self.index + offset < self.input.len() {
            Ok(&self.input[self.index + offset])
        } else {
            Err("out of bounds".into())
        }
    }

    fn expect(&mut self, kind: TokenKind) -> Result<&'tokens Token, ServError> {
        let current = self.get(0)?;
        if current.kind == kind {
            Ok(current)
        } else {
            Err("failed assertion".into())
        }
    }
}

fn parse_template(cursor: &mut Cursor) -> Result<ast::Template, ServError>  {
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
            	elements.push(ast::TemplateElement::Template(parse_template(cursor)?));
            	cursor.incr(1);
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
    while cursor.get(0)?.kind != TokenKind::ListEnd {
        output.push(parse_expression(cursor)?);
        if cursor.get(0)?.kind == TokenKind::Comma {
            cursor.incr(1);
        }
    }
    Ok(output)
}

fn parse_word(cursor: &mut Cursor) -> Result<ast::Word, ServError> {
    let token = cursor.get(0)?;
    let output = match token.kind {
        TokenKind::Identifier => ast::Word::Function(token.clone()),
        TokenKind::IntLiteral => ast::Word::Literal(token.contents.parse::<i64>().unwrap().into()),
        TokenKind::TemplateOpen => ast::Word::Template(parse_template(cursor)?.into()),
        TokenKind::ListBegin => {
            cursor.incr(1);
            ast::Word::List(parse_list(cursor)?)
        },
        TokenKind::LambdaBegin => {
            cursor.incr(1);
            ast::Word::Parantheses(parse_expression(cursor)?)
        },

        k @ _ => return Err("unhandled token".into()),
    };

    cursor.incr(1);
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
    if matches!(cursor.get(0).unwrap().kind, Route) {
		println!("test");
    }

    else if matches!(cursor.get(0).unwrap().kind, At) {
		println!("test");
    }
    todo!();
	// let (kind, pattern) = match cursor.get(0).unwrap().kind {
 //    	Route => ("route", token.contents.to_owned()),
 //    	At    => ("word",  {cursor.incr(1); cursor.expect(Identifier).unwrap().contents.to_owned()} ),
 //    	_ => panic!(),
 //    	// _     =>  return Err(format!("unexpected token {:?}", token).into()),
	// };
}

fn parse_root(cursor: &mut Cursor) -> Result<ast::AstRoot, ServError> {
	let mut output: Vec<ast::Declaration> = Vec::new();
	
	while cursor.get(0)?.kind != EndOfInput {
    	parse_declaration(cursor)?;
    	let token = cursor.get(0).unwrap();
    	let (kind, pattern) = match cursor.get(0).unwrap().kind {
        	Route => ("route", token.contents.to_owned()),
        	At    => ("word",  {cursor.incr(1); cursor.expect(Identifier).unwrap().contents.to_owned()} ),
        	_     =>  panic!("unexpected token {:?}", token),
    	};

    	cursor.incr(1);
    	_ = cursor.expect(WideArrow).unwrap();

    	cursor.incr(1);
    	let value = parse_expression(cursor)?;
    	output.push(ast::Declaration {kind: kind.to_owned(), key: pattern.to_owned(), value});
	}

	Ok(ast::AstRoot(output))
}

pub fn parse_expression_from_text(input: &str) -> Result<ast::Expression, ServError> {
	let mut tokens = lexer::tokenize(input);
	let mut cursor = Cursor::new(&tokens);
	parse_expression(&mut cursor)
}

pub fn parse_root_from_text(input: &str) -> Result<ast::AstRoot, ServError> {
	let tokens = lexer::tokenize(input);
	let mut cursor = Cursor::new(&tokens);
	let ast = parse_root(&mut cursor)?;

	Ok(ast)
}
