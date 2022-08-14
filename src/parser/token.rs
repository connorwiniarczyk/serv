#[derive(Clone, Copy, PartialEq, Debug)]
pub enum TokenKind {
    Root,
    Route,
    Path,
    PathAttribute,
    PathNode,
    PathExt,
    CommandList,
    Command,
    CommandName,
    CommandArg,
    Comment,
    MultiLine,
}

#[derive(Clone, Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub children: Vec<Token>,
    pub value: Option<String>,
}

impl Token {
    pub fn new(kind: TokenKind) -> Self {
        Self {
            kind,
            children: vec![],
            value: None,
        }
    }

    pub fn set_value(&mut self, value: &str) {
        match value {
            "" => self.value = None,
            _  => self.value = Some(value.to_string()),
        }
    }

    pub fn add_child(&mut self, child: Token) {
        self.children.push(child);
    }

    pub fn get_child(&self, kind: TokenKind) -> Option<&Token> {
        self.children.iter().find(|x| x.kind == kind)
    }
}

use std::fmt::{Display, Formatter};

impl Display for Token {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {

        fn write(next: &Token, f: &mut Formatter, depth: u32) -> std::fmt::Result {

            // write tab characters to the beginning of the line
            for _ in 0..depth { f.write_str("  ")?; }

            f.write_str(&format!("{:?}", next.kind))?;

            if let Some(value) = &next.value {
                f.write_str(&format!( "({:?})", value))?;
            }

            f.write_str("\n")?;

            for child in &next.children {
                write(child, f, depth + 1)?;
            }

            Ok(())
        }

        write(self, f, 0)
    }
}
