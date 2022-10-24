use async_trait::async_trait;
use crate::request_state::RequestState;

use std::fmt::Display;
use std::fmt;

use hyper::Body;

use std::sync::Arc;

use lazy_static::lazy_static;
use regex::{Regex, Captures};

lazy_static! {
	/// defines syntax for variables within an argument.
	/// syntax is based on Makefile variable syntax: ie. $(VAR)
	static ref VAR: Regex = Regex::new(r"(?P<precede>\$?)\$\((?P<name>.+?)\)").unwrap();
}

#[async_trait]
pub trait Cmd: Send + Sync {
    async fn run(&self, state: &mut RequestState); 
    fn with_arg(arg: Option<&str>) -> Self where Self: Sized;
    fn name(&self) -> &str;
    fn arg(&self) -> &str;

    fn wrap(self) -> Arc<dyn Cmd> where Self: Sized + 'static {
        Arc::new(self)
    }

    fn substitute_vars(text: &str, state: &RequestState) -> String where Self: Sized {

		VAR.replace_all(text, |caps: &Captures|{

			// check to see if the variable syntax is prefixed by a second dollar sign
			// ie. $$(var) instead of $(var)
			let is_double = caps.name("precede").unwrap().as_str() == "$";

			match is_double {
				// if so, strip the preceding dollar sign and use the string as is
				true => {
					caps.get(0).unwrap().as_str().strip_prefix("$").unwrap().to_string()
				},

				// otherwise, perform variable substitution
				false => {
					let var_name = caps.name("name").unwrap().as_str();
					state.get_variable(&var_name).unwrap_or("").to_string()
				},
			}
		}).to_string()
    }
}


impl Display for Cmd {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(&self.name())?;
		f.write_str(" ")?;

		let arg = self.arg();
		if arg.chars().count() >= 80 {
			f.write_str(" ... ")?;
		} else {
			f.write_str(&arg.replace("\t", " "))?;
		}

		Ok(())
	}
}
