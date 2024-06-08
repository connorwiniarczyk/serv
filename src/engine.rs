use crate::template::Template;
use crate::Context;
use crate::value::ServValue;
use crate::dictionary::Scope;
use std::sync::Arc;


pub trait ServFn: Send + Sync {
	fn call(&self, input: ServValue, ctx: &Context) -> Result<ServValue, &'static str>;
}

#[derive(Clone)]
pub struct ServFunction(Arc<dyn ServFn>);

impl ServFunction {
	pub fn call(&self, input: ServValue, ctx: &Context) -> Result<ServValue, &'static str> {
    	self.0.call(input, ctx)
	}
}



pub struct Word(String);

impl ServFn for Word {
	fn call(&self, input: ServValue, ctx: &Context) -> Result<ServValue, &'static str> {
    	match self.0.as_str() {
        	"hello" => Ok(ServValue::Text("Hello World!".to_owned())),
        	_ => todo!(),
    	}
	}
}

// pub struct Context<'a>(Scope<'a, String, Vec<ServFunction>>);
