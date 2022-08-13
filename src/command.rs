use crate::request_state::RequestState;
use std::process;

use lazy_static::lazy_static;
use regex::Regex;
use regex::Captures;

use std::fs;

use itertools::Itertools;

use tree_magic;
use std::io::Write;

lazy_static! {
    /// defines syntax for variables within an argument.
    /// syntax is based on Makefile variable syntax: ie. $(VAR)
    static ref VAR: Regex = Regex::new(r"\$\((?P<name>.+?)\)").unwrap();
}

pub type CommandFunction = for<'a> fn(&mut RequestState<'a>, Option<&str>);

#[derive(Clone)]
pub struct Command {
    pub name: String,
    pub arg: Option<String>,
    pub function: CommandFunction, 
}

impl Command {

    pub fn substitute_variables(&self, state: &RequestState) -> Option<String> {
        
        let original_value = self.arg.clone()?;

        let new_value = VAR.replace_all(&original_value, |caps: &Captures|{
            let var_name = caps.name("name").unwrap().as_str();
            state.get_variable(&var_name)
        });

        Some(new_value.to_string())
    }

    pub fn run<'request>(&self, state: &mut RequestState<'request>){
        // (self.function)(state, &self.substitute_variables(&state));
        (self.function)(state, None);
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
        f.write_str(&self.arg.clone().unwrap_or(String::new()))?;

        Ok(())
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
        pub fn $name<'a>($state: &mut RequestState<'a>, $args: Option<&str>) {
            $func
        }
    };

    ($name:ident, ($state:ident) => $func:block) => {
        pub fn $name<'a>($state: &mut RequestState<'a>, _args: Option<&str>) {
            $func
        }
    };
}

command_function!(set, (state, args) => {
    let mut args_iter = args.unwrap().split(" ");
    let key = &args_iter.next().unwrap();
    let value = &args_iter.next().unwrap();

    state.variables.insert(key.to_string(), value.to_string());
});

command_function!(echo, (state, args) => {
    state.body.append(&mut args.unwrap_or("").as_bytes().to_vec());
});

command_function!(exec, (state, args) => {

    let mut args_iter = args.unwrap().split(" ");
    let path = &args_iter.next().unwrap();
    let exec_args: Vec<&str> = args_iter.collect();

    let mut child = process::Command::new(&path)
        .args(exec_args)
        .stdin(process::Stdio::piped())
        .stdout(process::Stdio::piped())
        .spawn()
        .expect("failed to spawn child");

    let mut stdin = child.stdin.as_mut().expect("failed to open stdin");
    stdin.write_all(&state.body);

    let mut output = child.wait_with_output().expect("failed").stdout;
    state.body.append(&mut output);

});

command_function!(read, (state, arg) => {
    if let Some(paths) = arg {
        for path in paths.split(" ") {
            let mut data = fs::read(path).unwrap();
            state.body.append(&mut data);
        }
    }

});

command_function!(header, (state, args) => {
    let mut args_iter = args.unwrap().split(" ");    

    let key = args_iter.next().unwrap().to_string();
    let value = args_iter.next().unwrap().to_string();

    state.headers.insert(key, value);
});

command_function!(filetype, (state, arg) => {
    state.mime = arg.map(|x| x.to_string());
    // state.headers.insert("content-type".to_string(), value);
});

/// Joins all of the arguments into a single string and pipes it into /bin/sh, and appends the
/// result to the response body. Lets the user easily run arbitrary shell scripts with complicated
/// behaviors like pipes
command_function!(shell, (state, arg) => {
    // let input = args.iter().map(|arg| arg.value()).join(" ");

    let input = arg.unwrap_or("");

    let mut shell_process = process::Command::new("/bin/sh")
        .stdin(process::Stdio::piped())
        .stdout(process::Stdio::piped())
        .spawn()
        .expect("failed to start shell process");

    let mut stdin = shell_process.stdin.as_mut().expect("failed to open stdin");
    stdin.write_all(input.as_bytes()).expect("failed to write");

    let mut output = shell_process.wait_with_output().expect("failed to wait for shell").stdout;

    state.body.append(&mut output);

});


/// Prints the entirety of the current state to stdout
command_function!(debug, (state) => {
    println!("{:#?}", state);
});


command_function!(parse_query, (state, args) =>{

    // let mut input = state.request.url().query_pairs();
    // let mut output = String::new();

    // output.push_str("{");

    // match input.next() {
    //     Some((key, value)) => output.push_str(&format!("\"{}\": \"{}\"", key, value).as_str()),
    //     _ => return,
    // };

    // for (key, value) in input {
    //     output.push_str(&format!(", \"{}\": \"{}\"", key, value).as_str());
    // }

    // output.push_str("}");

    // // write output to variable
    // let key = &args[0];
    // state.variables.insert(key.value().to_string(), output);
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
        "debug" => debug,
        "parse_query" => parse_query,
        _ => panic!("command_function {} isn't defined", name), 
    }
}
