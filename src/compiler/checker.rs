use crate::compiler::ast::*;
use crate::compiler::types::*;
use crate::util::*;
use std::collections::hash_map::HashMap;

struct TypeEnv {
    scope: HashMap<u32, u32>,
}
