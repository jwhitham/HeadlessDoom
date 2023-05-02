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
//	Rendering main loop and setup functions,
//	 utility functions (BSP, geometry, trigonometry).
//	See tables.c, too.
//
//-----------------------------------------------------------------------------


use crate::defs::*;
use crate::globals::*;
use crate::funcs::*;
use crate::m_fixed::FixedMul;
use crate::m_fixed::FixedDiv;
use crate::r_draw::R_DrawColumn;
use crate::r_draw::R_DrawFuzzColumn;
use crate::r_draw::R_DrawTranslatedColumn;
use crate::r_draw::R_DrawSpan;
use crate::r_draw::R_InitBuffer;
use crate::r_draw::R_InitTranslationTables;
use crate::r_draw::R_DrawSpanLow;
use crate::r_draw::R_DrawColumnLow;
use crate::r_draw::R_DrawColumn_params_t;
use crate::r_draw::R_DrawSpan_params_t;
use crate::r_draw::VideoContext_t;
use crate::r_draw::empty_VideoContext;
use crate::r_draw::COLORMAP_SIZE;
use crate::r_draw::NULL_COLORMAP;
use crate::r_draw::colormap_index_t;
use crate::r_bsp::R_RenderBSPNode;
use crate::r_bsp::R_ClearClipSegs;
use crate::r_bsp::R_ClearDrawSegs;
use crate::r_things::R_ClearSprites;
use crate::r_things::R_DrawMasked;
use crate::tables::tantoangle;
use crate::tables::SlopeDiv;
use crate::tables::finesine;
use crate::tables::finetangent;
use crate::r_plane::R_DrawPlanes;
use crate::r_plane::R_ClearPlanes;
use crate::r_plane::R_InitPlanes;
use crate::r_data::R_InitData;
use crate::r_data::DataContext_t;
use crate::r_data::empty_DataContext;
use crate::r_sky::R_InitSkyMap;
use crate::r_plane::yslope;
use crate::r_plane::distscale;
use crate::r_segs::rw_normalangle;
use crate::r_segs::rw_distance;
use crate::r_segs::walllights;
use crate::r_things::pspritescale;
use crate::r_things::pspriteiscale;
use crate::r_things::screenheightarray;

pub struct RenderContext_t {
    pub dc: DataContext_t,
    pub vc: VideoContext_t,
}

const empty_RenderContext: RenderContext_t = RenderContext_t {
    dc: empty_DataContext,
    vc: empty_VideoContext,
};

static mut remove_this_rc_global: RenderContext_t = empty_RenderContext;

pub static mut colfunc: unsafe fn (vc: &mut VideoContext_t, dc: &mut R_DrawColumn_params_t) = R_DrawColumn;
pub static mut fuzzcolfunc: unsafe fn (vc: &mut VideoContext_t, dc: &mut R_DrawColumn_params_t) = R_DrawColumn;
pub static mut basecolfunc: unsafe fn (vc: &mut VideoContext_t, dc: &mut R_DrawColumn_params_t) = R_DrawColumn;
static mut transcolfunc: unsafe fn (vc: &mut VideoContext_t, dc: &mut R_DrawColumn_params_t) = R_DrawColumn;
pub static mut spanfunc: unsafe fn (vc: &mut VideoContext_t, ds: &mut R_DrawSpan_params_t) = R_DrawSpan;

pub static mut detailshift: i32 = 0;
pub static mut centerxfrac: fixed_t = 0;
pub static mut centeryfrac: fixed_t = 0;
pub static mut viewx: fixed_t = 0;
pub static mut viewy: fixed_t = 0;
pub static mut viewz: fixed_t = 0;
pub static mut viewcos: fixed_t = 0;
pub static mut viewsin: fixed_t = 0;
pub static mut projection: fixed_t = 0;
pub static mut fixedcolormap_index: colormap_index_t = NULL_COLORMAP;
pub static mut extralight: i32 = 0;
pub static mut viewplayer: *mut player_t = std::ptr::null_mut();
pub static mut viewangleoffset: i32 = 0;
pub static mut scalelight: [[colormap_index_t; MAXLIGHTSCALE as usize]; LIGHTLEVELS as usize] = [
    [NULL_COLORMAP; MAXLIGHTSCALE as usize]; LIGHTLEVELS as usize];
pub static mut centery: i32 = 0; 
pub static mut xtoviewangle: [angle_t; (SCREENWIDTH + 1) as usize] = [0; (SCREENWIDTH + 1) as usize];
pub static mut clipangle: angle_t = 0;
pub static mut viewangletox: [i32; (FINEANGLES / 2) as usize] = [0; (FINEANGLES / 2) as usize];
pub static mut sscount: i32 = 0;
pub static mut zlight: [[colormap_index_t; MAXLIGHTZ as usize]; LIGHTLEVELS as usize] = [
    [NULL_COLORMAP; MAXLIGHTZ as usize]; LIGHTLEVELS as usize];
pub static mut viewangle: angle_t = 0;
static mut centerx: i32 = 0;
static mut setblocks: i32 = 0;
static mut setdetail: i32 = 0;
static mut framecount: i32 = 0;
static mut scalelightfixed: [colormap_index_t; MAXLIGHTSCALE as usize] = 
    [NULL_COLORMAP; MAXLIGHTSCALE as usize];

// Fineangles in the SCREENWIDTH wide window.
const FIELDOFVIEW: u32 = 2048;

//
// R_PointOnSide
// Traverse BSP (sub) tree,
//  check point against partition plane.
// Returns side 0 (front) or 1 (back).
//
fn R_PointOnSide_common(x: fixed_t, y: fixed_t,
                        lx: fixed_t, ly: fixed_t,
                        ldx: fixed_t, ldy: fixed_t) -> i32 {
    if ldx == 0 {
        if x <= lx {
            return if ldy > 0 { 1 } else { 0 };
        }
        return if ldy < 0 { 1 } else { 0 };
    }
    if ldy == 0 {
        if y <= ly {
            return if ldx < 0 { 1 } else { 0 };
        }
        return if ldx > 0 { 1 } else { 0 };
    }

    let dx = x.wrapping_sub(lx);
    let dy = y.wrapping_sub(ly);

    // Try to quickly decide by looking at sign bits.
    if (((ldy ^ ldx ^ dx ^ dy) as u32) & 0x80000000) != 0 {
        if (((ldy ^ dx) as u32) & 0x80000000) != 0 {
            // (left is negative)
            return 1;
        }
        return 0;
    }

    let left = FixedMul(ldy>>FRACBITS, dx);
    let right = FixedMul(dy, ldx>>FRACBITS);

    if right < left {
        // front side
        return 0;
    }
    // back side
    return 1;
}

pub unsafe fn R_PointOnSide(x: fixed_t, y: fixed_t,
                            node: *mut node_t) -> i32 {
    return R_PointOnSide_common(x, y,
                                (*node).x, (*node).y,
                                (*node).dx, (*node).dy);
}


pub unsafe fn R_PointOnSegSide(x: fixed_t, y: fixed_t,
                               line: *mut seg_t) -> i32 {
    let lx = (*(*line).v1).x;
    let ly = (*(*line).v1).y;
    let ldx = (*(*line).v2).x - lx;
    let ldy = (*(*line).v2).y - ly;
    return R_PointOnSide_common(x, y, lx, ly, ldx, ldy);
}

//
// R_PointToAngle
// To get a global angle from cartesian coordinates,
//  the coordinates are flipped until they are in
//  the first octant of the coordinate system, then
//  the y (<=x) is scaled and divided by x to get a
//  tangent (slope) value which is looked up in the
//  tantoangle[] table.

//

fn R_PointToAngle_common(px: fixed_t, py: fixed_t) -> angle_t {
    let mut x = px;
    let mut y = py;
    
    if (x == 0) && (y == 0) {
        return 0;
    }

    if x >= 0 {
        // x >=0
        if y >= 0 {
            // y>= 0

            if x>y {
                // octant 0
                return tantoangle[SlopeDiv(y, x)];
            } else {
                // octant 1
                return (ANG90-1).wrapping_sub(
                        tantoangle[SlopeDiv(x, y)]);
            }
        } else {
            // y<0
            y = -y;

            if x>y {
                // octant 8
                return (0 as angle_t).wrapping_sub(tantoangle[SlopeDiv(y, x)]);
            } else {
                // octant 7
                return ANG270.wrapping_add(tantoangle[SlopeDiv(x, y)]);
            }
        }
    } else {
        // x<0
        x = -x;

        if y >= 0 {
            // y>= 0
            if x > y {
                // octant 3
                return (ANG180-1).wrapping_sub(tantoangle[SlopeDiv(y, x)]);
            } else {
                // octant 2
                return ANG90.wrapping_add(tantoangle[SlopeDiv(x, y)]);
            }
        } else {
            // y<0
            y = -y;

            if x > y {
                // octant 4
                return ANG180.wrapping_add(tantoangle[SlopeDiv(y, x)]);
            } else {
                 // octant 5
                return (ANG270-1).wrapping_sub(tantoangle[SlopeDiv(x, y)]);
            }
        }
    }
}

pub unsafe fn R_PointToAngle (x: fixed_t, y: fixed_t) -> angle_t {
    return R_PointToAngle_common(x - viewx, y - viewy);
}


#[no_mangle] // called from p_map and others
pub unsafe extern "C" fn R_PointToAngle2
        (x1: fixed_t, y1: fixed_t,
         x2: fixed_t, y2: fixed_t) -> angle_t {
    viewx = x1;
    viewy = y1;
    return R_PointToAngle_common(x2 - x1, y2 - y1);
}

pub unsafe fn R_PointToDist(x: fixed_t, y: fixed_t) -> fixed_t {
    let mut dx = i32::abs(x - viewx);
    let mut dy = i32::abs(y - viewy);

    if dy > dx {
        let temp = dx;
        dx = dy;
        dy = temp;
    }
        
    let angle = (tantoangle[(FixedDiv(dy,dx) >> DBITS) as usize]+ANG90) >> ANGLETOFINESHIFT;

    // use as cosine
    let dist = FixedDiv (dx, finesine[angle as usize]);	
        
    return dist;
}

//
// R_InitPointToAngle
//
fn R_InitPointToAngle () {
    // UNUSED - now getting from tables.c
    // #if 0
    //     int i;
    //     long t;
    //     float f;
    // //
    // // slope (tangent) to angle lookup
    // //
    //     for (i=0 ; i<=SLOPERANGE ; i++)
    //     {
    //         f = atan( (float)i/SLOPERANGE )/(3.141592657*2);
    //         t = 0xffffffff*f;
    //         tantoangle[i] = t;
    //     }
    // #endif
}

//
// R_ScaleFromGlobalAngle
// Returns the texture mapping scale
//  for the current line (horizontal span)
//  at the given angle.
// rw_distance must be calculated first.
//
pub unsafe fn R_ScaleFromGlobalAngle (visangle: angle_t) -> fixed_t {
    let anglea: u32 = ANG90.wrapping_add(visangle.wrapping_sub(viewangle)) as u32;
    let angleb: u32 = ANG90.wrapping_add(visangle.wrapping_sub(rw_normalangle)) as u32;

    // both sines are allways positive
    let sinea: i32 = finesine[(anglea>>ANGLETOFINESHIFT) as usize];
    let sineb: i32 = finesine[(angleb>>ANGLETOFINESHIFT) as usize];
    let num: fixed_t = FixedMul(projection,sineb)<<detailshift;
    let den: i32 = FixedMul(rw_distance,sinea);
    let mut scale: fixed_t;

    if den > (num>>16) {
        scale = FixedDiv (num, den);

        scale = fixed_t::max(256, fixed_t::min(64 * FRACUNIT as fixed_t, scale));
    } else {
        scale = 64*FRACUNIT as fixed_t;
    }
    
    return scale;
}

//
// R_InitTables
//
fn R_InitTables () {
    // UNUSED: now getting from tables.c
    // #if 0
    //     int  i;
    //     float a;
    //     float fv;
    //     int  t;
    //     
    //     // viewangle tangent table
    //     for (i=0 ; i<FINEANGLES/2 ; i++)
    //     {
    //     a = (i-FINEANGLES/4+0.5)*PI*2/FINEANGLES;
    //     fv = FRACUNIT*tan (a);
    //     t = fv;
    //     finetangent[i] = t;
    //     }
    //     
    //     // finesine table
    //     for (i=0 ; i<5*FINEANGLES/4 ; i++)
    //     {
    //     // OPTIMIZE: mirror...
    //     a = (i+0.5)*PI*2/FINEANGLES;
    //     t = FRACUNIT*sin (a);
    //     finesine[i] = t;
    //     }
    // #endif
}

//
// R_InitTextureMapping
//
unsafe fn R_InitTextureMapping () {
    let mut t: i32;
    
    // Use tangent table to generate viewangletox:
    //  viewangletox will give the next greatest x
    //  after the view angle.
    //
    // Calc focallength
    //  so FIELDOFVIEW angles covers SCREENWIDTH.
    let focallength = FixedDiv (centerxfrac,
                finetangent[(FINEANGLES/4+FIELDOFVIEW/2) as usize] );
    
    for i in 0 .. (FINEANGLES / 2) as usize {
        if finetangent[i] > (FRACUNIT*2) as i32 {
            t = -1;
        } else if finetangent[i] < -((FRACUNIT*2) as i32) {
            t = viewwidth+1;
        } else {
            t = FixedMul (finetangent[i], focallength);
            t = (centerxfrac - t + (FRACUNIT - 1) as i32) >> FRACBITS;
            t = i32::max(-1, i32::min(viewwidth + 1, t));
        }
        viewangletox[i] = t;
    }
    
    // Scan viewangletox[] to generate xtoviewangle[]:
    //  xtoviewangle will give the smallest view angle
    //  that maps to x.	
    for x in 0 ..= viewwidth {
        let mut i: usize = 0;
        while viewangletox[i] > x {
            i += 1;
        }
        xtoviewangle[x as usize] = ((i as u32) << ANGLETOFINESHIFT).wrapping_sub(ANG90);
    }
    
    // Take out the fencepost cases from viewangletox.
    for i in 0 .. (FINEANGLES / 2) as usize {
        //t = FixedMul (finetangent[i], focallength);
        //t = centerx - t;
        
        if viewangletox[i] == -1 {
            viewangletox[i] = 0;
        } else if viewangletox[i] == (viewwidth+1) {
            viewangletox[i]  = viewwidth;
        }
    }
    
    clipangle = xtoviewangle[0];
}

//
// R_InitLightTables
// Only inits the zlight table,
//  because the scalelight table changes with view size.
//
const DISTMAP: i32 = 2;

unsafe fn R_InitLightTables () {
    // Calculate the light levels to use
    //  for each level / distance combination.
    for i in 0 .. LIGHTLEVELS as u32 {
        let startmap: i32 = (((LIGHTLEVELS-1-i)*2)*NUMCOLORMAPS/LIGHTLEVELS) as i32;
        for j in 0 .. MAXLIGHTZ as u32 {
            let mut scale: i32 = FixedDiv ((SCREENWIDTH/2*FRACUNIT) as i32, ((j+1)<<LIGHTZSHIFT) as i32);
            scale >>= LIGHTSCALESHIFT;
            let mut level: i32 = startmap - scale/DISTMAP;
            
            level = i32::max(0, i32::min((NUMCOLORMAPS - 1) as i32, level));

            zlight[i as usize][j as usize] = (level * (COLORMAP_SIZE as i32)) as colormap_index_t;
        }
    }
}


//
// R_SetViewSize
// Do not really change anything here,
//  because it might be in the middle of a refresh.
// The change will take effect next refresh.
//
#[no_mangle] // called from M_StartControlPanel
pub unsafe extern "C" fn R_SetViewSize(blocks: i32, detail: i32) {
    setsizeneeded = c_true;
    setblocks = blocks;
    setdetail = detail;
}

//
// R_ExecuteSetViewSize
//
#[no_mangle] // called from D_Display
pub unsafe extern "C" fn R_ExecuteSetViewSize () {

    setsizeneeded = c_false;

    if setblocks == 11 {
        scaledviewwidth = SCREENWIDTH as i32;
        viewheight = SCREENHEIGHT as i32;
    } else {
        scaledviewwidth = setblocks*32;
        viewheight = (setblocks*168/10)&!7;
    }
    
    detailshift = setdetail;
    viewwidth = scaledviewwidth>>detailshift;
    
    centery = viewheight/2;
    centerx = viewwidth/2;
    centerxfrac = centerx<<FRACBITS;
    centeryfrac = centery<<FRACBITS;
    projection = centerxfrac;

    if detailshift == c_false {
        basecolfunc = R_DrawColumn;
        colfunc = R_DrawColumn;
        fuzzcolfunc = R_DrawFuzzColumn;
        transcolfunc = R_DrawTranslatedColumn;
        spanfunc = R_DrawSpan;
    } else {
        basecolfunc = R_DrawColumnLow;
        colfunc = R_DrawColumnLow;
        fuzzcolfunc = R_DrawFuzzColumn;
        transcolfunc = R_DrawTranslatedColumn;
        spanfunc = R_DrawSpanLow;
    }

    R_InitBuffer (&mut remove_this_rc_global.vc, scaledviewwidth, viewheight);
    
    R_InitTextureMapping ();
    
    // psprite scales
    pspritescale = ((FRACUNIT as i32) * viewwidth) / (SCREENWIDTH as i32);
    pspriteiscale = ((FRACUNIT as i32) * (SCREENWIDTH as i32)) / (viewwidth as i32);
    
    // thing clipping
    for i in 0 .. viewwidth {
        screenheightarray[i as usize] = viewheight as i16;
    }
    
    // planes
    for i in 0 .. viewheight {
        let mut dy: fixed_t = ((i-(viewheight/2))<<FRACBITS) + ((FRACUNIT as fixed_t) / 2);
        dy = fixed_t::abs(dy);
        yslope[i as usize] = FixedDiv(((viewwidth<<detailshift)/2)*(FRACUNIT as i32), dy);
    }
    
    for i in 0 .. viewwidth {
        let cosadj: fixed_t = fixed_t::abs(*finecosine.offset((xtoviewangle[i as usize]>>ANGLETOFINESHIFT) as isize));
        distscale[i as usize] = FixedDiv (FRACUNIT as i32, cosadj);
    }
    
    // Calculate the light levels to use
    //  for each level / scale combination.
    for i in 0 .. LIGHTLEVELS as u32 {
        let startmap: i32 = (((LIGHTLEVELS-1-i)*2)*NUMCOLORMAPS/LIGHTLEVELS) as i32;
        for j in 0 .. MAXLIGHTSCALE as u32 {
            let mut level: i32 = startmap -
                ((((j * SCREENWIDTH) as i32) / (viewwidth << detailshift)) / DISTMAP);
            
            level = i32::max(0, i32::min((NUMCOLORMAPS - 1) as i32, level));

            scalelight[i as usize][j as usize] = (level * (COLORMAP_SIZE as i32)) as colormap_index_t;
        }
    }
}

//
// R_Init
//
#[no_mangle] // called from D_DoomMain
pub unsafe extern "C" fn R_Init () {
    R_InitData (&mut remove_this_rc_global.vc);
    print!("\nR_InitData");
    R_InitPointToAngle ();
    print!("\nR_InitPointToAngle");
    R_InitTables ();
    // viewwidth / viewheight / detailLevel are set by the defaults
    print!("\nR_InitTables");

    R_SetViewSize (screenblocks, detailLevel);
    R_InitPlanes ();
    print!("\nR_InitPlanes");
    R_InitLightTables ();
    print!("\nR_InitLightTables");
    R_InitSkyMap ();
    print!("\nR_InitSkyMap");
    R_InitTranslationTables (&mut remove_this_rc_global.vc);
    print!("\nR_InitTranslationsTables");
    
    framecount = 0;
}

//
// R_PointInSubsector
//
#[no_mangle] // called from P_RespawnSpecials and others
pub unsafe extern "C" fn R_PointInSubsector(x: fixed_t, y: fixed_t) -> *mut subsector_t {
    // single subsector is a special case
    if numnodes == 0 {
        return subsectors;
    }
        
    let mut nodenum = numnodes-1;

    while 0 == (nodenum & (NF_SUBSECTOR as i32)) {
        let node = nodes.offset(nodenum as isize);
        let side = R_PointOnSide(x, y, node);
        nodenum = (*node).children[side as usize] as i32;
    }
    
    return subsectors.offset((nodenum & !(NF_SUBSECTOR as i32)) as isize);
}

//
// R_SetupFrame
//
unsafe fn R_SetupFrame (player: *mut player_t) {
    viewplayer = player;
    viewx = (*(*player).mo).x;
    viewy = (*(*player).mo).y;
    viewangle = (*(*player).mo).angle.wrapping_add(viewangleoffset as u32);
    extralight = (*player).extralight;

    viewz = (*player).viewz;
    
    viewsin = finesine[(viewangle>>ANGLETOFINESHIFT) as usize];
    viewcos = *finecosine.offset((viewangle>>ANGLETOFINESHIFT) as isize);
    
    sscount = 0;
    
    if (*player).fixedcolormap != 0 {
        fixedcolormap_index = ((*player).fixedcolormap * (COLORMAP_SIZE as i32)) as colormap_index_t;
    
        walllights = scalelightfixed.as_mut_ptr();

        for i in 0 .. MAXLIGHTSCALE as usize {
            scalelightfixed[i] = fixedcolormap_index;
        }
    } else {
        fixedcolormap_index = NULL_COLORMAP;
    }
        
    framecount += 1;
    validcount += 1;
}



//
// R_RenderView
//
#[no_mangle] // called from D_Display
pub unsafe extern "C" fn R_RenderPlayerView (player: *mut player_t) {
    memcpy(remove_this_rc_global.vc.screen.as_mut_ptr(), screens[0],
          (SCREENWIDTH * SCREENHEIGHT) as usize);
    R_SetupFrame (player);

    // Clear buffers.
    R_ClearClipSegs ();
    R_ClearDrawSegs ();
    R_ClearPlanes ();
    R_ClearSprites ();
    
    // check for new console commands.
    NetUpdate ();

    // The head node is the last node output.
    R_RenderBSPNode (&mut remove_this_rc_global.vc, numnodes-1);
    
    // Check for new console commands.
    NetUpdate ();
    
    R_DrawPlanes (&mut remove_this_rc_global.vc);
    
    // Check for new console commands.
    NetUpdate ();
    
    R_DrawMasked (&mut remove_this_rc_global.vc);

    memcpy(screens[0], remove_this_rc_global.vc.screen.as_ptr(),
          (SCREENWIDTH * SCREENHEIGHT) as usize);

    // Check for new console commands.
    NetUpdate ();				
}
