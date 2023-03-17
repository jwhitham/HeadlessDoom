
use libc::c_int;
use std::ffi::CString;

type fixed_t = u32;

extern {
    fn D_DoomMain();
    // int  myargc;
    // char**  myargv;
    static mut myargc: c_int;
    static mut myargv: *mut *const i8;


    static dc_colormap: *const u8; 
    static dc_x: c_int; 
    static dc_yl: c_int; 
    static dc_yh: c_int; 
    static dc_iscale: c_int; 
    static dc_texturemid: c_int;

    static dc_source: *const u8;  

    static dccount: c_int;
    static ylookup: *const *const i8;
    static columnofs: *const c_int;

    static centery: c_int; 
}


const FRACBITS: i32 = 16;
const SCREENWIDTH: i32 = 320;

//
// A column is a vertical slice/span from a wall texture that,
//  given the DOOM style restrictions on the view orientation,
//  will always have constant z depth.
// Thus a special case loop for very fast rendering can
//  be used. It has also been used with Wolfenstein 3D.
// 
extern "C" fn R_DrawColumn () { 
    /* int   count; 
    byte*  dest; 
    fixed_t  frac;
    fixed_t  fracstep;   */
 
    let count = dc_yh - dc_yl; 

    // Zero length, column does not exceed a pixel.
    if count < 0 {
        return; 
    }
     
    // Framebuffer destination address.
    // Use ylookup LUT to avoid multiply with ScreenWidth.
    // Use columnofs LUT for subwindows? 
    let mut dest = (ylookup[dc_yl] as c_int) + columnofs[dc_x];  

    // Determine scaling,
    //  which is the only mapping to be done.
    let mut fracstep = dc_iscale; 
    let frac = dc_texturemid + (dc_yl-centery)*fracstep; 

    // Inner loop that does the actual texture mapping,
    //  e.g. a DDA-lile scaling.
    // This is as fast as it gets.
    loop {
        // Re-map color indices from wall texture column
        //  using a lighting/special effects LUT.
        *dest = dc_colormap[dc_source[(frac>>FRACBITS)&127]];

        dest += SCREENWIDTH; 
        frac += fracstep;
        if count == 0 {
            break;
        }
        count -= 1;
    }
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
