use std::collections::HashMap;
use crate::*;
use std::process::{Command, Stdio};
use std::io::Write;
use std::io::{BufWriter};
use evalexpr::eval;
use std::collections::VecDeque;

pub fn exec(input: ServValue, scope: &Scope) -> ServResult {
    let text = input.to_string();
    let mut args = text.split_whitespace();
    let mut cmd = Command::new(args.next().ok_or("not enough arguments")?);
    for arg in args {
        cmd.arg(arg);
    }

    let result = cmd
        .stdout(Stdio::piped())
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .spawn().map_err(|_| "could not spawn")?.wait_with_output().unwrap();

    Ok(ServValue::Raw(result.stdout))
    // Ok(ServValue::Text(std::str::from_utf8(&result.stdout).unwrap().to_owned()))
}

pub fn exec_pipe(words: &mut Words, input: ServValue, scope: &Scope) -> ServResult {
    let arg_name = words.0.pop_front().unwrap();
    let arg_fn = scope.get(&arg_name).unwrap();
    let arg = arg_fn.call(input.clone(), scope)?;
    let rest = words.eval(input, scope)?;

    let mut cmd = Command::new(arg.to_string())
        .stderr(Stdio::null())
        .stdout(Stdio::piped())
        .stdin(Stdio::piped())
        .spawn()
        .unwrap();

    cmd.stdin.as_mut().unwrap().write_all(rest.to_string().as_bytes());
    let output = cmd.wait_with_output().unwrap();

    Ok(ServValue::Text(std::str::from_utf8(&output.stdout).unwrap().to_owned()))
}


pub fn read_file(input: ServValue, scope: &Scope) -> ServResult {
    let path = input.to_string();
    let contents = std::fs::read_to_string(path).map_err(|e| "failed to open file")?;
    Ok(ServValue::Text(contents))
}

pub fn read_file_raw(input: ServValue, scope: &Scope) -> ServResult {
    let path = input.to_string();
    let contents = std::fs::read(path).map_err(|e| "failed to open file")?;
    Ok(ServValue::Raw(contents))
}

pub fn read_dir(input: ServValue, scope: &Scope) -> ServResult {
    let paths = std::fs::read_dir(input.to_string()).map_err(|_| "invalid path")?;
    let mut output = VecDeque::new();
    for path in paths {
        if let Ok(p) = path { output.push_back(ServValue::Text(p.path().display().to_string())); }
    }
    Ok(ServValue::List(output))
}
