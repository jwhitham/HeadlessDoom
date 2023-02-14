
use libc::c_int;

extern {
    fn root() -> c_int;
}

fn main() {
    let x: c_int;
    unsafe {
        x = root();
    }
    println!("hahaha {}", x);
}
