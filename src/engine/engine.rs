use super::*;

use std::collections::HashMap;

pub struct Engine <'rt> {
    global_vars: &'rt HashMap<String, Value>,
    local_vars: HashMap<String, Value>,
    functions: &'rt HashMap<String, ServFunction>,
}
