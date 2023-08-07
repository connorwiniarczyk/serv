use crate::parser::{FromSyntax, ParseError, Tree};
use std::collections::HashMap;
use std::convert::TryFrom;

struct Template;

pub enum Operator {
    Html,
}

impl TryFrom<&str> for Operator {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let output = match value {
            "html" => Operator::Html,
            _ => return Err("invalid operator name"),
        };

        Ok(output)
    }
}

pub enum AstNode {
    Placeholder,
    Template(Template),
    Text(String),
    Prefix {
        op: Operator,
        options: HashMap<String, String>,
        value: Option<Box<AstNode>>
    },
    Add {
        left: Box<AstNode>,
        right: Box<AstNode>,
    }
}

struct Value(String);

pub struct ExecutionEngine;

impl ExecutionEngine {
    pub fn ResolveExpression(&mut self, expression: &AstNode) -> Result<Value, ()> {

        let output: Value = match expression {
            AstNode::Text(s) => {
                Value(s.to_string())
            },

            _ => todo!(),
        };

        Ok(output)
    }
}

#[cfg(test)]
mod test {
    
    #[test]
    fn test() {}
}
