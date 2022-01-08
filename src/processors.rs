// Use this module to define processors. Processors are functions that transform the server's
// response in some way.

//use crate::options::*;
//pub type Processor = for<'a> fn(ResponseGenerator<'a>, &Vec<Arg>) -> ResponseGenerator<'a>;


///// Declare an option using a javascript style arrow function syntax.
///// Generates a function pointer that can be used in the `func` field of
///// RouteOptions in the RouteTable. This is mainly useful as a way of
///// abbreviating the very verbose function signature that `RouteOption` 
///// requires.
///// 
///// Takes as arguments the name of the function, followed by an arrow function
///// that performs some operation on a `ResponseGenerator`.
/////
///// ### examples
///// ```
///// // Adds CORS headers to the response
///// define_processor!(cors (input) => {
/////     input.with_header("Access-Control-Allow-Origin", "*")
///// });
///// ```
//macro_rules! define_processor {
//    ($name:ident, ($input:ident, $args:ident) => $func:block) => {
//        pub fn $name<'a>(mut $input: ResponseGenerator<'a>, $args: &Vec<Arg>) -> ResponseGenerator<'a> {
//            $func
//        }
//    };

//    ($name:ident, ($input:ident) => $func:block) => {
//        pub fn $name<'a>(mut $input: ResponseGenerator<'a>, _args: &Vec<Arg>) -> ResponseGenerator<'a> {
//            $func
//        }
//    };
//}


///// Assign names to every processor so that the they can be built from strings
//pub fn get(input: &str) -> Processor {
//    match input {
//        "exec" => exec,
//        "read" => read,
//        "header" => header,
//        "cors" => cors,
//        "filetype" | "ft" | "file" => filetype,
//        _ => panic!("there is no processor defined with the name: {}", input),
//    }
//}

//// ---------------------
//// PROCESSOR DEFINITIONS
//// ---------------------

//use std::process::Command;
//use std::fs;
//use crate::parser::route_parser as parse;

//// Replace the body of the response with the content of the file at the specified resource
//define_processor!(read, (input, args) => {
//    println!("\t READ");

//    // if an argument was given, use it as the path instead of the resource
//    let path = match args.iter().next() {
//        Some(Arg { name, value }) => parse::resource(name).unwrap().get_path(input.request_match),
//        None => input.route.resource.get_path(input.request_match),
//    };
//    println!("\t\t reading from path: {:?}", &path);
//    input.append_to_body(fs::read(&path).unwrap_or_default()) 
//});

//// Replace the body of the response with the result of executing the file at the specified resource
//define_processor!(exec, (response, args) => {
//    println!("\t EXEC");
//    let path = response.route.resource.get_path(response.request_match);

//    println!("\t\t executing command: {:?}", &path);
//    let rendered_args: Vec<String> = args.iter()
//        .filter_map(|x| response.extract_data(x))
//        .collect();

//    let result = Command::new(&path).args(rendered_args).output().unwrap().stdout;
//    response.append_to_body(result)
//});


//// Adds one or more http headers to the http response
//define_processor!(header, (input, args) => {
//    println!("\t HEADER");
//    args.into_iter().fold(input, |response, arg| match arg {
//        Arg { name, value: Some(value) } => response.with_header(name, value),
//        Arg { name, value: None } => response,
//    })     
//});

//// Adds CORS related headers to the response
//define_processor!(cors, (input) => {
//    println!("\t CORS");
//    input.with_header("Access-Control-Allow-Origin", "*")
//});

//// Shorthand for header(content-type:<filetype>)  but with built in abbreviations for common
//// filetypes like js, css, and html.
//define_processor!(filetype, (input, args) => {
//    println!("\t FILETYPE");
//    let arg = args.iter().next().expect("filetype needs at least one argument").name.to_lowercase();
//    let header = match arg.as_str() {
//        "html" => "text/html",
//        "js" | "javascript" => "application/javascript",
//        "css" | "stylesheet" | "style" => "text/css",
//        "jpg" => "image/jpg",
//        "png" => "image/png",
//        "svg" => "image/svg",
//        other => other,
//    };

//    input.with_header("content-type", header)
//});

