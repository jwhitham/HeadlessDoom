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
use crate::tables::tantoangle;
use crate::tables::SlopeDiv;


// Fineangles in the SCREENWIDTH wide window.
// const FIELDOFVIEW: usize = 2048;

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
