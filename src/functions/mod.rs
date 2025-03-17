use crate::ServValue;
use crate::ServResult;

use crate::Label;
use crate::error::ServError;
use crate::Stack;
use crate::servparser;
use crate::ServModule;

use std::collections::HashMap;

use crate::value::ServFn;
use crate::value::ServList;

use std::io::Read;

mod host;
// mod list;
// mod sql;
// mod request;
// mod math;
mod core;

pub mod json;

pub fn standard_library() -> ServModule {
    let mut output = ServModule::empty();
    output.values.extend(core::get_module().values);
    // output.values.extend(math::get_module().values);
    // output.values.extend(list::get_module().values);
    // output.values.extend(request::get_module().values);
    // output.values.extend(json::get_module().values);
    output.values.extend(host::get_module().values);
    // output.values.extend(sql::get_module().values);

    output

}
