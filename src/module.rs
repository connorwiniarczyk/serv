use crate::ServValue;
use crate::Stack;
use crate::Label;
use crate::ServFn;
use crate::ServResult;
use crate::value::ServList;

#[derive(Clone, Debug)]
pub struct Element {
    pub pattern: Option<ServValue>,
    pub action: ServList,
}

#[derive(Clone, Debug, Default)]
pub struct ServModule {
    pub statements:  Vec<ServList>,
    pub routes:      Vec<(String, ServList)>,
    pub definitions: Vec<(Label, ServList)>,
    pub equalities:  Vec<(ServList, ServList)>,
}

impl ServModule {
    pub fn call(self, input: Option<ServValue>, scope: &mut Stack) -> ServResult {
		if self.statements.len() == 0 { return Ok(ServValue::Module(self)) }

        for (label, expr) in self.definitions {
			scope.insert(label, ServValue::Func(ServFn::Expr(expr, false)));
        }

        let mut output = ServValue::None;
        for mut expr in self.statements {
            if let Some(ref i) = input {
                expr.push_back(i.clone());
            }
            output = expr.eval(scope)?;
        }

		Ok(output)
    }

    pub fn push_element(&mut self, input: Element) {
        let Element { pattern, action } = input;
        match (pattern, action) {
            (None, expr) => self.statements.push(expr),
            (Some(ServValue::Ref(label)), expr) => self.definitions.push((label, expr)),
            (Some(ServValue::Func(ServFn::Route(r))), expr) => self.routes.push((r, expr)),
            (Some(ServValue::Func(ServFn::Expr(e, _))), expr) => {
                self.equalities.push((e, expr))
            },
            _ => panic!("invalid element"),
        }
    }

    pub fn from_elements<I>(input: I) -> Self where I: Iterator<Item = Element> {
        let mut output = Self::default();
        for e in input {
            output.push_element(e);
        }
        output
    }
}
