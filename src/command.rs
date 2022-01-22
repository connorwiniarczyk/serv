use crate::request_state::RequestState;
use std::process;

use lazy_static::lazy_static;
use regex::Regex;
use regex::Captures;

lazy_static! {
    /// defines syntax for variables within an argument.
    /// syntax is based on Makefile variable syntax: ie. $(VAR)
    static ref VAR: Regex = Regex::new(r"\$\((?P<name>.+?)\)").unwrap();
}

pub type CommandFunction = for<'a> fn(&mut RequestState<'a>, &Vec<Arg>);

#[derive(Clone)]
pub struct Command {
    pub name: String,
    pub args: Vec<Arg>,
    pub function: CommandFunction, 
}

impl Command {

    pub fn run<'request>(&self, mut state: RequestState<'request>) -> RequestState<'request> {
        let args: Vec<Arg> = self.args.iter().map(|arg| arg.substitute_variables(&state)).collect();
        (self.function)(&mut state, &args);

        return state;
    }

    pub fn new(name: &str, args: Vec<Arg>) -> Self {
        Self {
            name: name.to_string(),
            args,
            function: get_command_function(name),
        } 
    }

    // pub fn new(name: &str, arg_strings: Vec<&str>) -> Self {
    //     let args = arg_strings.iter().map(|arg| Arg::new(None, arg)).collect();
    //     Self {
    //         name: name.to_string(),
    //         args,
    //         function: get_command_function(name),
    //     } 
    // }
}

macro_rules! command {
    ($name:expr) => {{ Command::new($name, vec![]) }};
    ($name:expr, $( $arg:expr ),+) => {{ Command::new($name, vec![$(Arg::new(None, $arg),)+]) }};
}

pub(crate) use command;

use std::fmt;
impl fmt::Debug for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Point")
            .field("name", &self.name)
            .field("args", &self.args)
            .finish()
    }
}

// TODO macro for easier definition of args inside code
// macro_rules! arg {
//     ( $key:ident=$value:expr ) => { Arg::Named{ key: stringify!($key).to_string(), value: $value.to_string() } };
//     ( $value:expr ) => { Arg::Positional{ value: $value.to_string() } };
// }

#[derive(Clone, Debug)]
pub enum Arg {
    Positional{ value: String },
    Named{ key: String, value: String },
}

impl Arg {
    pub fn new(key: Option<&str>, value: &str) -> Self {
        match key {
            Some(key) => Self::Named { key: key.to_string(), value: value.to_string() },
            None => Self::Positional { value: value.to_string() },
        }
    }

    pub fn value(&self) -> &str {
        match self {
            Self::Named { key, value } => &value,
            Self::Positional { value } => &value,
        }
    }

    pub fn substitute_variables(&self, state: &RequestState) -> Self {
        let new_value = VAR.replace(self.value(), |caps: &Captures|{
            println!("{:?}", caps);
            let var_name = caps.name("name").unwrap().as_str();
            state.get_variable(&var_name)
        });

        match self {
            Self::Positional { value } => Self::new(None, &new_value),
            Self::Named { key, value } => Self::new(Some(key), &new_value),
        }
    }
}



/// Declare a command using a javascript style arrow function syntax.
/// Generates a function pointer that can be used in the `func` field of
/// RouteOptions in the RouteTable. This is mainly useful as a way of
/// abbreviating the very verbose function signature that `RouteOption` 
/// requires.
/// 
/// Takes as arguments the name of the function, followed by an arrow function
/// that performs some operation on a `ResponseGenerator`.
///
/// ### examples
/// ```
/// // Adds CORS headers to the response
/// define_processor!(cors (input) => {
///     input.with_header("Access-Control-Allow-Origin", "*")
/// });
/// ```
macro_rules! command_function {
    ($name:ident, ($state:ident, $args:ident) => $func:block) => {
        pub fn $name<'a>($state: &mut RequestState<'a>, $args: &Vec<Arg>) {
            $func
        }
    };

    ($name:ident, ($state:ident) => $func:block) => {
        pub fn $name<'a>($state: &mut RequestState<'a>, _args: &Vec<Arg>) {
            $func
        }
    };
}

command_function!(set, (state, args) => {
    let key = &args[0];
    let value = &args[1];

    state.variables.insert(key.value().to_string(), value.value().to_string());
});

command_function!(echo, (state, args) => {
    for arg in args {
        state.body.append(&mut arg.value().as_bytes().to_vec());
    }
});

command_function!(exec, (state, args) => {
    println!("\t exec");
    let mut args_iterator = args.iter();
    let path = &args_iterator.next().unwrap().value();
    let executable_arguments:Vec<&str> = args_iterator.map(|arg| arg.value()).collect();

    // println!("\t\t executing command: {:?}", &path);
    // let rendered_args: Vec<String> = args.iter()
    //     .filter_map(|x| response.extract_data(x))
    //     .collect();

    println!("{}", path);
    println!("{:?}", executable_arguments);

    let mut result = process::Command::new(&path).args(executable_arguments).output().unwrap().stdout;
    println!("{:?}", result);
    state.body.append(&mut result);
});

fn get_command_function(name: &str) -> CommandFunction{
    match name {
        "echo" => echo,
        "exec" => exec,
        "set" => set,
        _ => panic!("command_function {} isn't defined", name), 
    }
}
