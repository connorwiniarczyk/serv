// use crate::parser::{AstNode, ExecutionEngine};
use crate::parser::{AstNode};
use hyper::{Request, Response};

use crate::matchit::Router;
use crate::engine::{Expression, ServFunction};
use std::sync::Arc;
use crate::engine::Value;

pub struct RouteTable(Router<Expression>);

// pub struct RouteTable(Router<Arc<dyn ServFunction>>);

impl RouteTable {
    pub fn new(input: AstNode) -> Result<Self, ()> {
        let AstNode::Root(tree) = input else { return Err(()) };
        let mut output = Router::new();
        for route in tree {
            let AstNode::Route((pattern, expression)) = route else { return Err(()) }; 
            let AstNode::Pattern(p) = *pattern else { return Err(()) };
            output.insert(p, Expression::from_node(*expression)?);
        }
        Ok(Self(output))
    }

    pub async fn resolve(&self, mut req: Request<hyper::Body>) -> Result<Response<hyper::Body>, hyper::Error> {
        let matched = self.0.at(req.uri().path()).unwrap();

        println!("{:?}", matched.value);

        Ok(matched.value.eval().into())

        // match matched.value.run() {
        //     Value::Text(t) => {
        //         println!("{}", t);
        //     },

        //     _ => todo!(),
        // };

        // let mut output = hyper::Response::builder().status(500);
        // Ok(output.body(String::new().into()).unwrap())

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
