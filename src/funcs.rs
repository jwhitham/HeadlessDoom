// Functions (global scope) still defined in C

use crate::defs::*;
extern {
    pub fn R_GetColumn (tex: i32, col: i32) -> *mut u8;
    pub fn Z_Malloc(size: i32, tag: u32, user: *mut *mut u8) -> *mut u8;
    pub fn R_PointOnSegSide(x: fixed_t, y: fixed_t, line: *mut seg_t) -> i32;
    pub fn R_RenderMaskedSegRange(ds: *mut drawseg_t, x1: i32, x2: i32);
    pub fn R_PointToAngle(x: fixed_t, y: fixed_t) -> angle_t;
    pub fn R_PointToDist(x: fixed_t, y: fixed_t) -> fixed_t;
    pub fn R_ScaleFromGlobalAngle(visangle: angle_t) -> fixed_t;
    pub fn R_CheckPlane(pl: *mut visplane_t, start: i32, stop: i32) -> *mut visplane_t;
    pub fn R_FindPlane(height: fixed_t, picnum: i32, lightlevel: i32) -> *mut visplane_t;
    pub fn R_PointOnSide(x: fixed_t, y: fixed_t, node: *mut node_t) -> i32;
    pub fn memset(s: *mut u8, c: i32, n: usize) -> *mut u8;
    pub fn memcpy(d: *mut u8, s: *const u8, n: usize) -> *mut u8;
    pub fn W_GetNumForName (name: *const u8) -> i32;
    pub fn W_CheckNumForName (name: *const u8) -> i32;
    pub fn W_CacheLumpNum (lump: i32, tag: u32) -> *mut u8;
    pub fn W_CacheLumpName (name: *const u8, tag: u32) -> *mut u8;
    pub fn W_LumpLength (lump: i32) -> i32;
    pub fn R_DrawColumnLow();
    pub fn R_DrawSpanLow();
    pub fn R_InitData();
    pub fn R_InitSkyMap();
    pub fn R_InitTranslationTables();
    pub fn NetUpdate();
    pub fn Z_ChangeTag2(ptr: *mut u8, tag: u32);
    pub fn Z_Free(ptr: *mut u8);
}

pub unsafe fn W_Name(name_p: *const i8) -> String {
    let mut name: [i8; 9] = [0; 9];
    memcpy (name.as_mut_ptr() as *mut u8, name_p as *const u8, 8);
    return std::ffi::CStr::from_ptr(name.as_ptr()).to_str().unwrap().to_string();
}
