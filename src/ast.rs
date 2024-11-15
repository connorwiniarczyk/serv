use crate::value::ServValue;
pub use crate::template::{TemplateElement, Template};

use std::fmt::Display;

#[derive(Clone)]
pub enum Word {
   	// Function(Token)),
   	Function(String),
   	Template(Template),
   	Literal(ServValue),
   	Parantheses(Expression),
   	List(Vec<Expression>),
}

impl Display for Word {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::Function(t) => f.write_str(t)?,
            Self::Template(t) => t.fmt(f)?,
            Self::Literal(l)  => l.fmt(f)?,
            Self::List(l) => todo!(),
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
pub struct Expression(pub Vec<Word>);

// #[derive(Debug)]
pub struct Declaration {
   	pub kind: String,
	pub key: String,
	pub value: Expression,
}

// #[derive(Debug)]
pub struct AstRoot(pub Vec<Declaration>);
