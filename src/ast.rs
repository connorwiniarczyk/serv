use crate::lexer::Token;
use crate::value::ServValue;
pub use crate::template::{TemplateElement, Template};

#[derive(Debug, Clone)]
pub enum Word {
   	Function(Token),
   	Template(Template),
   	Literal(ServValue),
   	Parantheses(Expression),
}

#[derive(Debug, Clone)]
pub struct Expression(pub Vec<Word>);

#[derive(Debug)]
pub struct Declaration {
   	pub kind: String,
	pub key: String,
	pub value: Expression,
}

#[derive(Debug)]
pub struct AstRoot(pub Vec<Declaration>);
