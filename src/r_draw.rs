// Emacs style mode select   -*- C++ -*- 
//-----------------------------------------------------------------------------
//
// $Id:$
//
// Copyright (C) 1993-1996 by id Software, Inc.
//
// This source is available for distribution and/or modification
// only under the terms of the DOOM Source Code License as
// published by id Software. All rights reserved.
//
// The source is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// FITNESS FOR A PARTICULAR PURPOSE. See the DOOM Source Code License
// for more details.
//
// $Log:$
//
// DESCRIPTION:
//	The actual span/column drawing functions.
//	Here find the main potential for optimization,
//	 e.g. inline assembly, different algorithms.
//
//-----------------------------------------------------------------------------


// really these are elsewhere
use crate::defs::*;
use crate::globals::*;
use crate::funcs::*;
use crate::r_data::colormaps;
use crate::r_main::centery;

pub struct R_DrawColumn_params_t {
    pub dc_texturemid: fixed_t,
    pub dc_yl: i32,
    pub dc_yh: i32,
    pub dc_x: i32,
    pub dc_colormap: *const u8,
    pub dc_source: *mut u8,
    pub dc_iscale: fixed_t,
    pub dc_translation: *const u8,
}

pub const empty_R_DrawColumn_params: R_DrawColumn_params_t = R_DrawColumn_params_t {
    dc_texturemid: 0,
    dc_yl: 0,
    dc_yh: 0,
    dc_x: 0,
    dc_colormap: std::ptr::null(),
    dc_source: std::ptr::null_mut(),
    dc_iscale: 0,
    dc_translation: std::ptr::null(),
};

pub struct R_DrawSpan_params_t {
    pub ds_y: i32,
    pub ds_x1: i32,
    pub ds_x2: i32,
    pub ds_colormap: *const u8,
    pub ds_xfrac: fixed_t,
    pub ds_yfrac: fixed_t,
    pub ds_xstep: fixed_t,
    pub ds_ystep: fixed_t,
    pub ds_source: *mut u8,
}

pub const empty_R_DrawSpan_params: R_DrawSpan_params_t = R_DrawSpan_params_t {
    ds_y: 0,
    ds_x1: 0,
    ds_x2: 0,
    ds_colormap: std::ptr::null(),
    ds_xfrac: 0,
    ds_yfrac: 0,
    ds_xstep: 0,
    ds_ystep: 0,
    ds_source: std::ptr::null_mut(),
};

pub static mut translationtables: *mut u8 = std::ptr::null_mut();
static mut columnofs: [i32; SCREENWIDTH as usize] = [0; SCREENWIDTH as usize];
static mut ylookup: [*mut u8; SCREENWIDTH as usize] = [std::ptr::null_mut(); SCREENWIDTH as usize];


// status bar height at bottom of screen
const SBARHEIGHT: i32 = 32;

//
// All drawing to the view buffer is accomplished in this file.
// The other refresh files only know about ccordinates,
//  not the architecture of the frame buffer.
// Conveniently, the frame buffer is a linear one,
//  and we need only the base address,
//  and the total size == width*height*depth/8.,
//



//
// A column is a vertical slice/span from a wall texture that,
//  given the DOOM style restrictions on the view orientation,
//  will always have constant z depth.
// Thus a special case loop for very fast rendering can
//  be used. It has also been used with Wolfenstein 3D.
//
pub fn R_DrawColumn (dc: &mut R_DrawColumn_params_t) { 
    unsafe {
        let count = dc.dc_yh - dc.dc_yl; 

        // Zero length, column does not exceed a pixel.
        if count < 0 {
            return; 
        }
         
        // Framebuffer destination address.
        // Use ylookup LUT to avoid multiply with ScreenWidth.
        // Use columnofs LUT for subwindows? 
        let mut dest: *mut u8 = ylookup[dc.dc_yl as usize].offset(columnofs[dc.dc_x as usize] as isize); 

        // Determine scaling,
        //  which is the only mapping to be done.
        let fracstep: fixed_t = dc.dc_iscale; 
        let mut frac: fixed_t =
                dc.dc_texturemid.wrapping_add(
                    fracstep.wrapping_mul((dc.dc_yl - centery) as fixed_t));

        // Inner loop that does the actual texture mapping,
        //  e.g. a DDA-lile scaling.
        // This is as fast as it gets.
        for _ in 0 ..= count {
            // Re-map color indices from wall texture column
            //  using a lighting/special effects LUT.
            //*dest = dc.dc_colormap[dc.dc_source[((frac>>FRACBITS)&127) as usize] as usize];
            *dest = *dc.dc_colormap.offset(
                        *dc.dc_source.offset(((frac>>FRACBITS)&127) as isize)
                            as isize);

            dest = dest.offset(SCREENWIDTH as isize); 
            frac = frac.wrapping_add(fracstep);
        }
    }
} 






//
// Spectre/Invisibility.
//
const FUZZTABLE: usize = 50;
const FUZZOFF: isize = SCREENWIDTH as isize;


const fuzzoffset: [isize; FUZZTABLE] = [
    FUZZOFF,-FUZZOFF,FUZZOFF,-FUZZOFF,FUZZOFF,FUZZOFF,-FUZZOFF,
    FUZZOFF,FUZZOFF,-FUZZOFF,FUZZOFF,FUZZOFF,FUZZOFF,-FUZZOFF,
    FUZZOFF,FUZZOFF,FUZZOFF,-FUZZOFF,-FUZZOFF,-FUZZOFF,-FUZZOFF,
    FUZZOFF,-FUZZOFF,-FUZZOFF,FUZZOFF,FUZZOFF,FUZZOFF,FUZZOFF,-FUZZOFF,
    FUZZOFF,-FUZZOFF,FUZZOFF,FUZZOFF,-FUZZOFF,-FUZZOFF,FUZZOFF,
    FUZZOFF,-FUZZOFF,-FUZZOFF,-FUZZOFF,-FUZZOFF,FUZZOFF,FUZZOFF,
    FUZZOFF,FUZZOFF,-FUZZOFF,FUZZOFF,FUZZOFF,-FUZZOFF,FUZZOFF 
]; 

static mut fuzzpos: usize = 0; 

//
// Framebuffer postprocessing.
// Creates a fuzzy image by copying pixels
//  from adjacent ones to left and right.
// Used with an all black colormap, this
//  could create the SHADOW effect,
//  i.e. spectres and invisible players.
//
pub unsafe fn R_DrawFuzzColumn (dc: &mut R_DrawColumn_params_t) { 
    // Adjust borders. Low... 
    if dc.dc_yl == 0 {
        dc.dc_yl = 1;
    }

    // .. and high.
    if dc.dc_yh == viewheight-1 {
        dc.dc_yh = viewheight - 2; 
    }
         
    let count = dc.dc_yh - dc.dc_yl; 

    // Zero length.
    if count < 0 {
        return; 
    }
     
    // Does not work with blocky mode.
    let mut dest: *mut u8 = ylookup[dc.dc_yl as usize].offset(columnofs[dc.dc_x as usize] as isize); 

    // Looks familiar.
    let fracstep: fixed_t = dc.dc_iscale; 
    let mut frac: fixed_t =
            dc.dc_texturemid.wrapping_add(
                fracstep.wrapping_mul((dc.dc_yl - centery) as fixed_t));

    // Looks like an attempt at dithering,
    //  using the colormap #6 (of 0-31, a bit
    //  brighter than average).
    for _ in 0 ..= count {
        // Lookup framebuffer, and retrieve
        //  a pixel that is either one column
        //  left or right of the current one.
        // Add index from colormap to index.
        *dest = *colormaps.offset((6*256) as isize +
                    *dest.offset(fuzzoffset[fuzzpos]) as isize);

        // Clamp table lookup index.
        fuzzpos += 1;
        if fuzzpos == FUZZTABLE {
            fuzzpos = 0;
        }
        
        dest = dest.offset(SCREENWIDTH as isize); 
        frac = frac.wrapping_add(fracstep);
    }
} 


 
  

//
// R_DrawTranslatedColumn
// Used to draw player sprites
//  with the green colorramp mapped to others.
// Could be used with different translation
//  tables, e.g. the lighter colored version
//  of the BaronOfHell, the HellKnight, uses
//  identical sprites, kinda brightened up.
//

pub unsafe fn R_DrawTranslatedColumn (dc: &mut R_DrawColumn_params_t) { 
    let count = dc.dc_yh - dc.dc_yl; 

    // Zero length.
    if count < 0 {
        return; 
    }
     
    // FIXME. As above.
    let mut dest: *mut u8 = ylookup[dc.dc_yl as usize].offset(columnofs[dc.dc_x as usize] as isize); 

    // Looks familiar.
    let fracstep: fixed_t = dc.dc_iscale; 
    let mut frac: fixed_t =
            dc.dc_texturemid.wrapping_add(
                fracstep.wrapping_mul((dc.dc_yl - centery) as fixed_t));

    // Here we do an additional index re-mapping.
    for _ in 0 ..= count {
        // Translation tables are used
        //  to map certain colorramps to other ones,
        //  used with PLAY sprites.
        // Thus the "green" ramp of the player 0 sprite
        //  is mapped to gray, red, black/indigo. 
        *dest = *dc.dc_colormap.offset(
                    *dc.dc_translation.offset(
                        *dc.dc_source.offset(((frac>>FRACBITS)&127) as isize)
                            as isize)
                        as isize);
        dest = dest.offset(SCREENWIDTH as isize); 
        frac = frac.wrapping_add(fracstep);
    }
}




//
// R_InitTranslationTables
// Creates the translation tables to map
//  the green color ramp to gray, brown, red.
// Assumes a given structure of the PLAYPAL.
// Could be read from a lump instead.
//
pub unsafe fn R_InitTranslationTables () {
    translationtables = Z_Malloc (256*3+255, PU_STATIC,
                                        std::ptr::null_mut());
    //translationtables = (byte *)(( (intptr_t)translationtables + 255 )& ~255); // DSB-3
    
    // translate just the 16 green colors
    for i in 0 as u8 ..= 255 {
        let j: isize = i as isize;
        if i >= 0x70 && i<= 0x7f {
            // map green ramp to gray, brown, red
            *translationtables.offset(j) = 0x60 + (i&0xf);
            *translationtables.offset(j+256) = 0x40 + (i&0xf);
            *translationtables.offset(j+512) = 0x20 + (i&0xf);
        } else {
            // Keep all other colors as is.
            *translationtables.offset(j) = i;
            *translationtables.offset(j+256) = i;
            *translationtables.offset(j+512) = i;
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

//
// Draws the actual span.
pub fn R_DrawSpan (ds: &mut R_DrawSpan_params_t) { 
   
    unsafe {
        let mut xfrac: fixed_t = ds.ds_xfrac;
        let mut yfrac: fixed_t = ds.ds_yfrac;
         
        let mut dest: *mut u8 = ylookup[ds.ds_y as usize].offset(columnofs[ds.ds_x1 as usize] as isize);

        // We do not check for zero spans here?
        let count = ds.ds_x2 - ds.ds_x1; 

        for _ in 0 ..= count {
            // Current texture index in u,v.
            let spot = ((yfrac>>(16-6))&(63*64)) + ((xfrac>>16)&63);

            // Lookup pixel from flat texture tile,
            //  re-index using light/colormap.
            *dest = *ds.ds_colormap.offset(*ds.ds_source.offset(spot as isize) as isize);
            dest = dest.offset(1);

            // Next step in u,v.
            xfrac = xfrac.wrapping_add(ds.ds_xstep);
            yfrac = yfrac.wrapping_add(ds.ds_ystep);
        }
    }
} 


pub fn R_DrawSpanLow (ds: &mut R_DrawSpan_params_t) { 
    R_DrawSpan(ds);
}

pub fn R_DrawColumnLow(_dc: &mut R_DrawColumn_params_t) { 
    panic!("No implementation for R_DrawColumnLow");
}

//
// R_InitBuffer 
// Creats lookup tables that avoid
//  multiplies and other hazzles
//  for getting the framebuffer address
//  of a pixel to draw.
//
pub fn R_InitBuffer(width: i32, height: i32) {
    unsafe {
        // Handle resize,
        //  e.g. smaller view windows
        //  with border and/or status bar.
        viewwindowx = (SCREENWIDTH as i32 - width) >> 1; 

        // Column offset. For windows.
        for i in 0 .. width {
            columnofs[i as usize] = viewwindowx + i;
        }

        // Samw with base row offset.
        if width == SCREENWIDTH as i32 {
            viewwindowy = 0; 
        } else {
            viewwindowy = (SCREENHEIGHT as i32 - SBARHEIGHT as i32 - height) >> 1; 
        }

        // Preclaculate all row offsets.
        for i in 0 .. height {
            ylookup[i as usize] = screens[0].offset(
                ((i + viewwindowy) as isize) * (SCREENWIDTH as isize));
        }
    }
} 

#[no_mangle]
pub unsafe extern "C" fn R_FillBackScreen () { 
    if scaledviewwidth != SCREENWIDTH as i32 {
        panic!("No implementation for R_FillBackScreen");
    }
}
 
#[no_mangle]
pub unsafe extern "C" fn R_DrawViewBorder() { 
    if scaledviewwidth != SCREENWIDTH as i32 {
        panic!("No implementation for R_DrawViewBorder");
    }
}
 
#[no_mangle]
pub extern "C" fn R_VideoErase() { 
    panic!("No implementation for R_VideoErase");
}
