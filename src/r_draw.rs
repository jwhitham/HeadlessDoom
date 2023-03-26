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
use libc::c_int;

type fixed_t = u32;

const FRACBITS: i32 = 16;
const SCREENWIDTH: usize = 320;


// static const char
// rcsid[] = "$Id: r_draw.c,v 1.4 1997/02/03 16:47:55 b1 Exp $";


// #include "doomdef.h"

// #include "i_system.h"
// #include "z_zone.h"
// #include "w_wad.h"
// 
// #include "r_local.h"

// Needs access to LFB (guess what).
// #include "v_video.h"

// State.
// #include "doomstat.h"


// ?

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

extern {
    static ylookup: [*mut u8; SCREENWIDTH];
    static columnofs: [c_int; SCREENWIDTH];

    static centery: c_int; 
}

extern {
    static dc_colormap: *const u8;
    static dc_x: c_int; 
    static mut dc_yl: c_int; 
    static mut dc_yh: c_int; 
    static dc_iscale: fixed_t; 
    static dc_texturemid: fixed_t;

    static dc_source: *const u8;
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
        let mut dest: *mut u8 = ylookup[dc_yl as usize].offset(columnofs[dc_x as usize] as isize); 

        // Determine scaling,
        //  which is the only mapping to be done.
        let fracstep: fixed_t = dc_iscale; 
        let mut frac: fixed_t =
                dc_texturemid.wrapping_add(
                    fracstep.wrapping_mul((dc_yl - centery) as fixed_t));

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
            frac = frac.wrapping_add(fracstep);
            if count == 0 {
                break;
            }
            count -= 1;
        }
    }
} 


#[no_mangle]
pub extern "C" fn R_DrawColumnLow () { 
    R_DrawColumn();
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

extern {
    static colormaps: *const u8;
    static viewheight: i32;
}

//
// Framebuffer postprocessing.
// Creates a fuzzy image by copying pixels
//  from adjacent ones to left and right.
// Used with an all black colormap, this
//  could create the SHADOW effect,
//  i.e. spectres and invisible players.
//
#[no_mangle]
pub extern "C" fn R_DrawFuzzColumn () { 
    unsafe {
        // Adjust borders. Low... 
        if dc_yl == 0 {
            dc_yl = 1;
        }

        // .. and high.
        if dc_yh == viewheight-1 {
            dc_yh = viewheight - 2; 
        }
             
        let mut count = dc_yh - dc_yl; 

        // Zero length.
        if count < 0 {
            return; 
        }
         
        // Does not work with blocky mode.
        let mut dest: *mut u8 = ylookup[dc_yl as usize].offset(columnofs[dc_x as usize] as isize); 

        // Looks familiar.
        let fracstep: fixed_t = dc_iscale; 
        let mut frac: fixed_t =
                dc_texturemid.wrapping_add(
                    fracstep.wrapping_mul((dc_yl - centery) as fixed_t));

        // Looks like an attempt at dithering,
        //  using the colormap #6 (of 0-31, a bit
        //  brighter than average).
        loop {
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
            if count == 0 {
                break;
            }
            count -= 1;
        }
    }
} 


 
/*
  

//
// R_DrawTranslatedColumn
// Used to draw player sprites
//  with the green colorramp mapped to others.
// Could be used with different translation
//  tables, e.g. the lighter colored version
//  of the BaronOfHell, the HellKnight, uses
//  identical sprites, kinda brightened up.
//
byte*	dc_translation;
byte*	translationtables;

void R_DrawTranslatedColumn (void) 
{ 
    int			count; 
    byte*		dest; 
    fixed_t		frac;
    fixed_t		fracstep;	 
 
    count = dc_yh - dc_yl; 
    if (count < 0) 
	return; 
				 
#ifdef RANGECHECK 
    if ((unsigned)dc_x >= SCREENWIDTH
	|| dc_yl < 0
	|| dc_yh >= SCREENHEIGHT)
    {
	I_Error ( "R_DrawColumn: %i to %i at %i",
		  dc_yl, dc_yh, dc_x);
    }
    
#endif 


    // WATCOM VGA specific.
    /* Keep for fixing.
    if (detailshift)
    {
	if (dc_x & 1)
	    outp (SC_INDEX+1,12); 
	else
	    outp (SC_INDEX+1,3);
	
	dest = destview + dc_yl*80 + (dc_x>>1); 
    }
    else
    {
	outp (SC_INDEX+1,1<<(dc_x&3)); 

	dest = destview + dc_yl*80 + (dc_x>>2); 
    }*/

    
    // FIXME. As above.
    dest = ylookup[dc_yl] + columnofs[dc_x]; 

    // Looks familiar.
    fracstep = dc_iscale; 
    frac = dc_texturemid + (dc_yl-centery)*fracstep; 

    // Here we do an additional index re-mapping.
    do 
    {
	// Translation tables are used
	//  to map certain colorramps to other ones,
	//  used with PLAY sprites.
	// Thus the "green" ramp of the player 0 sprite
	//  is mapped to gray, red, black/indigo. 
	*dest = dc_colormap[dc_translation[dc_source[frac>>FRACBITS]]];
	dest += SCREENWIDTH;
	
	frac += fracstep; 
    } while (count--); 
} 




//
// R_InitTranslationTables
// Creates the translation tables to map
//  the green color ramp to gray, brown, red.
// Assumes a given structure of the PLAYPAL.
// Could be read from a lump instead.
//
void R_InitTranslationTables (void)
{
    int		i;
	
    translationtables = Z_Malloc (256*3+255, PU_STATIC, 0);
    translationtables = (byte *)(( (intptr_t)translationtables + 255 )& ~255); // DSB-3
    
    // translate just the 16 green colors
    for (i=0 ; i<256 ; i++)
    {
	if (i >= 0x70 && i<= 0x7f)
	{
	    // map green ramp to gray, brown, red
	    translationtables[i] = 0x60 + (i&0xf);
	    translationtables [i+256] = 0x40 + (i&0xf);
	    translationtables [i+512] = 0x20 + (i&0xf);
	}
	else
	{
	    // Keep all other colors as is.
	    translationtables[i] = translationtables[i+256] 
		= translationtables[i+512] = i;
	}
    }
}

*/

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

    static ds_colormap: *const u8; 

    static ds_xfrac: fixed_t; 
    static ds_yfrac: fixed_t; 
    static ds_xstep: fixed_t; 
    static ds_ystep: fixed_t;

    // start of a 64*64 tile image 
    static ds_source: *const u8;	
}

//
// Draws the actual span.
#[no_mangle]
pub extern "C" fn R_DrawSpan () { 
   
    unsafe {
        let mut xfrac: fixed_t = ds_xfrac;
        let mut yfrac: fixed_t = ds_yfrac;
         
        let mut dest: *mut u8 = ylookup[ds_y as usize].offset(columnofs[ds_x1 as usize] as isize);

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
            xfrac = xfrac.wrapping_add(ds_xstep);
            yfrac = yfrac.wrapping_add(ds_ystep);
            if count == 0 {
                break;
            }
            count -= 1;
        }
    }
} 


#[no_mangle]
pub extern "C" fn R_DrawSpanLow () { 
    R_DrawSpan();
}

/*
//
// R_InitBuffer 
// Creats lookup tables that avoid
//  multiplies and other hazzles
//  for getting the framebuffer address
//  of a pixel to draw.
//
void
R_InitBuffer
( int		width,
  int		height ) 
{ 
    int		i; 

    // Handle resize,
    //  e.g. smaller view windows
    //  with border and/or status bar.
    viewwindowx = (SCREENWIDTH-width) >> 1; 

    // Column offset. For windows.
    for (i=0 ; i<width ; i++) 
	columnofs[i] = viewwindowx + i;

    // Samw with base row offset.
    if (width == SCREENWIDTH) 
	viewwindowy = 0; 
    else 
	viewwindowy = (SCREENHEIGHT-SBARHEIGHT-height) >> 1; 

    // Preclaculate all row offsets.
    for (i=0 ; i<height ; i++) 
	ylookup[i] = screens[0] + (i+viewwindowy)*SCREENWIDTH; 
} 
 
 


//
// R_FillBackScreen
// Fills the back screen with a pattern
//  for variable screen sizes
// Also draws a beveled edge.
//
void R_FillBackScreen (void) 
{ 
    byte*	src;
    byte*	dest; 
    int		x;
    int		y; 
    patch_t*	patch;

    // DOOM border patch.
    char	name1[] = "FLOOR7_2";

    // DOOM II border patch.
    char	name2[] = "GRNROCK";	

    char*	name;
	
    if (scaledviewwidth == 320)
	return;
	
    if ( gamemode == commercial)
	name = name2;
    else
	name = name1;
    
    src = W_CacheLumpName (name, PU_CACHE); 
    dest = screens[1]; 
	 
    for (y=0 ; y<SCREENHEIGHT-SBARHEIGHT ; y++) 
    { 
	for (x=0 ; x<SCREENWIDTH/64 ; x++) 
	{ 
	    memcpy (dest, src+((y&63)<<6), 64); 
	    dest += 64; 
	} 

	if (SCREENWIDTH&63) 
	{ 
	    memcpy (dest, src+((y&63)<<6), SCREENWIDTH&63); 
	    dest += (SCREENWIDTH&63); 
	} 
    } 
	
    patch = W_CacheLumpName ("brdr_t",PU_CACHE);

    for (x=0 ; x<scaledviewwidth ; x+=8)
	V_DrawPatch (viewwindowx+x,viewwindowy-8,1,patch);
    patch = W_CacheLumpName ("brdr_b",PU_CACHE);

    for (x=0 ; x<scaledviewwidth ; x+=8)
	V_DrawPatch (viewwindowx+x,viewwindowy+viewheight,1,patch);
    patch = W_CacheLumpName ("brdr_l",PU_CACHE);

    for (y=0 ; y<viewheight ; y+=8)
	V_DrawPatch (viewwindowx-8,viewwindowy+y,1,patch);
    patch = W_CacheLumpName ("brdr_r",PU_CACHE);

    for (y=0 ; y<viewheight ; y+=8)
	V_DrawPatch (viewwindowx+scaledviewwidth,viewwindowy+y,1,patch);


    // Draw beveled edge. 
    V_DrawPatch (viewwindowx-8,
		 viewwindowy-8,
		 1,
		 W_CacheLumpName ("brdr_tl",PU_CACHE));
    
    V_DrawPatch (viewwindowx+scaledviewwidth,
		 viewwindowy-8,
		 1,
		 W_CacheLumpName ("brdr_tr",PU_CACHE));
    
    V_DrawPatch (viewwindowx-8,
		 viewwindowy+viewheight,
		 1,
		 W_CacheLumpName ("brdr_bl",PU_CACHE));
    
    V_DrawPatch (viewwindowx+scaledviewwidth,
		 viewwindowy+viewheight,
		 1,
		 W_CacheLumpName ("brdr_br",PU_CACHE));
} 
 

//
// Copy a screen buffer.
//
void
R_VideoErase
( unsigned	ofs,
  int		count ) 
{ 
  // LFB copy.
  // This might not be a good idea if memcpy
  //  is not optiomal, e.g. byte by byte on
  //  a 32bit CPU, as GNU GCC/Linux libc did
  //  at one point.
    memcpy (screens[0]+ofs, screens[1]+ofs, count); 
} 


//
// R_DrawViewBorder
// Draws the border around the view
//  for different size windows?
//
void
V_MarkRect
( int		x,
  int		y,
  int		width,
  int		height ); 
 
void R_DrawViewBorder (void) 
{ 
    int		top;
    int		side;
    int		ofs;
    int		i; 
 
    if (scaledviewwidth == SCREENWIDTH) 
	return; 
  
    top = ((SCREENHEIGHT-SBARHEIGHT)-viewheight)/2; 
    side = (SCREENWIDTH-scaledviewwidth)/2; 
 
    // copy top and one line of left side 
    R_VideoErase (0, top*SCREENWIDTH+side); 
 
    // copy one line of right side and bottom 
    ofs = (viewheight+top)*SCREENWIDTH-side; 
    R_VideoErase (ofs, top*SCREENWIDTH+side); 
 
    // copy sides using wraparound 
    ofs = top*SCREENWIDTH + SCREENWIDTH-side; 
    side <<= 1;
    
    for (i=1 ; i<viewheight ; i++) 
    { 
	R_VideoErase (ofs, side); 
	ofs += SCREENWIDTH; 
    } 

    // ? 
    V_MarkRect (0,0,SCREENWIDTH, SCREENHEIGHT-SBARHEIGHT); 
} 
 
*/

