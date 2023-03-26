
use libc::c_int;
use std::ffi::CString;

extern {
    fn D_DoomMain();
    static mut myargc: c_int;
    static mut myargv: *mut *const i8;
}



fn main() {
    let r = std::env::set_current_dir("headless_doom");
    assert!(r.is_ok());

    let mut cargs: Vec<CString> = Vec::new();
    for arg in std::env::args_os() {
        cargs.push(CString::new(arg.into_string().unwrap()).unwrap());
    }

    let mut dst: Vec<*const i8> = Vec::new();
    for carg in &cargs {
        dst.push(carg.as_ptr());
    }
    unsafe {
        myargc = dst.len() as i32;
        myargv = dst.as_mut_ptr();
        D_DoomMain();
    }
}
