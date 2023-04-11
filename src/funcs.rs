
use crate::defs::*;
extern {
    pub fn FixedMul (a: fixed_t, b: fixed_t) -> fixed_t;
    pub fn R_GetColumn (tex: i32, col: i32) -> *mut u8;
    pub fn Z_Malloc(size: i32, tag: i32, user: *const u8) -> *mut u8;
}
