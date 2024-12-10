use crate::value::ServValue;
pub use crate::template::{TemplateElement, Template};

use std::fmt::Display;

#[derive(Clone)]
pub enum Word {
   	Function(String),
   	Template(Template),
   	Literal(ServValue),
   	Parantheses(Expression),
}

impl Display for Word {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::Function(t) => f.write_str(t)?,
            Self::Template(t) => t.fmt(f)?,
            Self::Literal(l)  => l.fmt(f)?,
            Self::Parantheses(e) => {
				f.write_str("(")?;
				for word in e.0.iter() {
    				word.fmt(f)?;
    				f.write_str(" ")?;
				}
				f.write_str(")")?;
            },
        }

		Ok(())
    }
}

#[derive(Clone)]
pub struct Expression(pub Vec<Word>, pub bool);

pub enum Pattern {
    Key(String),
    Expr(Expression),
}

// #[derive(Debug)]
pub struct Declaration {
   	pub kind: String,
	pub key: Pattern,
	pub value: Expression,
}

impl Declaration {
    pub fn with_key(kind: &str, key: &str, value: Expression) -> Self {
        Self {
            kind: kind.to_owned(),
            key: Pattern::Key(key.to_owned()),
            value: value,
        }
    }

    pub fn with_expr(kind: &str, key: Expression, value: Expression) -> Self {
        Self {
            kind: kind.to_owned(),
            key: Pattern::Expr(key),
            value: value,
        }
    }

    pub fn key(&self) -> String {
        match &self.key {
            Pattern::Key(s) => s.clone(),
            Pattern::Expr(e) => panic!(),
        }
    }
}

// #[derive(Debug)]
pub struct AstRoot(pub Vec<Declaration>);
