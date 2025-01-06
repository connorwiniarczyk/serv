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
