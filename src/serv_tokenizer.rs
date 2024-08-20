use crate::cursor::{Tokenizer, Token};

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum TokenKind {
    Identifier,
    IntLiteral,
    Route,
    TemplateOpen,
    TemplateClose,
    TemplateText,
    TemplateVariable,

    Comment,

    ListBegin,
    ListEnd,
    Comma,

	OpenParenthesis,
	CloseParenthesis,

    WhiteSpace,
    At,
    WideArrow,
    Semicolon,
    Dollar,
    Percent,
    NewLine,
    Equals,
}

// #[derive(Debug, Clone, Copy)]
// pub struct ServToken<'input> {
//     pub kind: TokenKind,
//     token: Token<'input>
// }

// impl ServToken<'_> {
//     pub fn to_string(&self) -> String {
//         let mut output = String::new();
//         for c in self.token.value {
//             output.push(*c);
//         }
//         output
//     }
// }

// impl std::fmt::Display for ServToken<'_> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
//         write!(f, "({}-{}) {:?} {:?}", self.token.start, self.token.end, self.kind, self.token.value)
//     }
// }

struct Cursor<'input> {
    input: Tokenizer<'input>,
    output: Vec<Token<TokenKind>>,
}

impl<'i> Cursor<'i> {
    fn new(input: &'i [char]) -> Self {
        Self {
            input: Tokenizer::new(input),
            output: Vec::new(),
        }
    }

    fn push_token(&mut self, kind: TokenKind) {
        self.output.push(self.input.emit(kind));
    }

    fn skip_token(&mut self) {
        _ = self.input.emit(());
    }

    fn tokenize_root(&mut self) {
        while let Some(c) = self.input.get(0) {
            match c {
                '/' => {
                    self.input.incr_while(|x| !x.is_whitespace());
                    self.push_token(TokenKind::Route);
                },

                '@'  => {
                    self.input.incr(1);
                    self.push_token(TokenKind::At)
                },

                '#'  => {
                    self.input.incr(1);
                    self.input.incr_while(|x| x != '\n' && x != '#');
                    self.skip_token();
                },

                '=' => {
                    let i = if self.input.get(1) == Some('>') {2} else {1};
                    self.input.incr(i);
                    self.push_token(TokenKind::WideArrow);
                }

                _ => self.tokenize_expression(),
            }
        }
    }

    fn tokenize_expression(&mut self) {
        while let Some(c) = self.input.get(0) {
            match c {
                ','  => {self.input.incr(1); self.push_token(TokenKind::Comma)},
                ';'  => {self.input.incr(1); self.push_token(TokenKind::Semicolon)},
                '$'  => {self.input.incr(1); self.push_token(TokenKind::Dollar)},
                '%'  => {self.input.incr(1); self.push_token(TokenKind::Identifier)},
                '.'  => {self.input.incr(1); self.push_token(TokenKind::Identifier)},
                '*'  => {self.input.incr(1); self.push_token(TokenKind::Identifier)},
                '!'  => {self.input.incr(1); self.push_token(TokenKind::Identifier)},
                
                '[' => {self.input.incr(1); self.push_token(TokenKind::ListBegin)},
                ']' => {self.input.incr(1); self.push_token(TokenKind::ListEnd)},
                '(' => {
                    self.input.incr(1);
                    self.push_token(TokenKind::OpenParenthesis);
                    self.tokenize_expression();
                },

                ')' => {
                    self.input.incr(1);
                    self.push_token(TokenKind::CloseParenthesis);
                    return;
                },
              
                '\n' => {self.input.incr(1); self.skip_token()},
                '\t' | ' ' => {
                    self.input.incr_while(|x| x == '\t' || x == ' ');
                    self.skip_token();
                },
            
                '{' | '"' => self.tokenize_template(),
                
                c @ _ if c.is_alphabetic() => {
                    self.input.incr_while( |x| x.is_alphanumeric() || x == '_' || x == '.');
                    self.push_token(TokenKind::Identifier);
                },
                
                c @ _ if c.is_digit(10) => {
                    self.input.incr_while(|x| x.is_digit(10));
                    self.push_token(TokenKind::IntLiteral);
                },

                _ => return,
            }
    	}
    }

    fn tokenize_template(&mut self) {
        assert_eq!(self.input.get(0), Some('{'));
        self.input.incr(1);
        self.push_token(TokenKind::TemplateOpen);

        let special_characters = ['{', '\\', '$', '}'];

        while self.input.get(0).expect("reached end of input while parsing string") != '}' {
            let c = self.input.get(0).unwrap();
            match c {
                '{'  => self.tokenize_template(), 
                '$'  => self.tokenize_dollar(),
                '\\' => self.tokenize_escape_sequence(),

                _  => {
                    self.input.incr_while(|x| !special_characters.contains(&x));
                    self.push_token(TokenKind::TemplateText);
                }
            }
        }

        self.input.incr(1);
        self.push_token(TokenKind::TemplateClose);
    }


    fn tokenize_dollar(&mut self) {
        assert_eq!(self.input.get(0), Some('$'));

		// treat '$$' as escaped '$'
        if self.input.get(1) == Some('$') {
            self.input.incr(1);
            self.skip_token();
            self.input.incr(1);
            self.push_token(TokenKind::TemplateText);
            return;
        }

        self.input.incr(1);
        self.push_token(TokenKind::Dollar);

		// if we see a parentheses, tokenize a whole expression
        if self.input.get(0) == Some('(') {
            self.input.incr(1);
            self.push_token(TokenKind::OpenParenthesis);
            self.tokenize_expression();
        } else {
			self.input.incr_while(|x| x.is_alphanumeric() || x == '_' || x == '.');
			self.push_token(TokenKind::Identifier);
        }
    }

    fn tokenize_escape_sequence(&mut self) {todo!("tell connor to implement escape sequences")}
}

pub fn tokenize_serv<'input>(input: &'input [char]) -> Vec<Token<TokenKind>> {
    let mut cursor = Cursor::new(input);
    cursor.tokenize_root();
    std::mem::take(&mut cursor.output)
}
