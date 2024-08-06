#[derive(Clone, Copy, Debug, PartialEq)]
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

    LambdaBegin,
    LambdaEnd,

    WhiteSpace,
    At,
    WideArrow,
    Semicolon,
    Plus,
    Dollar,
    Percent,
    NewLine,
    Equals,

    EndOfInput,
}

impl std::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub contents: String,
    pub start: usize,
    pub end: usize,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		write!(f, "({} {:?})", self.kind, self.contents)?;
		Ok(())
    }
}

pub struct TokenList(pub Vec<Token>);

impl std::fmt::Display for TokenList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let mut iter = self.0.iter().peekable();
        while let Some(token) = iter.next() {
			token.fmt(f)?;
			f.write_str("\n");
        }
        Ok(())
    }
}

struct Cursor<'input> {
    input: &'input [char],
    index: usize,
    mark: usize,
}

impl<'input> Cursor<'input> {
    fn new(input: &'input [char]) -> Self {
        Self { input, index: 0, mark: 0 }
    }

    fn incr(&mut self, offset: usize) {
        self.index += offset;
    }

    fn incr_while<F>(&mut self, test: F) where F: Fn(char) -> bool {
        while (self.get(0).is_some() && (test)(self.get(0).unwrap())) {
            self.incr(1);
        }
    }

    fn get(&self, offset: usize) -> Option<char> {
        if (self.index + offset < self.input.len()) {
            Some(self.input[self.index + offset])
        } else {
            None
        }
    }
    
    fn emit_special(&mut self, text: &str) -> Token {
        let output = Token {
            start: self.mark,
            end: self.index,
            contents: text.to_string(),
            kind: TokenKind::TemplateText,
        };
        self.mark = self.index;
        output
    }

    fn emit_token(&mut self, kind: TokenKind) -> Token {
        let mut contents = String::new();
        for i in self.mark..self.index {
            contents.push(self.input[i]);
        }
        let output = Token {
            start: self.mark,
            end: self.index,
            contents,
            kind,
        };
        self.mark = self.index;
        return output;
    }
}

fn tokenize_string_template(cursor: &mut Cursor, output: &mut Vec<Token>) {
    cursor.incr(1);
    output.push(cursor.emit_token(TokenKind::TemplateOpen));

    let special_characters = ['{', '\\', '$', '}'];
    // let special_characters = ['{', '\\', '"', '$', '}'];
    
    while let Some(c) = cursor.get(0) {
        if special_characters.contains(&c) {
            match c {
                '}' => {
                    cursor.incr(1);
                    output.push(cursor.emit_token(TokenKind::TemplateClose));
                    break;
                },

                '{' => tokenize_string_template(cursor, output),

                '$' if cursor.get(1) == Some('$') => {
                    cursor.incr(1);
                    _ = cursor.emit_token(TokenKind::Dollar);
                    cursor.incr(1);
                    output.push(cursor.emit_token(TokenKind::TemplateText))
                },

                '$' => {
                    cursor.incr(1);
                    output.push(cursor.emit_token(TokenKind::Dollar));

                    if cursor.get(0) == Some('(') {
                        cursor.incr(1);
                        output.push(cursor.emit_token(TokenKind::LambdaBegin));
                        tokenize_inner_expression(cursor, output);
                    }

                    // else if cursor.get(0) == Some('$') {
                    //     cursor.incr(1);
                    //     // output.push(cursor.emit_token(TokenKind::Dollar));
                    //     // cursor.incr(1);
                    // }

                    else {
                        cursor.incr_while(|x| x.is_alphanumeric() || x == '_' || x == '.');
                        output.push(cursor.emit_token(TokenKind::Identifier));
                    }

                },

                '\\' if cursor.get(1) == Some('n') => {
                    cursor.incr(2);
                    output.push(cursor.emit_special("\n"));
                },

                '\\' if cursor.get(1) == Some('t') => {
                    cursor.incr(2);
                    output.push(cursor.emit_special("\t"));
                },

                '\\' if cursor.get(1) == Some('{') => {
                    cursor.incr(2);
                    output.push(cursor.emit_special("{"));
                },

                '\\' if cursor.get(1) == Some('}') => {
                    cursor.incr(2);
                    output.push(cursor.emit_special("}"));
                },
                // '"' => todo!()

                _ => unreachable!(),
            }
        }

        else {
            cursor.incr_while(|x| !special_characters.contains(&x));
            output.push(cursor.emit_token(TokenKind::TemplateText));
        }
    }
}

fn tokenize_inner_expression(cursor: &mut Cursor, output: &mut Vec<Token>) {
    while let Some(c) = cursor.get(0) {
        match c {
            ','  => {cursor.incr(1); output.push(cursor.emit_token(TokenKind::Comma))},
            ';'  => {cursor.incr(1); output.push(cursor.emit_token(TokenKind::Semicolon))},
            '$'  => {cursor.incr(1); output.push(cursor.emit_token(TokenKind::Dollar))},
            '%'  => {cursor.incr(1); output.push(cursor.emit_token(TokenKind::Identifier))},
            '.'  => {cursor.incr(1); output.push(cursor.emit_token(TokenKind::Identifier))},
            '*'  => {cursor.incr(1); output.push(cursor.emit_token(TokenKind::Identifier))},
            '!'  => {cursor.incr(1); output.push(cursor.emit_token(TokenKind::Identifier))},
            '\n' => {cursor.incr(1); _ = cursor.emit_token(TokenKind::NewLine)},
            
            '[' => {cursor.incr(1); output.push(cursor.emit_token(TokenKind::ListBegin))},
            ']' => {cursor.incr(1); output.push(cursor.emit_token(TokenKind::ListEnd))},
            
            '(' => {cursor.incr(1); output.push(cursor.emit_token(TokenKind::LambdaBegin)); tokenize_inner_expression(cursor, output); },
            ')' => {cursor.incr(1); output.push(cursor.emit_token(TokenKind::LambdaEnd)); break; },
          
            '\t' | ' ' => {
                cursor.incr_while(|x| x == '\t' || x == ' ');
                _ = cursor.emit_token(TokenKind::WhiteSpace)
            },
        
            '{' | '"' => tokenize_string_template(cursor, output),
            
            c @ _ if c.is_alphabetic() => {
                cursor.incr_while( |x| x.is_alphanumeric() || x == '_' || x == '.');
                output.push(cursor.emit_token(TokenKind::Identifier))
            },
            
            c @ _ if c.is_digit(10) => {
                cursor.incr_while(|x| x.is_digit(10));
                output.push(cursor.emit_token(TokenKind::IntLiteral))
            },

            _ => return,
        }
	}
}

pub fn tokenize(input: &str) -> TokenList {
    let chars: Vec<char> = input.chars().collect();
    let mut cursor = Cursor::new(&chars);
    let mut output: Vec<Token> = Vec::new();
	
    while let Some(c) = cursor.get(0) {
        match c {
            '@'  => {cursor.incr(1); output.push(cursor.emit_token(TokenKind::At))},
            '#'  => {
                cursor.incr(1);
                cursor.incr_while(|x| x != '\n' && x != '#');
                _ = cursor.emit_token(TokenKind::Comment);
            },
            '=' if cursor.get(1) == Some('>') => {
                cursor.incr(2);
                output.push(cursor.emit_token(TokenKind::WideArrow))
            },
            '=' => { cursor.incr(1); output.push(cursor.emit_token(TokenKind::WideArrow))},
            '/' => {
                cursor.incr_while(|x| !x.is_whitespace());
                output.push(cursor.emit_token(TokenKind::Route))
            }
            _ => tokenize_inner_expression(&mut cursor, &mut output),

            // '\t' | ' ' => {
            //     cursor.incr_while(|x| x == '\t' || x == ' ');
            //     _ = cursor.emit_token(TokenKind::WhiteSpace)
            // },

            // '{' | '"' => tokenize_string_template(&mut cursor, &mut output),
            
            // c @ _ if c.is_alphabetic() => {
            //     cursor.incr_while(|x| x.is_alphanumeric() || x == '_' || x == '.');
            //     output.push(cursor.emit_token(TokenKind::Identifier))
            // },
            
            // c @ _ if c.is_digit(10) => {
            //     cursor.incr_while(|x| x.is_digit(10));
            //     output.push(cursor.emit_token(TokenKind::IntLiteral))
            // },
            //
            // ';'  => {cursor.incr(1); output.push(cursor.emit_token(TokenKind::Semicolon))},
            // '$'  => {cursor.incr(1); output.push(cursor.emit_token(TokenKind::Dollar))},
            // '%'  => {cursor.incr(1); output.push(cursor.emit_token(TokenKind::Identifier))},
            // '*'  => {cursor.incr(1); output.push(cursor.emit_token(TokenKind::Identifier))},
            // '!'  => {cursor.incr(1); output.push(cursor.emit_token(TokenKind::Identifier))},
            // '\n' => {cursor.incr(1); _ = cursor.emit_token(TokenKind::NewLine)},
            
            // '[' => {cursor.incr(1); output.push(cursor.emit_token(TokenKind::ListBegin))},
            // ']' => {cursor.incr(1); output.push(cursor.emit_token(TokenKind::ListEnd))},
            // ',' => {cursor.incr(1); output.push(cursor.emit_token(TokenKind::Comma))},
            
            // '(' => {cursor.incr(1); output.push(cursor.emit_token(TokenKind::LambdaBegin))},
            // ')' => {cursor.incr(1); output.push(cursor.emit_token(TokenKind::LambdaEnd))},

            // c @ _ => panic!("{}", c),
        }
    }

    output.push(cursor.emit_token(TokenKind::EndOfInput));
    return TokenList(output)
}
