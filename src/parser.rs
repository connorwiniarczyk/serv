// use crate::lexer;
// use crate::lexer::*;
// use crate::lexer::TokenKind::*;
use crate::ast;
use crate::ast::{TemplateElement, Template};
use crate::error::ServError;

use crate::serv_tokenizer::{ServToken as Token, TokenKind};
use crate::serv_tokenizer::TokenKind::*;

use std::iter::Peekable;

struct Cursor<'input, I: Iterator<Item = Token<'input>>>(Peekable<I>);

impl<'input, I> Cursor<'input, I> where I: Iterator<Item = Token<'input>> {
    fn new(input: I) -> Self {
		Self(input.peekable())
    }

    fn advance(&mut self) {
        _ = self.0.next();
    }

    fn get_kind(&mut self) -> TokenKind {
        let t = self.0.peek().unwrap();
        let output = t.clone();
        output.kind
        // self.0.peek().ok_or("out of bounds".into()).copied()
    }

    fn current(&mut self) -> Result<Token, ServError> {
        self.0.peek().ok_or("out of bounds".into()).copied()
    }

    fn take(&mut self, kind: TokenKind) -> Result<Token, ServError> {
        if self.0.peek().unwrap().kind == kind {
        // if self.current()?.kind == kind {
            Ok(self.0.next().unwrap())
        } else {
            Err("incorrect kind".into())
        }
    }

    fn expect(&mut self, kind: TokenKind) -> Result<Token, ServError> {
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
        let token = self.current()?;
        let output = match token.kind {
            TokenKind::Identifier => ast::Word::Function(token.to_string()),
            TokenKind::IntLiteral => ast::Word::Literal(token.to_string().parse::<i64>().unwrap().into()),
            TokenKind::TemplateOpen => ast::Word::Template(self.parse_template()?.into()),
            TokenKind::ListBegin => {
                self.advance();
                todo!();
                // ast::Word::List(self.parse_list()?)
            },
            TokenKind::OpenParenthesis=> {
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
       
        Ok(ast::Expression(output))
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
        if let Ok(r) = self.take(Route) {
            let route = r.clone();
            _ = self.take(WideArrow)?;
            let value = self.parse_expression()?;
            Ok(ast::Declaration {
                kind: "route".to_string(),
                key:   route.to_string(),
                value: value,
            })
        }

        else if self.take(At).is_ok() {
            let word = self.take(Identifier)?;
            _ = self.take(WideArrow)?;
            let value = self.parse_expression()?;
            Ok(ast::Declaration {
                kind: "word".to_string(),
                key:   word.to_string(),
                value: value,
            })
        }

        else {
            Err(format!("token {:?} is not a valid way to start a route", self.current()?).as_str().into())
        }
    }

    fn parse_root(&mut self) -> Result<ast::AstRoot, ServError> {
    	let mut output: Vec<ast::Declaration> = Vec::new();

    	while self.current().is_ok() {
        	output.push(self.parse_declaration()?);
    	}
    	
    	// while cursor.current()?.kind != EndOfInput {
     //    	output.push(parse_declaration(cursor)?)
    	// }

    	Ok(ast::AstRoot(output))
    }
}

// struct Cursor<'input> {
//     input: &'input [Token<'input>],
//     // input: Vec<Token<'input>>,
//     index: usize,
// }

// impl<'input> Cursor<'input> {
//     // fn new(input: Vec<Token<'input>>) -> Self {
//     fn new(input: &'input[Token<'input>]) -> Self {
//         Self { input, index: 0 }
//     }

//     fn advance(&mut self) {
//         self.index += 1;
//     }

//     fn current(&self) -> Result<Token, ServError> {
//         if self.index >= self.input.len() { return Err("out of bounds".into()) };
//         Ok(self.input[self.index])
//     }

//     fn take(& mut self, kind: TokenKind) -> Result<Token, ServError> {
//         if self.index >= self.input.len() { return Err("out of bounds".into()) };
// 		if self.current()?.kind == kind {
//     		let output = self.input[self.index];
//     		self.index += 1;
//     		Ok(output)
// 		} else {
//     		Err("incorrect kind".into())
// 		}
//     }

//     fn expect(&mut self, kind: TokenKind) -> Result<&Token, ServError> {
//         todo!();
//         // let current = self.current()?;
//         // if current.kind == kind {
//         //     Ok(current)
//         // } else {
//         //     Err("failed assertion".into())
//         // }
//     }
// }



pub fn parse_template_from_text(input: &str) -> Result<ast::Template, ServError> {
    todo!();
	// let mut tokens = lexer::tokenize(input);
	// let mut cursor = Cursor::new(&tokens.0);

	// parse_template(&mut cursor)
}

pub fn parse_expression_from_text(input: &str) -> Result<ast::Expression, ServError> {
    todo!();
	// let mut tokens = lexer::tokenize(input);
	// let mut cursor = Cursor::new(&tokens.0);
	// parse_expression(&mut cursor)
}

pub fn parse_root_from_text(input: &str) -> Result<ast::AstRoot, ServError> {
    let chars: Vec<char> = input.chars().collect();
    let tokens = crate::serv_tokenizer::tokenize_serv(&chars);
    for t in tokens.iter() {
        println!("{}", t);
    }
    let mut cursor = Cursor::new(tokens.into_iter());

    let ast = cursor.parse_template();
    println!("{:#?}", ast);


	todo!();

	// let tokens = lexer::tokenize(input);
	// let mut cursor = Cursor::new(&tokens.0);
	// let ast = parse_root(&mut cursor)?;

	// Ok(ast)
}
