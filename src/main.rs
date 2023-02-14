
use libc::c_int;

extern {
    static mut value: c_int;
    fn hello(x: c_int) -> c_int;
}

#[no_mangle]
pub extern "C" fn world(x: c_int) -> c_int {
    if x > 100 {
        panic!();
    }
    return x + 2;
}

fn main() {
    let mut x: c_int = 1;
    unsafe {
        value = 16;
        x = hello(x);
    }
    println!("Hello, world! {}", x);
    unsafe {
        x = hello(x);
    }
    println!("hahaha {}", x);
}
