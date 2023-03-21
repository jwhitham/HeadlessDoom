
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

//
// R_DrawSpan 
// With DOOM style restrictions on view orientation,
//  the floors and ceilings consist of horizontal slices
//  or spans with constant z depth.
// However, rotation around the world z axis is possible,
//  thus this mapping, while simpler and faster than
//  perspective correct texture mapping, has to traverse
//  the texture at an angle in all but a few cases.
// In consequence, flats are not stored by column (like walls),
//  and the inner loop has to step in texture space u and v.
//
extern {
    static ds_y: c_int; 
    static ds_x1: c_int; 
    static ds_x2: c_int;

    static ds_colormap: *const i8; 

    static ds_xfrac: c_int; 
    static ds_yfrac: c_int; 
    static ds_xstep: c_int; 
    static ds_ystep: c_int;

    // start of a 64*64 tile image 
    static ds_source: *const u8;	
}

//
// Draws the actual span.
#[no_mangle]
pub extern "C" fn R_DrawSpan () { 
   
    unsafe {
        let mut xfrac = ds_xfrac;
        let mut yfrac = ds_yfrac;
         
        let mut dest: *mut i8 = ylookup[ds_y as usize].offset(columnofs[ds_x1 as usize] as isize);

        // We do not check for zero spans here?
        let mut count = ds_x2 - ds_x1; 

        loop {
            // Current texture index in u,v.
            let spot = ((yfrac>>(16-6))&(63*64)) + ((xfrac>>16)&63);

            // Lookup pixel from flat texture tile,
            //  re-index using light/colormap.
            *dest = *ds_colormap.offset(*ds_source.offset(spot as isize) as isize);
            dest = dest.offset(1);

            // Next step in u,v.
            xfrac += ds_xstep;
            yfrac += ds_ystep;
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
