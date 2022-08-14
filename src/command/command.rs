use crate::request_state::RequestState;
use std::process;
use lazy_static::lazy_static;
use regex::Regex;
use regex::Captures;
use std::fs;
use itertools::Itertools;
use tree_magic;
use super::get_command_function;
use std::io::Write;

lazy_static! {
    /// defines syntax for variables within an argument.
    /// syntax is based on Makefile variable syntax: ie. $(VAR)
    static ref VAR: Regex = Regex::new(r"(?P<precede>\$?)\$\((?P<name>.+?)\)").unwrap();
}

pub type CommandFunction = for<'a> fn(&mut RequestState<'a>, Option<&str>);

#[derive(Clone)]
pub struct Command {
    pub name: String,
    pub arg: Option<String>,
    pub function: CommandFunction, 
}

impl Command {

    pub fn substitute_variables(&self, state: &RequestState) -> Option<String>{

        let new_value = VAR.replace_all(&self.arg.as_deref()?, |caps: &Captures|{

            // check to see if the variable syntax is prefixed by a second dollar sign
            // ie. $$(var) instead of $(var)
            let is_double = caps.name("precede").unwrap().as_str() == "$";

            match is_double {
                // if so, strip the preceding dollar sign and use the string as is
                true => {
                    caps.get(0)
                        .unwrap()
                        .as_str()
                        .strip_prefix("$").unwrap().to_string()
                },

                // otherwise, perform variable substitution
                false => {
                    let var_name = caps.name("name").unwrap().as_str();
                    state.get_variable(&var_name).to_string()
                },
            }
        });

        Some(new_value.into_owned())
    }

    pub fn run<'request>(&self, state: &mut RequestState<'request>){
        (self.function)(state, self.substitute_variables(&state).as_deref());
    }

    pub fn new(name: &str, arg: Option<&str>) -> Self {
        Self {
            name: name.to_string(),
            arg: arg.map(|arg| arg.to_string()),
            function: get_command_function(name),
        } 
    }
}

use std::fmt;
impl fmt::Debug for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Command")
            .field("name", &self.name)
            .field("args", &self.arg)
            .finish()
    }
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name)?;
        f.write_str(" ")?;
        f.write_str(&self.arg.clone().unwrap_or(String::new()))?;

        Ok(())
    }
}


