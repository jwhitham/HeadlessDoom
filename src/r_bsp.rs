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
use crate::r_things::R_AddSprites;
use crate::defs::bbox_t::*;


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

// R_ClipAngles has common code from R_AddLine and R_CheckBBox
// Determine the view X range occupied by two angles
struct R_ClipAngles_return_t {
    x1: i32,
    x2: i32,
}

unsafe fn R_ClipAngles(angle1_param: angle_t, angle2_param: angle_t) -> Option<R_ClipAngles_return_t> {
    let mut angle1 = angle1_param;
    let mut angle2 = angle2_param;
    let span = angle1.wrapping_sub(angle2);
    let mut tspan = angle1.wrapping_add(clipangle);
    if tspan > (2 * clipangle) {
        tspan -= 2 * clipangle;

        // Totally off the left edge?
        if tspan >= span {
            return None;
        }
        
        angle1 = clipangle;
    }
    tspan = clipangle.wrapping_sub(angle2);
    if tspan > (2 * clipangle) {
        tspan -= 2 * clipangle;

        // Totally off the left edge?
        if tspan >= span {
            return None;
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
        return None;
    }
    return Some(R_ClipAngles_return_t { x1: x1, x2: x2 });
}
//
// R_AddLine
// Clips the given segment
// and adds any visible pieces to the line list.
//
unsafe fn R_AddLine (line: *mut seg_t) {
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

    let car = R_ClipAngles(angle1, angle2);
    if car.is_none() {
        return;
    }
    let ca = car.unwrap();
    
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
        R_ClipPassWallSegment (ca.x1, ca.x2-1);	
    } else {
        R_ClipSolidWallSegment (ca.x1, ca.x2-1);
    }
}

//
// R_CheckBBox
// Checks BSP node/subtree bounding box.
// Returns true
//  if some part of the bbox might be visible.
//
const checkcoord: [[i32; 4]; 12] =
[
    [3,0,2,1],
    [3,0,2,0],
    [3,1,2,0],
    [0,0,0,0],
    [2,0,2,1],
    [0,0,0,0],
    [3,1,3,0],
    [0,0,0,0],
    [2,0,3,1],
    [2,1,3,1],
    [2,1,3,0],
    [0,0,0,0],
];


#[no_mangle]
pub unsafe extern "C" fn R_CheckBBox (bspcoord: *mut fixed_t) -> boolean {
    // Find the corners of the box
    // that define the edges from current viewpoint.
    let boxx =
        if viewx <= *bspcoord.offset(BOXLEFT as isize) { 0 }
        else if viewx < *bspcoord.offset(BOXRIGHT as isize) { 1 }
        else { 2 };
    
    let boxy =
        if viewy >= *bspcoord.offset(BOXTOP as isize) { 0 }
        else if viewy > *bspcoord.offset(BOXBOTTOM as isize) { 1 }
        else { 2 };
        
    let boxpos = (boxy<<2)+boxx;
    if boxpos == 5 {
        return c_true;
    }
    
    let x1 = *bspcoord.offset(checkcoord[boxpos][0] as isize);
    let y1 = *bspcoord.offset(checkcoord[boxpos][1] as isize);
    let x2 = *bspcoord.offset(checkcoord[boxpos][2] as isize);
    let y2 = *bspcoord.offset(checkcoord[boxpos][3] as isize);
    
    // check clip list for an open space
    let angle1 = R_PointToAngle (x1, y1).wrapping_sub(viewangle);
    let angle2 = R_PointToAngle (x2, y2).wrapping_sub(viewangle);
	
    let span = angle1.wrapping_sub(angle2);

    // Sitting on a line?
    if span >= ANG180 {
        return c_true;
    }
    
    let car = R_ClipAngles(angle1, angle2);
    if car.is_none() {
        return c_false;
    }
    let ca = car.unwrap();
    let sx1 = ca.x1;
    let mut sx2 = ca.x2;
    sx2 -= 1;
    
    let mut start = solidsegs.as_mut_ptr();
    while (*start).last < sx2 {
        start = start.offset(1);
    }
    
    if (sx1 >= (*start).first)
    && (sx2 <= (*start).last) {
        // The clippost contains the new span.
        return c_false;
    }

    return c_true;
}

//
// R_Subsector
// Determine floor/ceiling planes.
// Add sprites of things in sector.
// Draw one or more line segments.
//
#[no_mangle]
pub unsafe extern "C" fn R_Subsector (num: i32) {
    if num>=numsubsectors {
        panic!("R_Subsector: ss {} with numss = {}",
             num,
             numsubsectors);
    }

    sscount += 1;
    let sub: *mut subsector_t = subsectors.offset(num as isize);
    frontsector = (*sub).sector;
    let count = (*sub).numlines;
    let mut line: *mut seg_t = segs.offset((*sub).firstline as isize);

    if (*frontsector).floorheight < viewz {
        floorplane = R_FindPlane ((*frontsector).floorheight,
                      (*frontsector).floorpic as i32,
                      (*frontsector).lightlevel as i32);
    } else {
        floorplane = std::ptr::null_mut();
    }
        
    if ((*frontsector).ceilingheight > viewz)
    || (((*frontsector).ceilingpic as i32) == skyflatnum) {
        ceilingplane = R_FindPlane ((*frontsector).ceilingheight,
                        (*frontsector).ceilingpic as i32,
                        (*frontsector).lightlevel as i32);
    } else {
        ceilingplane = std::ptr::null_mut();
    }
        
    R_AddSprites (frontsector);	

    for _ in 0 .. count {
        R_AddLine (line);
        line = line.offset(1);
    }
}

