// Functions (global scope) still defined in C

extern {
    pub fn Z_Malloc(size: i32, tag: u32, user: *mut *mut u8) -> *mut u8;
    pub fn memset(s: *mut u8, c: i32, n: usize) -> *mut u8;
    pub fn memcpy(d: *mut u8, s: *const u8, n: usize) -> *mut u8;
    pub fn _strnicmp(a: *const u8, b: *const u8, n: usize) -> i32;
    pub fn NetUpdate();
    pub fn Z_ChangeTag2(ptr: *mut u8, tag: u32);
    pub fn Z_Free(ptr: *mut u8);
}

pub unsafe fn W_Str_C2R(s: *const u8) -> String {
    return std::ffi::CStr::from_ptr(s as *const i8).to_str().unwrap().to_string();
}

pub unsafe fn W_Name(name_p: *const u8) -> String {
    let mut name: [u8; 9] = [0; 9];
    memcpy (name.as_mut_ptr() as *mut u8, name_p as *const u8, 8);
    return W_Str_C2R(name.as_ptr());
}
