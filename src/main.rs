
use libc::c_int;
use std::ffi::CString;

extern {
    fn D_DoomMain();
    // int  myargc;
    // char**  myargv;
    static mut myargc: c_int;
    static mut myargv: *mut *const i8;
}

fn main() {
    let r = std::env::set_current_dir("headless_doom");
    assert!(r.is_ok());
    let mut dst: Vec<*const i8> = Vec::with_capacity(3);
    let arg0 = CString::new("arg0").unwrap();
    let test = CString::new("test").unwrap();
    dst.push(arg0.as_ptr());
    dst.push(test.as_ptr());
    unsafe {
        myargc = 2;
        myargv = dst.as_mut_ptr();
        D_DoomMain();
    }
}
