use crate::request_state::RequestState;

pub type CommandFunction = for<'a> fn(RequestState<'a>, &Vec<Arg>) -> RequestState<'a>;

pub struct Command {
    pub name: String,
    pub args: Vec<Arg>,
    pub function: CommandFunction, 
}

impl Command {
    pub fn run<'request>(&self, state: RequestState<'request>) -> RequestState<'request> {
        (self.function)(state, &self.args)
    }
}

pub struct Arg(String);




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
    ($name:ident, ($input:ident, $args:ident) => $func:block) => {
        pub fn $name<'a>(mut $input: RequestState<'a>, $args: &Vec<Arg>) -> RequestState<'a> {
            $func
        }
    };

    ($name:ident, ($input:ident) => $func:block) => {
        pub fn $name<'a>(mut $input: RequestState<'a>, _args: &Vec<Arg>) -> RequestState<'a> {
            $func
        }
    };
}

command_function!(echo, (input, args) => {
    for arg in args {
        input.body.append(&mut "test".as_bytes().to_vec());
    }

    return input
});
