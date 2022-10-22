pub mod cmd;
pub mod core;
pub mod files;
pub mod exec;

pub use cmd::Cmd;
pub use self::core::Write;
pub use self::files::ReadFile;
pub use self::exec::Exec;

use std::sync::Arc;

pub fn parse_cmd(name: &str, arg: Option<&str>) -> Result<Arc<dyn Cmd>, ()> {

    match name {
        "write" | "echo" => Ok(Write::with_arg(arg).wrap()),
        "read" => Ok(ReadFile::with_arg(arg).wrap()),
        "exec" => Ok(Exec::with_arg(arg).wrap()),
        _ => Err(()),
    }
}
