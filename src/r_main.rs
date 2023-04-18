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


// Fineangles in the SCREENWIDTH wide window.
// const FIELDOFVIEW: usize = 2048;

//
// R_PointOnSide
// Traverse BSP (sub) tree,
//  check point against partition plane.
// Returns side 0 (front) or 1 (back).
//
unsafe fn R_PointOnSide_common(x: fixed_t, y: fixed_t,
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

