pub mod cmd;
pub mod core;
pub mod files;
pub mod exec;
pub mod sql;

pub use cmd::Cmd;
pub use self::core::Echo;
pub use self::files::{ReadFile, WriteFile};
pub use self::exec::{Exec, Shell};
pub use self::sql::Sql;

use std::sync::Arc;

pub fn parse_cmd(name: &str, arg: Option<&str>) -> Result<Arc<dyn Cmd>, ()> {

    match name {
        "echo" => Ok(Echo::with_arg(arg).wrap()),
        "debug" => Ok(core::Debug::with_arg(arg).wrap()),
        "read" => Ok(ReadFile::with_arg(arg).wrap()),
        "write" => Ok(WriteFile::with_arg(arg).wrap()),
        "exec" => Ok(Exec::with_arg(arg).wrap()),
        "sh" | "shell" => Ok(Shell::with_arg(arg).wrap()),
        "sql" => Ok(Sql::with_arg(arg).wrap()),
        "set" => Ok(core::SetVar::with_arg(arg).wrap()),
        "parse" => Ok(core::ParseBody::with_arg(arg).wrap()),
        "run" => Ok(core::Jump::with_arg(arg).wrap()),
        _ => Err(()),
    }
}
