#![allow(non_upper_case_globals, non_camel_case_types, non_snake_case, dead_code)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub type boolean = c_boolean::Type;
pub const c_false: boolean = c_boolean::c_false;
pub const c_true: boolean = c_boolean::c_true;
