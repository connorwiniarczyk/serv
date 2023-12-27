// use crate::parser::{AstNode, ExecutionEngine};
use crate::parser::{AstNode};
use crate::routetree::RouteTree;
use hyper::{Request, Response};
use crate::value::{Table};
use crate::value;
use crate::engine::Engine;

use crate::matchit::Router;

pub struct RouteTable(Router<Vec<AstNode>>);

impl RouteTable {
    pub fn new(input: AstNode) -> Result<Self, ()> {
        let AstNode::Root(tree) = input else { return Err(()) };
        let mut output: Router<Vec<AstNode>> = Router::new();
        for route in tree {
            let AstNode::Route((pattern, expression)) = route else { return Err(()) }; 
            let (AstNode::Pattern(p), AstNode::Expression(e)) = (*pattern, *expression) else { return Err(()) };
            output.insert(p, e);
        }
        Ok(Self(output))
    }

    pub async fn resolve(&self, mut req: Request<hyper::Body>) -> Result<Response<hyper::Body>, hyper::Error> {
        // println!("{:?}", req.uri().path());
        let expression = self.0.at(req.uri().path()).unwrap();
        println!("{:?}", expression);

        let mut output = hyper::Response::builder().status(500);
        Ok(output.body(String::new().into()).unwrap())

        // todo!();

        // let expression = self.0.get(req.uri().path());

        // if let Some((e, vars)) = expression {
        //     let mut engine = Engine::new::<Table>(vars.into());
        //     let result = engine.resolve_expression(e).unwrap();
        //     let response = value::Response {
        //         headers: Vec::new(),
        //         body: result,
        //     };
        //     Ok(response.into())
        // }

        // else {
		    // let mut output = hyper::Response::builder().status(404);
        //     Ok(output.body(String::new().into()).unwrap())
        // }
    }
}



#[cfg(test)]
mod test {
    use super::*;


    #[test]
    fn test() {
         
    }
}
