pub mod cmd;
pub mod core;
pub mod files;

pub use cmd::Cmd;
pub use self::core::Write;
pub use self::files::ReadFile;

use std::sync::Arc;

pub fn parse_cmd(name: &str, arg: Option<&str>) -> Result<Arc<dyn Cmd>, ()> {

    match name {
        "write" => Ok(Write::with_arg(arg).wrap()),
        "read" => Ok(ReadFile::with_arg(arg).wrap()),
        _ => Err(()),
    }
}
