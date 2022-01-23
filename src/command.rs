use crate::request_state::RequestState;
use std::process;

use lazy_static::lazy_static;
use regex::Regex;
use regex::Captures;

use std::fs;

use itertools::Itertools;

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

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name);
        for arg in &self.args {
            f.write_str(" ");
            f.write_str(arg.value());
        }

        Ok(())
    }
}


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
    let mut args_iterator = args.iter();
    let path = &args_iterator.next().unwrap().value();
    let executable_arguments:Vec<&str> = args_iterator.map(|arg| arg.value()).collect();

    let mut result = process::Command::new(&path).args(executable_arguments).output().unwrap().stdout;
    state.body.append(&mut result);
});

command_function!(read, (state, args) => {
    for arg in args {
        let mut data = fs::read(arg.value()).unwrap();
        state.body.append(&mut data);
    }
});

command_function!(header, (state, args) => {
    let mut args_iter = args.iter();    
    let key = args_iter.next().unwrap().value().to_string();
    let value = args_iter.next().unwrap().value().to_string();
    state.headers.insert(key, value);
});

command_function!(filetype, (state, args) => {
    let value = args.iter().next().unwrap().value().to_string();
    state.headers.insert("content-type".to_string(), value);
});

/// Joins all of the arguments into a single string and pipes it into /bin/sh, and appends the
/// result to the response body. Lets the user easily run arbitrary shell scripts with complicated
/// behaviors like pipes
command_function!(shell, (state, args) => {
    let input = args.iter().map(|arg| arg.value()).join(" ");
    let input_stream = process::Command::new("echo")
        .arg(input)
        .stdout(process::Stdio::piped())
        .spawn()
        .expect("failed to start echo process");


    let mut shell_process = process::Command::new("/bin/sh")
        .stdin(process::Stdio::from(input_stream.stdout.unwrap()))
        .stdout(process::Stdio::piped())
        .spawn()
        .expect("failed to start shell process");

    let mut output = shell_process.wait_with_output().expect("failed to wait for shell").stdout;

    state.body.append(&mut output);

});

fn get_command_function(name: &str) -> CommandFunction{
    match name {
        "echo" => echo,
        "exec" => exec,
        "set" => set,
        "read" | "file" => read,
        "header" => header,
        "filetype" | "ft" | "type" => filetype,
        "shell" | "sh" => shell,
        _ => panic!("command_function {} isn't defined", name), 
    }
}
