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
//	BSP traversal, handling of LineSegs for rendering.
//
//-----------------------------------------------------------------------------



use crate::defs::*;
use crate::globals::*;
//use crate::funcs::*;
use crate::r_segs::R_StoreWallRange;


//
// R_ClearDrawSegs
//
#[no_mangle]
pub unsafe extern "C" fn R_ClearDrawSegs () {
    ds_p = drawsegs.as_mut_ptr();
}



//
// R_ClipSolidWallSegment
// Does handle solid walls,
//  e.g. single sided LineDefs (middle texture)
//  that entirely block the view.
// 
#[no_mangle]
pub unsafe extern "C" fn R_ClipSolidWallSegment(first: i32, last: i32) {
    // Find the first range that touches the range
    //  (adjacent pixels are touching).
    let mut start: *mut cliprange_t = solidsegs.as_mut_ptr();
    while (*start).last < (first - 1) {
        start = start.offset(1);
    }

    if first < (*start).first {
        if last < ((*start).first - 1) {
            // Post is entirely visible (above start),
            //  so insert a new clippost.
            R_StoreWallRange (first, last);
            let mut next: *mut cliprange_t = newend;
            newend = newend.offset(1);
            
            while next != start {
                *next = *(next.offset(-1));
                next = next.offset(-1);
            }
            (*next).first = first;
            (*next).last = last;
            return;
        }
            
        // There is a fragment above *start.
        R_StoreWallRange (first, (*start).first - 1);
        // Now adjust the clip size.
        (*start).first = first;	
    }

    // Bottom contained in start?
    if last <= (*start).last {
        return;
    }
        
    let mut next: *mut cliprange_t = start;
    let mut crunch = false;
    while last >= ((*next.offset(1)).first - 1) {
        // There is a fragment between two posts.
        R_StoreWallRange ((*next).last + 1, (*next.offset(1)).first - 1);
        next = next.offset(1);
        
        if last <= (*next).last {
            // Bottom is contained in next.
            // Adjust the clip size.
            (*start).last = (*next).last;	
            crunch = true;
            break;
        }
    }
   
    if !crunch {
        // There is a fragment after *next.
        R_StoreWallRange ((*next).last + 1, last);
        // Adjust the clip size.
        (*start).last = last;
    }
    
    // Remove start+1 to next from the clip list,
    // because start now covers their area.
    if next == start {
        // Post just extended past the bottom of one post.
        return;
    }
    
    while next != newend {
        next = next.offset(1);
        start = start.offset(1);
        // Remove a post.
        *start = *next;
    }

    newend = start.offset(1);
}

//
// R_ClipPassWallSegment
// Clips the given range of columns,
//  but does not includes it in the clip list.
// Does handle windows,
//  e.g. LineDefs with upper and lower texture.
//
#[no_mangle]
pub unsafe extern "C" fn R_ClipPassWallSegment(first: i32, last: i32) {
    // Find the first range that touches the range
    //  (adjacent pixels are touching).
    let mut start: *mut cliprange_t = solidsegs.as_mut_ptr();
    while (*start).last < (first - 1) {
        start = start.offset(1);
    }

    if first < (*start).first {
        if last < ((*start).first - 1) {
            // Post is entirely visible (above start).
            R_StoreWallRange (first, last);
            return;
        }
        
        // There is a fragment above *start.
        R_StoreWallRange (first, (*start).first - 1);
    }

    // Bottom contained in start?
    if last <= (*start).last {
        return;
    }
        
    while last >= ((*start.offset(1)).first - 1) {
        // There is a fragment between two posts.
        R_StoreWallRange ((*start).last + 1, (*start.offset(1)).first - 1);
        start = start.offset(1);
        
        if last <= (*start).last {
            return;
        }
    }
    
    // There is a fragment after *next.
    R_StoreWallRange ((*start).last + 1, last);
}




//
// R_ClearClipSegs
//
#[no_mangle]
pub unsafe extern "C" fn R_ClearClipSegs () {
    solidsegs[0].first = -0x7fffffff;
    solidsegs[0].last = -1;
    solidsegs[1].first = viewwidth;
    solidsegs[1].last = 0x7fffffff;
    newend = solidsegs.as_mut_ptr().offset(2);
}

