use parsetool::cursor::{Tokenizer, Token};

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum TokenKind {
    Identifier,
    IntLiteral,
    Route,
    TemplateOpen,
    TemplateClose,
    TemplateText,
    TemplateVariable,

    Semicolon,

    Comment,

	OpenParenthesis,
	CloseParenthesis,
	ListEnd,

    WhiteSpace,
    At,
    Equals,
    Dollar,
    NewLine,
}

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

                ';' | '\n' => {
                    self.input.incr(1);
                    self.push_token(TokenKind::Semicolon)
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
                    self.push_token(TokenKind::Equals);
                }

                _ => self.tokenize_expression(),
            }
        }
    }

    fn tokenize_expression(&mut self) {
        while let Some(c) = self.input.get(0) {
            match c {
                // ';'  => {self.input.incr(1); self.push_token(TokenKind::Semicolon)},
                '$'  => {self.input.incr(1); self.push_token(TokenKind::Dollar)},
                '%'  => {self.input.incr(1); self.push_token(TokenKind::Identifier)},
                '.'  => {self.input.incr(1); self.push_token(TokenKind::Identifier)},
                '*'  => {self.input.incr(1); self.push_token(TokenKind::Identifier)},
                '&'  => {self.input.incr(1); self.push_token(TokenKind::Identifier)},
                '!'  => {self.input.incr(1); self.push_token(TokenKind::Identifier)},
                '+'  => {self.input.incr(1); self.push_token(TokenKind::Identifier)},
                '-'  => {self.input.incr(1); self.push_token(TokenKind::Identifier)},
                '|'  => {self.input.incr(1); self.push_token(TokenKind::Identifier)},

                ':'  => {self.input.incr(1); self.push_token(TokenKind::Identifier)},
                '<'  => {self.input.incr(1); self.push_token(TokenKind::Identifier)},
                '>'  => {self.input.incr(1); self.push_token(TokenKind::Identifier)},
                '~'  => {self.input.incr(1); self.push_token(TokenKind::Identifier)},
                '@'  => {self.input.incr(1); self.push_token(TokenKind::Identifier)},


                '[' => {self.input.incr(1); self.push_token(TokenKind::Identifier)},
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
              
                // '\n' => {self.input.incr(1); self.skip_token()},
                '\t' | ' ' => {
                    self.input.incr_while(|x| x == '\t' || x == ' ');
                    self.skip_token();
                },
            
                '{' | '"' => self.tokenize_template(true),
                
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

    fn tokenize_template(&mut self, brackets: bool) {
        if brackets {
            assert_eq!(self.input.get(0), Some('{'));
            self.input.incr(1);
        }

        self.push_token(TokenKind::TemplateOpen);
        let special_characters = ['{', '\\', '$', '}'];
        let close_test = if brackets { Some('}') } else { None };

        while self.input.get(0) != close_test {
            let c = self.input.get(0).unwrap();
            match c {
                '{'  => self.tokenize_template(true),
                '$'  => self.tokenize_dollar(),
                '\\' => self.tokenize_escape_sequence(),

                _  => {
                    self.input.incr_while(|x| !special_characters.contains(&x));
                    self.push_token(TokenKind::TemplateText);
                }
            }
        }

        if brackets { self.input.incr(1); }
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
			self.input.incr_while(|x| x.is_alphanumeric() || x == '_' || x == '.' || x == ':');
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

pub fn tokenize_template<'input>(input: &'input [char]) -> Vec<Token<TokenKind>> {
    let mut cursor = Cursor::new(input);
    cursor.tokenize_template(false);
    std::mem::take(&mut cursor.output)
}
