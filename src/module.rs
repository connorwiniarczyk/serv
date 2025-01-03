use crate::ServValue;
use crate::Stack;
use crate::Label;
use crate::ServFn;
use crate::ServResult;

use std::collections::VecDeque;

#[derive(Clone, Debug, Default)]
pub struct Expression(pub VecDeque<ServValue>, pub bool);

impl Expression {
    pub fn empty() -> Self {
        Self(VecDeque::new(), false)
    }

    pub fn next(&mut self) -> Option<ServValue> {
        self.0.pop_front()
    }

    pub fn push(&mut self, input: ServValue) {
        self.0.push_front(input)
    }

    pub fn push_back(&mut self, input: ServValue) {
        self.0.push_back(input)
    }

    pub fn prepend<I: Iterator<Item = ServValue> + std::iter::DoubleEndedIterator>(&mut self, input: I) {
        for value in input.rev() {
            self.0.push_front(value);
        }
    }

    pub fn as_list(self) -> ServValue {
        ServValue::List(self.0)
    }

    pub fn eval(&mut self, scope: &mut crate::Stack) -> ServResult {
        let Some(mut next) = self.next() else { return Ok(ServValue::None) };

        if let ServValue::Ref(ref label) = next {
            if let Ok(value) = scope.get(label.clone()) {
                next = value;
            }
        }

        match next {
			ServValue::Func(ServFn::Expr(e, _)) => {
                self.prepend(e.0.into_iter());
                self.eval(scope)
			},

            ServValue::Func(ServFn::Meta(f)) => {
                f(std::mem::take(self), scope)
                // todo!();
            },

            ServValue::Func(ServFn::ArgFn(f)) => {
                let arg = self.next().ok_or("argument expected")?;
                let rest = self.eval(scope)?;
                f(arg, rest, scope)
            },

            f => f.call(Some(self.eval(scope)?), scope),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Element {
    pub pattern: Option<ServValue>,
    pub action: Expression,
}

#[derive(Clone, Debug, Default)]
pub struct ServModule {
    pub statements:  Vec<Expression>,
    pub routes:      Vec<(String, Expression)>,
    pub definitions: Vec<(Label, Expression)>,
    pub equalities:  Vec<(Expression, Expression)>,
}

impl ServModule {

    pub fn push_element(&mut self, input: Element) {
        let Element { pattern, action } = input;
        match (pattern, action) {
            (None, expr) => self.statements.push(expr),
            (Some(ServValue::Ref(label)), expr) => self.definitions.push((label, expr)),
            (Some(ServValue::Func(ServFn::Route(r))), expr) => self.routes.push((r, expr)),
            (Some(ServValue::Func(ServFn::Expr(e, _))), expr) => {
                // println!("{:?}", e);
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
