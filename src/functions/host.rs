use std::collections::HashMap;
use crate::*;
use std::process::{Command, Stdio};
use std::io::Write;
use std::io::{BufWriter};
use std::collections::VecDeque;

fn exec(input: ServValue, scope: &Stack) -> ServResult {
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

fn exec_pipe(arg: ServValue, input: ServValue, scope: &Stack) -> ServResult {
    let cmd = arg.call(None, scope)?.to_string();
    let mut args = cmd.split_whitespace();
    let mut command = Command::new(args.next().ok_or("bad input")?);
    for arg in args {
        command.arg(arg);
    }

    let mut command = Command::new(arg.to_string())
        .stderr(Stdio::null())
        .stdout(Stdio::piped())
        .stdin(Stdio::piped())
        .spawn()
        .unwrap();

    command.stdin.as_mut().unwrap().write_all(input.to_string().as_bytes());
    let output = command.wait_with_output().unwrap();

    Ok(ServValue::Text(std::str::from_utf8(&output.stdout).unwrap().to_owned()))
}


fn read_file(input: ServValue, scope: &Stack) -> ServResult {
    let path = input.to_string();
    let contents = std::fs::read_to_string(path).map_err(|e| "failed to open file")?;
    Ok(ServValue::Text(contents))
}

fn read_file_raw(input: ServValue, scope: &Stack) -> ServResult {
    let path = input.to_string();
    let contents = std::fs::read(path).map_err(|e| "failed to open file")?;
    Ok(ServValue::Raw(contents))
}

fn read_dir(input: ServValue, scope: &Stack) -> ServResult {
    let paths = std::fs::read_dir(input.to_string()).map_err(|_| "invalid path")?;
    let mut output = VecDeque::new();
    for path in paths {
        if let Ok(p) = path { output.push_back(ServValue::Text(p.path().display().to_string())); }
    }
    Ok(ServValue::List(output))
}

pub fn bind(scope: &mut crate::Stack) {
	scope.insert(Label::name("read"),            ServValue::Func(ServFn::Core(read_file)));
	scope.insert(Label::name("read.raw"),        ServValue::Func(ServFn::Core(read_file_raw)));
	scope.insert(Label::name("file.utf8"),       ServValue::Func(ServFn::Core(read_file)));
	scope.insert(Label::name("file.raw"),        ServValue::Func(ServFn::Core(read_file_raw)));
	scope.insert(Label::name("file"),            ServValue::Func(ServFn::Core(read_file_raw)));
	scope.insert(Label::name("ls"),              ServValue::Func(ServFn::Core(read_dir)));
	scope.insert(Label::name("exec"),            ServValue::Func(ServFn::Core(exec)));
	scope.insert(Label::name("exec.pipe"),            ServValue::Func(ServFn::ArgFn(exec_pipe)));
	scope.insert(Label::name("pipe"),            ServValue::Func(ServFn::ArgFn(exec_pipe)));

}
