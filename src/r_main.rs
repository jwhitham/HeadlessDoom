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
use crate::r_draw::R_DrawSpan;
use crate::r_draw::R_InitBuffer;
use crate::r_draw::R_InitTranslationTables;
use crate::r_draw::R_DrawSpanLow;
use crate::r_draw::R_DrawColumnLow;
use crate::r_draw::R_DrawColumn_params_t;
use crate::r_draw::R_DrawSpan_params_t;
use crate::r_draw::VideoContext_t;
use crate::r_draw::empty_VideoContext;
use crate::r_bsp::R_RenderBSPNode;
use crate::r_bsp::R_ClearClipSegs;
use crate::r_bsp::R_ClearDrawSegs;
use crate::r_bsp::BspContext_t;
use crate::r_bsp::empty_BspContext;
use crate::r_bsp::seg_index_t;
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
use crate::r_data::RenderData_t;
use crate::r_data::empty_RenderData;
use crate::r_data::COLORMAP_SIZE;
use crate::r_data::NULL_COLORMAP;
use crate::r_data::colormap_index_t;
use crate::r_sky::R_InitSkyMap;
use crate::r_plane::yslope;
use crate::r_plane::distscale;
use crate::r_segs::rw_normalangle;
use crate::r_segs::rw_distance;
use crate::r_segs::walllights;
use crate::r_things::pspritescale;
use crate::r_things::pspriteiscale;
use crate::r_things::screenheightarray;

type dc_function_t = unsafe fn (rc: &mut RenderContext_t, dc: &mut R_DrawColumn_params_t);
type ds_function_t = unsafe fn (rc: &mut RenderContext_t, ds: &mut R_DrawSpan_params_t);

pub struct ViewContext_t {
    pub viewx: fixed_t,
    pub viewy: fixed_t,
    pub viewz: fixed_t,
    pub viewcos: fixed_t,
    pub viewsin: fixed_t,
    pub viewangle: angle_t,
    pub viewangleoffset: i32,
    pub clipangle: angle_t,
    pub viewangletox: [i32; (FINEANGLES / 2) as usize],
}

pub const empty_ViewContext: ViewContext_t = ViewContext_t {
    viewx: 0,
    viewy: 0,
    viewz: 0,
    viewcos: 0,
    viewsin: 0,
    viewangle: 0,
    viewangleoffset: 0,
    clipangle: 0,
    viewangletox: [0; (FINEANGLES / 2) as usize],
};

pub struct RenderContext_t {
    pub rd: RenderData_t,
    pub vc: VideoContext_t,
    pub bc: BspContext_t,
    pub centerx: i32,
    pub centery: i32,
    pub centerxfrac: fixed_t,
    pub centeryfrac: fixed_t,
    pub projection: fixed_t,
    pub fixedcolormap_index: colormap_index_t,
    pub colfunc: dc_function_t,
    pub fuzzcolfunc: dc_function_t,
    pub basecolfunc: dc_function_t,
    pub spanfunc: ds_function_t,
    pub detailshift: i32,
    pub extralight: i32,
    pub viewplayer: *mut player_t,
    pub scalelight: [[colormap_index_t; MAXLIGHTSCALE as usize]; LIGHTLEVELS as usize],
    pub xtoviewangle: [angle_t; (SCREENWIDTH + 1) as usize],
    pub sscount: i32,
    pub zlight: [[colormap_index_t; MAXLIGHTZ as usize]; LIGHTLEVELS as usize],
    setblocks: i32,
    setdetail: i32,
    framecount: i32,
    scalelightfixed: [colormap_index_t; MAXLIGHTSCALE as usize],
    pub view: ViewContext_t,
}

const empty_RenderContext: RenderContext_t = RenderContext_t {
    rd: empty_RenderData,
    vc: empty_VideoContext,
    bc: empty_BspContext,
    centerx: 0,
    centery: 0,
    centerxfrac: 0,
    centeryfrac: 0,
    projection: 0,
    fixedcolormap_index: NULL_COLORMAP,
    colfunc: R_DrawColumn,
    fuzzcolfunc: R_DrawColumn,
    basecolfunc: R_DrawColumn,
    spanfunc: R_DrawSpan,
    detailshift: 0,
    extralight: 0,
    viewplayer: std::ptr::null_mut(),
    scalelight: [[NULL_COLORMAP; MAXLIGHTSCALE as usize]; LIGHTLEVELS as usize],
    xtoviewangle: [0; (SCREENWIDTH + 1) as usize],
    sscount: 0,
    zlight: [[NULL_COLORMAP; MAXLIGHTZ as usize]; LIGHTLEVELS as usize],
    setblocks: 0,
    setdetail: 0,
    framecount: 0,
    scalelightfixed: [NULL_COLORMAP; MAXLIGHTSCALE as usize],
    view: empty_ViewContext,
};

pub static mut remove_this_rc_global: RenderContext_t = empty_RenderContext;



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
                               line: seg_index_t) -> i32 {
    let lx = (*(*segs.offset(line as isize)).v1).x;
    let ly = (*(*segs.offset(line as isize)).v1).y;
    let ldx = (*(*segs.offset(line as isize)).v2).x - lx;
    let ldy = (*(*segs.offset(line as isize)).v2).y - ly;
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

pub unsafe fn R_PointToAngle (view: &mut ViewContext_t, x: fixed_t, y: fixed_t) -> angle_t {
    return R_PointToAngle_common(x - view.viewx, y - view.viewy);
}


#[no_mangle] // called from p_map and others
pub unsafe extern "C" fn R_PointToAngle2
        (x1: fixed_t, y1: fixed_t,
         x2: fixed_t, y2: fixed_t) -> angle_t {
    let mut view = &mut remove_this_rc_global.view;
    view.viewx = x1;
    view.viewy = y1;
    return R_PointToAngle_common(x2 - x1, y2 - y1);
}

pub unsafe fn R_PointToDist(view: &mut ViewContext_t, x: fixed_t, y: fixed_t) -> fixed_t {
    let mut dx = i32::abs(x - view.viewx);
    let mut dy = i32::abs(y - view.viewy);

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
pub unsafe fn R_ScaleFromGlobalAngle (rc: &mut RenderContext_t, visangle: angle_t) -> fixed_t {
    let anglea: u32 = ANG90.wrapping_add(visangle.wrapping_sub(rc.view.viewangle)) as u32;
    let angleb: u32 = ANG90.wrapping_add(visangle.wrapping_sub(rw_normalangle)) as u32;

    // both sines are allways positive
    let sinea: i32 = finesine[(anglea>>ANGLETOFINESHIFT) as usize];
    let sineb: i32 = finesine[(angleb>>ANGLETOFINESHIFT) as usize];
    let num: fixed_t = FixedMul(rc.projection,sineb)<<rc.detailshift;
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
unsafe fn R_InitTextureMapping (rc: &mut RenderContext_t) {
    let mut t: i32;
    
    // Use tangent table to generate viewangletox:
    //  viewangletox will give the next greatest x
    //  after the view angle.
    //
    // Calc focallength
    //  so FIELDOFVIEW angles covers SCREENWIDTH.
    let focallength = FixedDiv (rc.centerxfrac,
                finetangent[(FINEANGLES/4+FIELDOFVIEW/2) as usize] );
    
    for i in 0 .. (FINEANGLES / 2) as usize {
        if finetangent[i] > (FRACUNIT*2) as i32 {
            t = -1;
        } else if finetangent[i] < -((FRACUNIT*2) as i32) {
            t = viewwidth+1;
        } else {
            t = FixedMul (finetangent[i], focallength);
            t = (rc.centerxfrac - t + (FRACUNIT - 1) as i32) >> FRACBITS;
            t = i32::max(-1, i32::min(viewwidth + 1, t));
        }
        rc.view.viewangletox[i] = t;
    }
    
    // Scan viewangletox[] to generate xtoviewangle[]:
    //  xtoviewangle will give the smallest view angle
    //  that maps to x.	
    for x in 0 ..= viewwidth {
        let mut i: usize = 0;
        while rc.view.viewangletox[i] > x {
            i += 1;
        }
        rc.xtoviewangle[x as usize] = ((i as u32) << ANGLETOFINESHIFT).wrapping_sub(ANG90);
    }
    
    // Take out the fencepost cases from viewangletox.
    for i in 0 .. (FINEANGLES / 2) as usize {
        //t = FixedMul (finetangent[i], focallength);
        //t = centerx - t;
        
        if rc.view.viewangletox[i] == -1 {
            rc.view.viewangletox[i] = 0;
        } else if rc.view.viewangletox[i] == (viewwidth+1) {
            rc.view.viewangletox[i]  = viewwidth;
        }
    }
    
    rc.view.clipangle = rc.xtoviewangle[0];
}

//
// R_InitLightTables
// Only inits the zlight table,
//  because the scalelight table changes with view size.
//
const DISTMAP: i32 = 2;

unsafe fn R_InitLightTables (rc: &mut RenderContext_t) {
    // Calculate the light levels to use
    //  for each level / distance combination.
    for i in 0 .. LIGHTLEVELS as u32 {
        let startmap: i32 = (((LIGHTLEVELS-1-i)*2)*NUMCOLORMAPS/LIGHTLEVELS) as i32;
        for j in 0 .. MAXLIGHTZ as u32 {
            let mut scale: i32 = FixedDiv ((SCREENWIDTH/2*FRACUNIT) as i32, ((j+1)<<LIGHTZSHIFT) as i32);
            scale >>= LIGHTSCALESHIFT;
            let mut level: i32 = startmap - scale/DISTMAP;
            
            level = i32::max(0, i32::min((NUMCOLORMAPS - 1) as i32, level));

            rc.zlight[i as usize][j as usize] = (level * (COLORMAP_SIZE as i32)) as colormap_index_t;
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
    let mut rc = &mut remove_this_rc_global;
    setsizeneeded = c_true;
    rc.setblocks = blocks;
    rc.setdetail = detail;
}

//
// R_ExecuteSetViewSize
//
#[no_mangle] // called from D_Display
pub unsafe extern "C" fn R_ExecuteSetViewSize () {
    let rc = &mut remove_this_rc_global;

    setsizeneeded = c_false;

    if rc.setblocks == 11 {
        scaledviewwidth = SCREENWIDTH as i32;
        viewheight = SCREENHEIGHT as i32;
    } else {
        scaledviewwidth = rc.setblocks*32;
        viewheight = (rc.setblocks*168/10)&!7;
    }
    
    rc.detailshift = rc.setdetail;
    viewwidth = scaledviewwidth>>rc.detailshift;
    
    rc.centery = viewheight/2;
    rc.centerx = viewwidth/2;
    rc.centerxfrac = rc.centerx<<FRACBITS;
    rc.centeryfrac = rc.centery<<FRACBITS;
    rc.projection = rc.centerxfrac;

    if rc.detailshift == c_false {
        rc.basecolfunc = R_DrawColumn;
        rc.colfunc = R_DrawColumn;
        rc.fuzzcolfunc = R_DrawFuzzColumn;
        rc.spanfunc = R_DrawSpan;
    } else {
        rc.basecolfunc = R_DrawColumnLow;
        rc.colfunc = R_DrawColumnLow;
        rc.fuzzcolfunc = R_DrawFuzzColumn;
        rc.spanfunc = R_DrawSpanLow;
    }

    R_InitBuffer (rc, scaledviewwidth, viewheight);
    
    R_InitTextureMapping (rc);
    
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
        yslope[i as usize] = FixedDiv(((viewwidth<<rc.detailshift)/2)*(FRACUNIT as i32), dy);
    }
    
    for i in 0 .. viewwidth {
        let cosadj: fixed_t = fixed_t::abs(*finecosine.offset((rc.xtoviewangle[i as usize]>>ANGLETOFINESHIFT) as isize));
        distscale[i as usize] = FixedDiv (FRACUNIT as i32, cosadj);
    }
    
    // Calculate the light levels to use
    //  for each level / scale combination.
    for i in 0 .. LIGHTLEVELS as u32 {
        let startmap: i32 = (((LIGHTLEVELS-1-i)*2)*NUMCOLORMAPS/LIGHTLEVELS) as i32;
        for j in 0 .. MAXLIGHTSCALE as u32 {
            let mut level: i32 = startmap -
                ((((j * SCREENWIDTH) as i32) / (viewwidth << rc.detailshift)) / DISTMAP);
            
            level = i32::max(0, i32::min((NUMCOLORMAPS - 1) as i32, level));

            rc.scalelight[i as usize][j as usize] = (level * (COLORMAP_SIZE as i32)) as colormap_index_t;
        }
    }
}

//
// R_Init
//
#[no_mangle] // called from D_DoomMain
pub unsafe extern "C" fn R_Init () {
    let rc = &mut remove_this_rc_global;
    R_InitData (&mut rc.rd);
    print!("\nR_InitData");
    R_InitPointToAngle ();
    print!("\nR_InitPointToAngle");
    R_InitTables ();
    // viewwidth / viewheight / detailLevel are set by the defaults
    print!("\nR_InitTables");

    R_SetViewSize (screenblocks, detailLevel);
    R_InitPlanes ();
    print!("\nR_InitPlanes");
    R_InitLightTables (rc);
    print!("\nR_InitLightTables");
    R_InitSkyMap ();
    print!("\nR_InitSkyMap");
    R_InitTranslationTables (rc);
    print!("\nR_InitTranslationsTables");
    
    rc.framecount = 0;
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
unsafe fn R_SetupFrame (rc: &mut RenderContext_t, player: *mut player_t) {
    rc.viewplayer = player;
    rc.view.viewx = (*(*player).mo).x;
    rc.view.viewy = (*(*player).mo).y;
    rc.view.viewangle = (*(*player).mo).angle.wrapping_add(rc.view.viewangleoffset as u32);
    rc.extralight = (*player).extralight;

    rc.view.viewz = (*player).viewz;
    
    rc.view.viewsin = finesine[(rc.view.viewangle>>ANGLETOFINESHIFT) as usize];
    rc.view.viewcos = *finecosine.offset((rc.view.viewangle>>ANGLETOFINESHIFT) as isize);
    
    rc.sscount = 0;
    
    if (*player).fixedcolormap != 0 {
        rc.fixedcolormap_index = ((*player).fixedcolormap * (COLORMAP_SIZE as i32)) as colormap_index_t;
    
        walllights = rc.scalelightfixed.as_mut_ptr();

        for i in 0 .. MAXLIGHTSCALE as usize {
            rc.scalelightfixed[i] = rc.fixedcolormap_index;
        }
    } else {
        rc.fixedcolormap_index = NULL_COLORMAP;
    }
        
    rc.framecount += 1;
    validcount += 1;
}



//
// R_RenderView
//
#[no_mangle] // called from D_Display
pub unsafe extern "C" fn R_RenderPlayerView (player: *mut player_t) {

    let rc = &mut remove_this_rc_global;
    memcpy(rc.vc.screen.as_mut_ptr(), screens[0],
          (SCREENWIDTH * SCREENHEIGHT) as usize);
    R_SetupFrame (rc, player);

    // Clear buffers.
    R_ClearClipSegs (&mut rc.bc);
    R_ClearDrawSegs (&mut rc.bc);
    R_ClearPlanes (rc);
    R_ClearSprites ();
    
    // check for new console commands.
    NetUpdate ();

    // The head node is the last node output.
    R_RenderBSPNode (rc, numnodes-1);
    
    // Check for new console commands.
    NetUpdate ();
    
    R_DrawPlanes (rc);
    
    // Check for new console commands.
    NetUpdate ();
    
    R_DrawMasked (rc);

    memcpy(screens[0], rc.vc.screen.as_ptr(),
          (SCREENWIDTH * SCREENHEIGHT) as usize);

    // Check for new console commands.
    NetUpdate ();				
}
