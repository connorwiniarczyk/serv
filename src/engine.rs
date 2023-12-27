use crate::parser::AstNode;
use crate::value::{ Value, Table };

pub struct Engine;

impl Engine {
    pub fn new<I>(input: I) -> Self
    where I: Into<Table> {
        todo!();
    }

    pub fn resolve_expression(&mut self, e: &AstNode) -> Result<Value, ()> {
        todo!();
    }
}
