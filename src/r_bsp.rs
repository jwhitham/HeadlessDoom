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
use crate::funcs::*;
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
unsafe fn R_ClipSolidWallSegment(first: i32, last: i32) {
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
unsafe fn R_ClipPassWallSegment(first: i32, last: i32) {
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

//
// R_AddLine
// Clips the given segment
// and adds any visible pieces to the line list.
//
#[no_mangle]
pub unsafe extern "C" fn R_AddLine (line: *mut seg_t) {
    curline = line;

    // OPTIMIZE: quickly reject orthogonal back sides.
    let mut angle1 = R_PointToAngle ((*(*line).v1).x, (*(*line).v1).y);
    let mut angle2 = R_PointToAngle ((*(*line).v2).x, (*(*line).v2).y);
    
    // Clip to view edges.
    // OPTIMIZE: make constant out of 2*clipangle (FIELDOFVIEW).
    let span = angle1.wrapping_sub(angle2);
    
    // Back side? I.e. backface culling?
    if span >= ANG180 {
        return;
    }

    // Global angle needed by segcalc.
    rw_angle1 = angle1 as i32;
    angle1 = angle1.wrapping_sub(viewangle);
    angle2 = angle2.wrapping_sub(viewangle);
    
    let mut tspan = angle1.wrapping_add(clipangle);
    if tspan > (2 * clipangle) {
        tspan -= 2 * clipangle;

        // Totally off the left edge?
        if tspan >= span {
            return;
        }
        
        angle1 = clipangle;
    }
    tspan = clipangle.wrapping_sub(angle2);
    if tspan > (2 * clipangle) {
        tspan -= 2 * clipangle;

        // Totally off the left edge?
        if tspan >= span {
            return;
        }
        angle2 = (0 as angle_t).wrapping_sub(clipangle);
    }
    
    // The seg is in the view range,
    // but not necessarily visible.
    angle1 = (angle1.wrapping_add(ANG90))>>ANGLETOFINESHIFT;
    angle2 = (angle2.wrapping_add(ANG90))>>ANGLETOFINESHIFT;
    let x1 = viewangletox[angle1 as usize];
    let x2 = viewangletox[angle2 as usize];

    // Does not cross a pixel?
    if x1 == x2 {
        return;
    }
    
    backsector = (*line).backsector;
    let mut clipsolid = false;

    // Single sided line?
    if backsector == std::ptr::null_mut() {
        clipsolid = true;

    // Closed door.
    } else if ((*backsector).ceilingheight <= (*frontsector).floorheight)
    || ((*backsector).floorheight >= (*frontsector).ceilingheight) {
        clipsolid = true;

    // Window.
    } else if ((*backsector).ceilingheight != (*frontsector).ceilingheight)
    || ((*backsector).floorheight != (*frontsector).floorheight) {
        clipsolid = false;
        
    // Reject empty lines used for triggers
    //  and special events.
    // Identical floor and ceiling on both sides,
    // identical light levels on both sides,
    // and no middle texture.
    } else if ((*backsector).ceilingpic == (*frontsector).ceilingpic)
    && ((*backsector).floorpic == (*frontsector).floorpic)
    && ((*backsector).lightlevel == (*frontsector).lightlevel)
    && ((*(*curline).sidedef).midtexture == 0) {
        return;
    }

    if !clipsolid {
        R_ClipPassWallSegment (x1, x2-1);	
    } else {
        R_ClipSolidWallSegment (x1, x2-1);
    }
}

