/// Declare CommandFunctions here

use super::command::CommandFunction;

use crate::request_state::RequestState;
use std::process;
use lazy_static::lazy_static;
use regex::Regex;
use regex::Captures;
use std::fs;
use itertools::Itertools;
use tree_magic;
use std::io::Write;

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
#[macro_export]
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
    state.body.append(args.unwrap_or(""));
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
    stdin.write_all(state.body.data());

    let mut output = child.wait_with_output().expect("failed").stdout;
    state.body.replace(output.as_slice());
});

command_function!(read, (state, arg) => {
    if let Some(paths) = arg {
        for path in paths.split(" ") {
            match fs::read(path) {
                Ok(data) => state.body.append(data.as_slice()),
                Err(error) => state.body = crate::body::Body::Err(error.into()),
            }
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
    let input = arg.unwrap_or("");
    let mut shell_process = process::Command::new("/bin/sh")
        .stdin(process::Stdio::piped())
        .stdout(process::Stdio::piped())
        .spawn()
        .expect("failed to start shell process");

    let mut stdin = shell_process.stdin.as_mut().expect("failed to open stdin");
    stdin.write_all(input.as_bytes()).expect("failed to write");

    let mut output = shell_process.wait_with_output().expect("failed to wait for shell").stdout;

    state.body.append(output.as_slice());

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

pub fn get_command_function(name: &str) -> CommandFunction{
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
