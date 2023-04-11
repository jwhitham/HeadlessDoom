#![allow(non_upper_case_globals, non_camel_case_types, non_snake_case, dead_code)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub type boolean = c_boolean::Type;
pub const c_false: boolean = c_boolean::c_false;
pub const c_true: boolean = c_boolean::c_true;
pub const MAXCHAR: i8 = 0x7f;
pub const MAXSHORT: i16 = 0x7fff;
pub const MAXINT: i32 = 0x7fffffff;
pub const MAXLONG: i32 = 0x7fffffff;
pub const MINCHAR: i8 = -0x80;
pub const MINSHORT: i16 = -0x8000;
pub const MININT: i32 = -0x80000000;
pub const MINLONG: i32 = -0x80000000;
