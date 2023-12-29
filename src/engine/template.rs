use super::*;

#[derive(Debug)]
enum TemplateElement {
    Var(String),
    Text(String),
}

#[derive(Debug)]
pub struct Template(Vec<TemplateElement>);

impl Template {
    pub fn from_ast(input: AstNode) -> Result<Self, ()> { 
        let AstNode::Template(t) = input else { return Err(()) };
        let mut output: Vec<TemplateElement> = Vec::new();
        for node in t {
            match node {
                AstNode::Variable(v) => output.push(TemplateElement::Var(v)),
                AstNode::Text(t) => output.push(TemplateElement::Text(t)),
                _ => todo!(),
            }
        }
        Ok(Self(output))
    }

    pub fn eval(&self, context: Value) -> Value {
        let mut output = String::new();
        let mut elements = self.0.iter();
        while let Some(e) = elements.next() {
            match e {
                TemplateElement::Text(ref s) => output.push_str(s),
                TemplateElement::Var(ref s) => output.push_str(context.get(s)),
            }
        }
        Value::Text(output)
    }
}

// impl ServFunctionT for Template {
//     fn call(&self, input: Value) -> Value {
//         let mut output = String::new();
//         let mut elements = self.0.iter();
//         while let Some(e) = elements.next() {
//             match e {
//                 TemplateElement::Text(ref s) => output.push_str(s),
//                 TemplateElement::Var(ref s) => output.push_str(s),
//             }
//         }
//         Value::Text(output)
//     }
// }

// impl Template {
// 	pub fn render(&self, vars: &Table) -> String {
// 		let mut output = String::new();

// 		for piece in self.elements.iter() {
// 			match piece {
// 				Literal(l) => { 
// 					output.push_str(l);
// 				}, 
// 				Variable(v) => { 
// 					let value = vars.get(&v.to_string()).unwrap_or_default();
// 					output.push_str(&value.to_string());
// 				}, 
// 			}
// 		}

// 		return output
// 	}
// }

