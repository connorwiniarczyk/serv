/// Declare CommandFunctions here
use super::command::CommandFunction;

use crate::request_state::RequestState;
use std::process;
use std::fs;
use std::io::Write;
use crate::body;

use sqlite;
use json;



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

fn json_encode_object(object: Vec<(String, String)>) -> String {
    let mut output = String::new();
    output.push('{');

    let mut iter = object.into_iter().peekable();

    loop {
        match iter.next() {
            Some((key, value)) => {
                output.push_str(&format!("\"{}\": \"{}\"", key, value));
                if let Some(_) = iter.peek() { output.push(','); }
            },
            None => break,
        }
    }
    output.push('}');
    output
}

fn json_encode_array(array: Vec<String>) -> String {
    let mut output = String::new();
    output.push('[');

    let mut iter = array.into_iter().peekable();

    while let Some(value) = iter.next() {
        output.push_str(&value);

        if let Some(_) = iter.peek() {
            output.push_str(",\n");
        }
    }
    output.push(']');
    output
}

struct SqlParams<'request> {
    state: &'request RequestState<'request>,
    index: u32,
}

impl<'request> SqlParams<'request> {
    fn new(state: &'request RequestState<'request>) -> Self {
        Self { state, index: 0 }
    }
}

impl<'request> Iterator for SqlParams<'request> {
    type Item = &'request str;

    fn next(&mut self) -> Option<Self::Item> {
        self.index += 1;
        let name = format!("sql.params.{}", self.index);
        self.state.get_variable(&name)
    }
}


command_function!(sql, (state, args) => {
    let db_name = "local.sqlite";
    let connection = sqlite::open(&db_name).unwrap();

    // create the sql query and bind parameters
    let mut query = connection.prepare(&args.unwrap()).unwrap();
    for (index, param) in SqlParams::new(&state).enumerate() {
        query = query.bind(index + 1, param).expect("failed to bind sql parameter");
    }

    let mut output: Vec<String> = Vec::new();

    while let sqlite::State::Row = query.next().unwrap() {
        let mut row: Vec<(String, String)> = Vec::new();
        for (index, column) in query.column_names().iter().enumerate() {
            row.push((column.to_string(), query.read(index).unwrap_or(String::new())));
        }
        output.push(json_encode_object(row));
    }
    state.body.append(json_encode_array(output).as_str());
});

command_function!(jumpto, (state, args) => {
    let route_name = args.expect("need to specifiy a route to jump to");
    let route = state.table.get(route_name).expect("that route does not exist");
    
    // for command in &route.commands {
    //     command.run(state);
    // }
});


command_function!(set, (state, args) => {
    let mut args_iter = args.unwrap().split(" ");
    let key = &args_iter.next().unwrap();
    let value = &args_iter.next().unwrap();

    state.variables.insert(key.to_string(), value.to_string());
});

command_function!(echo, (state, args) => {
    state.body = body::Body::from_str(args.unwrap_or(""));
    // state.body.append(args.unwrap_or(""));
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

use json::JsonValue;

// parse the body 
command_function!(parse_body, (state, args) => {
    let json = json::parse(std::str::from_utf8(state.body.data()).unwrap()).unwrap();       

    fn insert_value(parent_prefix: &str, value: JsonValue, state: &mut RequestState) {
        match value {
            JsonValue::Short(value) => state.variables.insert(format!("{}", parent_prefix), value.as_str().to_string()),
            JsonValue::String(value) => state.variables.insert(format!("{}", parent_prefix), value.as_str().to_string()),
            JsonValue::Number(value) => {
                let number: f32 = value.clone().into();
                state.variables.insert(format!("{}", parent_prefix), number.to_string())
            },
            JsonValue::Object(ref _object) => {
                for (key, value) in value.entries() {
                    insert_value(&format!("{}.{}", parent_prefix, key), value.clone(), state);
                }
                None
            },

            JsonValue::Array(ref _array) => {
                for (key, value) in value.members().enumerate() {
                    insert_value(&format!("{}.{}", parent_prefix, key), value.clone(), state);
                }
                None
            },

            _ => todo!(),
        };
    }

    insert_value("body", json, state);
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
        "jumpto" | "jump" | "use" => jumpto,
        "parse_query" => parse_query,
        "sql" => sql,
        "parse_body" => parse_body,
        _ => panic!("command_function {} isn't defined", name), 
    }
}
