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
use crate::m_fixed::FixedMul;
use crate::m_fixed::FixedDiv;
use crate::tables::tantoangle;
use crate::tables::SlopeDiv;
use crate::tables::finesine;
use crate::tables::finetangent;


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

#[no_mangle]
pub unsafe extern "C" fn R_PointOnSide(x: fixed_t, y: fixed_t,
                                       node: *mut node_t) -> i32 {
    return R_PointOnSide_common(x, y,
                                (*node).x, (*node).y,
                                (*node).dx, (*node).dy);
}


#[no_mangle]
pub unsafe extern "C" fn R_PointOnSegSide(x: fixed_t, y: fixed_t,
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

#[no_mangle]
pub unsafe extern "C" fn R_PointToAngle (x: fixed_t, y: fixed_t) -> angle_t {
    return R_PointToAngle_common(x - viewx, y - viewy);
}


#[no_mangle]
pub unsafe extern "C" fn R_PointToAngle2
        (x1: fixed_t, y1: fixed_t,
         x2: fixed_t, y2: fixed_t) -> angle_t {
    viewx = x1;
    viewy = y1;
    return R_PointToAngle_common(x2 - x1, y2 - y1);
}

#[no_mangle]
pub unsafe extern "C" fn R_PointToDist(x: fixed_t, y: fixed_t) -> fixed_t {
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
#[no_mangle]
pub extern "C" fn R_InitPointToAngle () {
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
#[no_mangle]
pub unsafe extern "C" fn R_ScaleFromGlobalAngle (visangle: angle_t) -> fixed_t {
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
#[no_mangle]
pub unsafe extern "C" fn R_InitTables () {
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
#[no_mangle]
pub unsafe extern "C" fn R_InitTextureMapping () {
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

#[no_mangle]
pub unsafe extern "C" fn R_InitLightTables () {
    // Calculate the light levels to use
    //  for each level / distance combination.
    for i in 0 .. LIGHTLEVELS as u32 {
        let startmap: i32 = (((LIGHTLEVELS-1-i)*2)*NUMCOLORMAPS/LIGHTLEVELS) as i32;
        for j in 0 .. MAXLIGHTZ as u32 {
            let mut scale: i32 = FixedDiv ((SCREENWIDTH/2*FRACUNIT) as i32, ((j+1)<<LIGHTZSHIFT) as i32);
            scale >>= LIGHTSCALESHIFT;
            let mut level: i32 = startmap - scale/DISTMAP;
            
            level = i32::max(0, i32::min((NUMCOLORMAPS - 1) as i32, level));

            zlight[i as usize][j as usize] = colormaps.offset((level*256) as isize);
        }
    }
}


