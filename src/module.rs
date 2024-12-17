use crate::ast;
use crate::ServValue;
use crate::Stack;
use crate::Label;
use crate::ServFn;
use crate::ServResult;

use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub struct Expression(VecDeque<ServValue>, bool);

impl Expression {
    pub fn next(&mut self) -> Option<ServValue> {
        self.0.pop_front()
    }

    pub fn push(&mut self, input: ServValue) {
        self.0.push_front(input)
    }

    pub fn push_back(&mut self, input: ServValue) {
        self.0.push_back(input)
    }

    fn compile_word(input: ast::Word) -> ServValue {
		match input {
			ast::Word::Function(v) => ServValue::Ref(Label::Name(v)),
			ast::Word::Literal(v) => v,
			ast::Word::Template(v) => ServValue::Func(ServFn::Template(v)),
			ast::Word::Parantheses(e) => {
    			let is_meta = e.1;
    			let inner = Self::compile(e);
    			// ServValue::Func(ServFn::Expr(inner.0, is_meta))
    			ServValue::Func(ServFn::Expr(inner, is_meta))
			},
			_ => panic!(),
		}
    }
    pub fn compile(input: ast::Expression) -> Self {
        let is_meta = input.1;
        let mut output = VecDeque::new();
        for word in input.0.into_iter() {
            output.push_back(Self::compile_word(word));
        }

		Self(output, is_meta)
    }

    fn prepend<I: Iterator<Item = ServValue> + std::iter::DoubleEndedIterator>(&mut self, input: I) {
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
            if let Some(value) = scope.get(label.clone()) {
                next = value;
            }
        }

        match next {
			ServValue::Func(ServFn::Expr(e, _)) => {
                self.prepend(e.0.into_iter());
                self.eval(scope)
			},

            ServValue::Func(ServFn::Meta(f)) => {
                todo!();
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
enum Line {
    Statement(Expression),
    Route(String, Expression),
    Definition(String, Expression),
    Equality(Expression, Expression),
}

#[derive(Clone, Debug)]
pub struct ServModule (Vec<Line>);

impl ServModule {
	pub fn compile(input: ast::AstRoot) -> Self {
    	let mut output = Vec::new();
    	for line in input.0 {
			output.push(match line {
    			ast::Line::Statement(e) => Line::Statement(Expression::compile(e)),
    			ast::Line::Equality{ lhs, rhs } if lhs.0.len() == 1 => {
        			match lhs.0[0].clone() {
						ast::Word::Function(name) => Line::Definition(name, Expression::compile(rhs)),
						ast::Word::Route(name) => Line::Route(name, Expression::compile(rhs)),
						otherwise => panic!("invalid"),
        			}
    			},

    			otherwise => todo!(),
			});
    	}

		Self(output)
	}

	pub fn bind_to_scope(&self, scope: &mut Stack) {
		for line in &self.0 {
    		match line {
        		Line::Statement(e) => _ = e.clone().eval(scope),
        		Line::Definition(name, Expression(e, is_meta)) => {
            		let expr = ServFn::Expr(Expression(e.clone(), *is_meta), *is_meta);
            		scope.insert_name(name, ServValue::Func(expr))
        		},
        		otherwise => todo!(),
    		};
		}
	}

	// pub fn eq
}
