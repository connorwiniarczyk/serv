use crate::serv_value::ServValue;
use crate::AstNode;
use crate::Words;
use crate::Context;
use crate::parser;

use std::fmt::Debug;
use std::sync::Arc;
use std::iter::Peekable;

use crate::compiler::Compiler;

pub trait ServFunctionTrait: Send + Sync {
	fn call(&self, input: ServValue, ctx: &mut Context) -> Result<ServValue, &'static str>;
}

#[derive(Clone)]
pub struct ServFunction(Arc<dyn ServFunctionTrait>);

impl ServFunction {
	pub fn new<S: ServFunctionTrait + 'static>(input: S) -> Self {
		Self(Arc::new(input))
	}
}

impl ServFunctionTrait for ServFunction {
	fn call(&self, input: ServValue, ctx: &mut Context) -> Result<ServValue, &'static str> {
		self.0.call(input, ctx)
	}
}


pub struct FnWrapper<A, B>(Box<dyn Fn(A) -> (B) + Send + Sync>);

impl<A, B> FnWrapper<A, B> where
A: TryFrom<ServValue, Error = &'static str> + 'static,
B: Into<ServValue> + 'static,
{
	pub fn new<F: 'static + Send + Sync + Fn(A) -> B>(input: F) -> ServFunction {
		let inner = Self(Box::new(input));
		ServFunction::new(inner)
	}
}

impl<A, B> ServFunctionTrait for FnWrapper<A, B> where
A: TryFrom<ServValue, Error = &'static str>,
B: Into<ServValue>,

{
	fn call(&self, input: ServValue, ctx: &mut Context) -> Result<ServValue, &'static str> {
		let res = (self.0)(input.try_into()?).into();
		Ok(res)
	}
}

impl ServFunctionTrait for String {
	fn call(&self, input: ServValue, ctx: &mut Context) -> Result<ServValue, &'static str> {
		Ok(ServValue::Text(self.clone()))
	}
}

impl ServFunctionTrait for &str {
	fn call(&self, input: ServValue, ctx: &mut Context) -> Result<ServValue, &'static str> {
		Ok(ServValue::Text(self.to_string()))
	}
}

impl ServFunctionTrait for i64 {
	fn call(&self, input: ServValue, ctx: &mut Context) -> Result<ServValue, &'static str> {
		Ok(ServValue::Int(*self))
	}
}

impl ServFunctionTrait for Vec<ServFunction> {
	fn call(&self, input: ServValue, ctx: &mut Context) -> Result<ServValue, &'static str> {
		let result: ServValue = self.iter().rev().fold(input, |acc, next| next.call(acc, ctx).unwrap());
		Ok(result)
	}
}


