use crate::ast::{AstNode, ExecutionEngine};
use crate::routetree::RouteTree;
use hyper::{Request, Response};

pub struct RouteTable(RouteTree<AstNode>);

impl RouteTable {
    pub fn new(input: RouteTree<AstNode>) -> Self {
        Self(input)
    }

    pub async fn resolve(&self, mut req: Request<hyper::Body>) -> Result<Response<hyper::Body>, hyper::Error> {

        let expression = self.0.get(req.uri().path()).unwrap();
        let mut engine = ExecutionEngine::new();

        let result = engine.resolve_expression(expression).unwrap();

        Ok(result.into())
    }
}



#[cfg(test)]
mod test {
    use super::*;


    #[test]
    fn test() {
         
    }
}
