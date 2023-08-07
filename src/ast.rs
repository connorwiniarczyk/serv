use crate::parser::{FromSyntax, ParseError, Tree};
use std::collections::HashMap;
use std::convert::TryFrom;
use hyper;

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

pub struct Value {
    headers: Vec<(String, String)>,
    body: String,
}

impl From<Value> for hyper::Response<hyper::Body> {
    fn from(value: Value) -> Self {
        let mut output = hyper::Response::builder().status(200);

        output.body(value.body.into()).unwrap()
    }
}

pub struct ExecutionEngine;

impl ExecutionEngine {

    pub fn new() -> Self {
        Self
    }

    pub fn resolve_expression(&mut self, expression: &AstNode) -> Result<Value, ()> {

        let output: Value = match expression {
            AstNode::Text(s) => Value { headers: Vec::new(), body: s.clone() },
            AstNode::Prefix { op, options, value } => { 
                let input: Option<Value> = match value {
                    Some(v) => Some(self.resolve_expression(v)?),
                    None => None,
                };

                match op {
                    Operator::Html => {
                        let mut output = String::new();
                        output.push_str("<html>\n<body>\n");
                        if let Some(ref v) = input { output.push_str(v.body.as_str()); }
                        output.push_str("\n</body>\n</html>");

                        Value { headers: Vec::new(), body: output }
                    },
                    _ => todo!(),
                }
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
