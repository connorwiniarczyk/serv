pub mod cursor;
pub mod grammar;

use cursor::Cursor;

#[derive(Debug)]
pub struct SyntaxError;

#[derive(Debug)]
pub struct MatchFail;

#[derive(Debug)]
pub enum PatternSegment {
	Value(String),
	Wildcard(String),
	DeepWildcard(String),
}

#[derive(Debug)]
pub enum AstNode {
	Root(Vec<AstNode>),
	Route((Box<AstNode>, Box<AstNode>)),
	Pattern(String),

	// Pattern(Vec<AstNode>),
	// PathSegment(Box<AstNode>),
	// PathExtension(Box<AstNode>),

	// Value(String),
	// Wildcard(String),
	// DeepWildcard(String),

	Expression(Vec<AstNode>),
	Function(String),
	Variable(String),
	Template(Vec<AstNode>),
	Text(String),
}

pub fn parse(input: &str) -> Result<AstNode, SyntaxError> {
	let text: Vec<char> = input.chars().collect();
	let mut cursor = Cursor::new(&text);
	let result = grammar::root(&mut cursor).map_err(|e| SyntaxError);
	result
}
