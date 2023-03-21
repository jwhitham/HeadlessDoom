
use libc::c_int;
use std::ffi::CString;

type fixed_t = u32;

const FRACBITS: i32 = 16;
const SCREENWIDTH: usize = 320;

extern {
    fn D_DoomMain();
    // int  myargc;
    // char**  myargv;
    static mut myargc: c_int;
    static mut myargv: *mut *const i8;


    static dc_colormap: *const i8;
    static dc_x: c_int; 
    static dc_yl: c_int; 
    static dc_yh: c_int; 
    static dc_iscale: c_int; 
    static dc_texturemid: c_int;

    static dc_source: *const u8;

    static dccount: c_int;
    static ylookup: [*mut i8; SCREENWIDTH];
    static columnofs: [c_int; SCREENWIDTH];

    static centery: c_int; 
}


//
// A column is a vertical slice/span from a wall texture that,
//  given the DOOM style restrictions on the view orientation,
//  will always have constant z depth.
// Thus a special case loop for very fast rendering can
//  be used. It has also been used with Wolfenstein 3D.
//
#[no_mangle]
pub extern "C" fn R_DrawColumn () { 
    /* int   count; 
    byte*  dest; 
    fixed_t  frac;
    fixed_t  fracstep;   */
 
    unsafe {
        let mut count = dc_yh - dc_yl; 

        // Zero length, column does not exceed a pixel.
        if count < 0 {
            return; 
        }
         
        // Framebuffer destination address.
        // Use ylookup LUT to avoid multiply with ScreenWidth.
        // Use columnofs LUT for subwindows? 
        let mut dest: *mut i8 = ylookup[dc_yl as usize].offset(columnofs[dc_x as usize] as isize); 

        // Determine scaling,
        //  which is the only mapping to be done.
        let fracstep = dc_iscale; 
        let mut frac = dc_texturemid + (dc_yl-centery)*fracstep; 

        // Inner loop that does the actual texture mapping,
        //  e.g. a DDA-lile scaling.
        // This is as fast as it gets.
        loop {
            // Re-map color indices from wall texture column
            //  using a lighting/special effects LUT.
            //*dest = dc_colormap[dc_source[((frac>>FRACBITS)&127) as usize] as usize];
            *dest = *dc_colormap.offset(
                        *dc_source.offset(((frac>>FRACBITS)&127) as isize)
                            as isize);

            dest = dest.offset(SCREENWIDTH as isize); 
            frac += fracstep;
            if count == 0 {
                break;
            }
            count -= 1;
        }
    }
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
