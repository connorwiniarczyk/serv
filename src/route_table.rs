use crate::ast::{AstNode, ExecutionEngine};
use crate::routetree::RouteTree;
use hyper::{Request, Response};
use crate::value;

pub struct RouteTable(RouteTree<AstNode>);

impl RouteTable {
    pub fn new(input: RouteTree<AstNode>) -> Self {
        Self(input)
    }

    pub async fn resolve(&self, mut req: Request<hyper::Body>) -> Result<Response<hyper::Body>, hyper::Error> {

        let expression = self.0.get(req.uri().path());

        if let Some((e, vars)) = expression {
            let mut engine = ExecutionEngine::new(vars.into());
            let result = engine.resolve_expression(e).unwrap();
            let response = value::Response {
                headers: Vec::new(),
                body: result,
            };
            Ok(response.into())
        }

        else {
		    let mut output = hyper::Response::builder().status(404);
            Ok(output.body(String::new().into()).unwrap())
        }
    }
}



#[cfg(test)]
mod test {
    use super::*;


    #[test]
    fn test() {
         
    }
}
