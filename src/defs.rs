include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub type sprtemp_t = [spriteframe_t; 29];
pub type boolean = c_boolean::Type;
pub const c_false: boolean = c_boolean::c_false;
pub const c_true: boolean = c_boolean::c_true;
