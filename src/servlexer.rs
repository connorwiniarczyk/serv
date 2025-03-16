use parsetool::cursor::{Tokenizer, Token};

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum TokenKind {
    Identifier,
    Route,
    Equals,

    IntLiteral,

    TemplateOpen,
    TemplateClose,
    TemplateText,
    TemplateVariable,
    Dollar,

    ModuleOpen,
    ModuleClose,
    ModuleSeparator,

    Comment,
    Include,
    At,
}

fn tokenize_module(cursor: &mut Tokenizer, output: &mut Vec<Token<TokenKind>>) {
    while let Some(c) = cursor.get(0) {
        match c {
            ';' | ',' | '\n' => {
                cursor.incr(1);
                cursor.emit_to(output, TokenKind::ModuleSeparator)
            },

            '#'  => {
                cursor.incr(1);
                cursor.incr_while(|x| x != '\n' && x != '#');
                cursor.skip_token();
            },

            '=' => {
                let i = if cursor.get(1) == Some('>') {2} else {1};
                cursor.incr(i);
                cursor.emit_to(output, TokenKind::Equals);
            },

            ')' => {
                cursor.incr(1);
                cursor.emit_to(output, TokenKind::ModuleClose);
                return;
            },

            otherwise => tokenize_expression(cursor, output),
        }
    }
}

fn tokenize_expression(cursor: &mut Tokenizer, output: &mut Vec<Token<TokenKind>>) {
    let identifiers: [char; 13] = [ '%', '*', '&', '!', '+', '-', '|', ':', '<', '>', '?', '~', '[' ];

    while let Some(c) = cursor.get(0) {
        match c {
            '@'  => { cursor.incr(1); cursor.emit_to(output, TokenKind::At) },


            '/' => {
                cursor.incr_while(|x| !x.is_whitespace());
                cursor.emit_to(output, TokenKind::Route);
            },

            '(' => {
                cursor.incr(1);
                cursor.emit_to(output, TokenKind::ModuleOpen);
                tokenize_module(cursor, output);
            },
          
            '\t' | ' ' => {
                cursor.incr_while(|x| x == '\t' || x == ' ');
                cursor.skip_token();
            },
        
            '{' => tokenize_template(cursor, output, true),
            
            c if identifiers.contains(&c) => { cursor.incr(1); cursor.emit_to(output, TokenKind::Identifier) },

            '"' => {
				cursor.incr(1);
				cursor.skip_token();
				cursor.incr_while(|x| x != '"');
				cursor.emit_to(output, TokenKind::Identifier);
				cursor.incr(1);
				cursor.skip_token();
            },

            c if c.is_alphabetic() => {
                cursor.incr_while(|x| x.is_alphanumeric() || x == '_' || x == '.');

				// include needs to be special since it is executed at parsetime
                let mut ident = cursor.emit(TokenKind::Identifier);
                if ident.value == "include" { ident.kind = TokenKind::Include };
                output.push(ident);
            },
            
            c if c.is_digit(10) => {
                cursor.incr_while(|x| x.is_digit(10));
                cursor.emit_to(output, TokenKind::IntLiteral);
            },

            otherwise => return,
        }
	}
}

fn tokenize_template(cursor: &mut Tokenizer, output: &mut Vec<Token<TokenKind>>, brackets: bool) {
    if brackets {
        assert_eq!(cursor.get(0), Some('{'));
        cursor.incr(1);
    }

    cursor.emit_to(output, TokenKind::TemplateOpen);
    let special_characters = ['{', '\\', '$', '}'];
    let close_test = if brackets { Some('}') } else { None };

    while cursor.get(0) != close_test {
        let c = cursor.get(0).unwrap();
        match c {
            '{'  => tokenize_template(cursor, output, true),
            '$'  => tokenize_dollar(cursor, output),
            '\\' => tokenize_escape_sequence(cursor, output),

            c  => {
                cursor.incr_while(|x| !special_characters.contains(&x));
                cursor.emit_to(output, TokenKind::TemplateText);
            }
        }
    }

    if brackets { cursor.incr(1); }

    cursor.emit_to(output, TokenKind::TemplateClose);
}


fn tokenize_dollar(cursor: &mut Tokenizer, output: &mut Vec<Token<TokenKind>>) {
    assert_eq!(cursor.get(0), Some('$'));

	// treat '$$' as escaped '$'
    if cursor.get(1) == Some('$') {
        cursor.incr(1);
        cursor.skip_token();
        cursor.incr(1);
        cursor.emit_to(output, TokenKind::TemplateText);
        return;
    }

    cursor.incr(1);
    cursor.emit_to(output, TokenKind::Dollar);

	// if we see a parentheses, tokenize a whole expression
    if cursor.get(0) == Some('(') {
        cursor.incr(1);
        cursor.emit_to(output, TokenKind::ModuleOpen);
        tokenize_module(cursor, output);
    }

    else if cursor.get(0) == Some('"') {
    	cursor.incr(1);
    	cursor.skip_token();
    	cursor.incr_while(|x| x != '"');
    	cursor.emit_to(output, TokenKind::Identifier);
    	cursor.incr(1);
    	cursor.skip_token();
    }

    else {
		cursor.incr_while(|x| x.is_alphanumeric() || x == '_' || x == '.' || x == ':');
		cursor.emit_to(output, TokenKind::Identifier);
    }
}

fn tokenize_escape_sequence(cursor: &mut Tokenizer, output: &mut Vec<Token<TokenKind>>) {
    todo!("tell connor to implement escape sequences");
}

pub fn tokenize<'input>(input: &'input [char]) -> Vec<Token<TokenKind>> {
    let mut cursor = Tokenizer::new(input);
    let mut output = Vec::new();

    tokenize_module(&mut cursor, &mut output);
    output
}

// pub fn tokenize_template<'input>(input: &'input [char]) -> Vec<Token<TokenKind>> {
    // todo!();

    // let mut cursor = Cursor::new(input);
    // cursor.tokenize_template(false);
    // std::mem::take(&mut cursor.output)
// }
