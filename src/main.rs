
//use libc::c_int;

extern {
    fn D_DoomMain();
}

fn main() {
    std::env::set_current_dir("headless_doom");
    unsafe {
        D_DoomMain();
    }
}
