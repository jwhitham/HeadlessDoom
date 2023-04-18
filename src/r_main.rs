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
#[no_mangle]
pub unsafe extern "C" fn R_PointOnSide(x: fixed_t, y: fixed_t,
                                       node: *mut node_t) -> i32 {
    if (*node).dx == 0 {
        if x <= (*node).x {
            return if (*node).dy > 0 { 1 } else { 0 };
        }
        return if (*node).dy < 0 { 1 } else { 0 };
    }
    if (*node).dy == 0 {
        if y <= (*node).y {
            return if (*node).dx < 0 { 1 } else { 0 };
        }
        return if (*node).dx > 0 { 1 } else { 0 };
    }

    let dx = x.wrapping_sub((*node).x);
    let dy = y.wrapping_sub((*node).y);

    // Try to quickly decide by looking at sign bits.
    if ((((*node).dy ^ (*node).dx ^ dx ^ dy) as u32) & 0x80000000) != 0 {
        if ((((*node).dy ^ dx) as u32) & 0x80000000) != 0 {
            // (left is negative)
            return 1;
        }
        return 0;
    }

    let left = FixedMul((*node).dy>>FRACBITS, dx);
    let right = FixedMul(dy, (*node).dx>>FRACBITS);

    if right < left {
        // front side
        return 0;
    }
    // back side
    return 1;
}


