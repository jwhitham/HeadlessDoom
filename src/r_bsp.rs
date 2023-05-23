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
use crate::r_segs::R_StoreWallRange;
use crate::r_things::R_AddSprites;
use crate::r_plane::R_FindPlane;
use crate::r_plane::INVALID_PLANE;
use crate::r_main::R_PointToAngle;
use crate::r_main::R_PointOnSide;
use crate::r_main::RenderContext_t;
use crate::r_main::ViewContext_t;
use crate::defs::bbox_t::*;


pub type seg_index_t = u16;

pub struct drawseg_t {
    pub curline: seg_index_t,
    pub x1: i32,
    pub x2: i32,
    pub scale1: fixed_t,
    pub scale2: fixed_t,
    pub scalestep: fixed_t,
    // 0=none, 1=bottom, 2=top, 3=both
    pub silhouette: i32,
    // do not clip sprites above this
    pub bsilheight: fixed_t,
    // do not clip sprites below this
    pub tsilheight: fixed_t,
    // Pointers to lists for sprite clipping,
    //  all three adjusted so [x1] is first value.
    pub sprtopclip: *mut i16,
    pub sprbottomclip: *mut i16,
    pub maskedtexturecol: *mut i16,
}

const empty_drawseg: drawseg_t = drawseg_t {
    curline: 0,
    x1: 0,
    x2: 0,
    scale1: 0,
    scale2: 0,
    scalestep: 0,
    silhouette: 0,
    bsilheight: 0,
    tsilheight: 0,
    sprtopclip: std::ptr::null_mut(),
    sprbottomclip: std::ptr::null_mut(),
    maskedtexturecol: std::ptr::null_mut(),
};

//
// ClipWallSegment
// Clips the given range of columns
// and includes it in the new clip list.
//
#[derive(Debug, Copy, Clone)]
pub struct cliprange_t {
    pub first: i32,
    pub last: i32,
}

const empty_cliprange: cliprange_t = cliprange_t {
    first: 0,
    last: 0,
};

pub type drawsegs_index_t = i16;
pub type solidsegs_index_t = u16;

pub struct BspContext_t {
    pub ds_index: drawsegs_index_t,
    pub drawsegs: [drawseg_t; MAXDRAWSEGS as usize],
    pub curline: seg_index_t,
    pub frontsector: *mut sector_t,
    pub backsector: *mut sector_t,
    pub sidedef: *mut side_t,
    pub linedef: *mut line_t,
    newend: solidsegs_index_t,
    solidsegs: [cliprange_t; MAXSEGS as usize],
}

pub const empty_BspContext: BspContext_t = BspContext_t {
    ds_index: 0,
    drawsegs: [empty_drawseg; MAXDRAWSEGS as usize],
    curline: 0,
    frontsector: std::ptr::null_mut(),
    backsector: std::ptr::null_mut(),
    sidedef: std::ptr::null_mut(),
    linedef: std::ptr::null_mut(),
    newend: 0,
    solidsegs: [empty_cliprange; MAXSEGS as usize],
};

//
// R_ClearDrawSegs
//
pub unsafe fn R_ClearDrawSegs (bc: &mut BspContext_t) {
    bc.ds_index = 0;
}



//
// R_ClipSolidWallSegment
// Does handle solid walls,
//  e.g. single sided LineDefs (middle texture)
//  that entirely block the view.
// 
unsafe fn R_ClipSolidWallSegment(rc: &mut RenderContext_t, first: i32, last: i32) {
    // Find the first range that touches the range
    //  (adjacent pixels are touching).
    let mut start: solidsegs_index_t = 0;
    while rc.bc.solidsegs[start as usize].last < (first - 1) {
        start += 1;
    }

    if first < rc.bc.solidsegs[start as usize].first {
        if last < (rc.bc.solidsegs[start as usize].first - 1) {
            // Post is entirely visible (above start),
            //  so insert a new clippost.
            R_StoreWallRange (rc, first, last);
            let mut next: solidsegs_index_t = rc.bc.newend;
            rc.bc.newend += 1;
            
            while next != start {
                rc.bc.solidsegs[next as usize] = rc.bc.solidsegs[(next - 1) as usize];
                next -= 1;
            }
            rc.bc.solidsegs[next as usize].first = first;
            rc.bc.solidsegs[next as usize].last = last;
            return;
        }
            
        // There is a fragment above *start.
        R_StoreWallRange (rc, first, rc.bc.solidsegs[start as usize].first - 1);
        // Now adjust the clip size.
        rc.bc.solidsegs[start as usize].first = first;
    }

    // Bottom contained in start?
    if last <= rc.bc.solidsegs[start as usize].last {
        return;
    }
        
    let mut next: solidsegs_index_t = start;
    let mut crunch = false;
    while last >= (rc.bc.solidsegs[(next + 1) as usize].first - 1) {
        // There is a fragment between two posts.
        R_StoreWallRange (rc, rc.bc.solidsegs[next as usize].last + 1,
                          rc.bc.solidsegs[(next + 1) as usize].first - 1);
        next += 1;
        
        if last <= rc.bc.solidsegs[next as usize].last {
            // Bottom is contained in next.
            // Adjust the clip size.
            rc.bc.solidsegs[start as usize].last = rc.bc.solidsegs[next as usize].last;
            crunch = true;
            break;
        }
    }
   
    if !crunch {
        // There is a fragment after *next.
        R_StoreWallRange (rc, rc.bc.solidsegs[next as usize].last + 1, last);
        // Adjust the clip size.
        rc.bc.solidsegs[start as usize].last = last;
    }
    
    // Remove start+1 to next from the clip list,
    // because start now covers their area.
    if next == start {
        // Post just extended past the bottom of one post.
        return;
    }
    
    while next != rc.bc.newend {
        next += 1;
        start += 1;
        // Remove a post.
        rc.bc.solidsegs[start as usize] = rc.bc.solidsegs[next as usize];
    }

    rc.bc.newend = start + 1;
}

//
// R_ClipPassWallSegment
// Clips the given range of columns,
//  but does not includes it in the clip list.
// Does handle windows,
//  e.g. LineDefs with upper and lower texture.
//
unsafe fn R_ClipPassWallSegment(rc: &mut RenderContext_t, first: i32, last: i32) {
    // Find the first range that touches the range
    //  (adjacent pixels are touching).
    let mut start: solidsegs_index_t = 0;
    while rc.bc.solidsegs[start as usize].last < (first - 1) {
        start += 1;
    }

    if first < rc.bc.solidsegs[start as usize].first {
        if last < (rc.bc.solidsegs[start as usize].first - 1) {
            // Post is entirely visible (above start).
            R_StoreWallRange (rc, first, last);
            return;
        }
        
        // There is a fragment above *start.
        R_StoreWallRange (rc, first, rc.bc.solidsegs[start as usize].first - 1);
    }

    // Bottom contained in start?
    if last <= rc.bc.solidsegs[start as usize].last {
        return;
    }
        
    while last >= (rc.bc.solidsegs[(start + 1) as usize].first - 1) {
        // There is a fragment between two posts.
        R_StoreWallRange (rc, rc.bc.solidsegs[start as usize].last + 1,
                            rc.bc.solidsegs[(start + 1) as usize].first - 1);
        start += 1;
        
        if last <= rc.bc.solidsegs[start as usize].last {
            return;
        }
    }
    
    // There is a fragment after *next.
    R_StoreWallRange (rc, rc.bc.solidsegs[start as usize].last + 1, last);
}




//
// R_ClearClipSegs
//
pub unsafe fn R_ClearClipSegs (bc: &mut BspContext_t) {
    bc.solidsegs[0].first = -0x7fffffff;
    bc.solidsegs[0].last = -1;
    bc.solidsegs[1].first = viewwidth;
    bc.solidsegs[1].last = 0x7fffffff;
    bc.newend = 2;
}

// R_ClipAngles has common code from R_AddLine and R_CheckBBox
// Determine the view X range occupied by two angles
struct R_ClipAngles_return_t {
    x1: i32,
    x2: i32,
}

unsafe fn R_ClipAngles(view: &mut ViewContext_t, angle1_param: angle_t, angle2_param: angle_t) -> Option<R_ClipAngles_return_t> {
    let mut angle1 = angle1_param;
    let mut angle2 = angle2_param;
    let span = angle1.wrapping_sub(angle2);
    let mut tspan = angle1.wrapping_add(view.clipangle);
    if tspan > (2 * view.clipangle) {
        tspan -= 2 * view.clipangle;

        // Totally off the left edge?
        if tspan >= span {
            return None;
        }
        
        angle1 = view.clipangle;
    }
    tspan = view.clipangle.wrapping_sub(angle2);
    if tspan > (2 * view.clipangle) {
        tspan -= 2 * view.clipangle;

        // Totally off the left edge?
        if tspan >= span {
            return None;
        }
        angle2 = (0 as angle_t).wrapping_sub(view.clipangle);
    }
    
    // The seg is in the view range,
    // but not necessarily visible.
    angle1 = (angle1.wrapping_add(ANG90))>>ANGLETOFINESHIFT;
    angle2 = (angle2.wrapping_add(ANG90))>>ANGLETOFINESHIFT;
    let x1 = view.viewangletox[angle1 as usize];
    let x2 = view.viewangletox[angle2 as usize];

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
unsafe fn R_AddLine (rc: &mut RenderContext_t, line: seg_index_t) {
    rc.bc.curline = line;

    // OPTIMIZE: quickly reject orthogonal back sides.
    let mut angle1 = R_PointToAngle (&mut rc.view,
                                     (*(*segs.offset(line as isize)).v1).x,
                                     (*(*segs.offset(line as isize)).v1).y);
    let mut angle2 = R_PointToAngle (&mut rc.view,
                                     (*(*segs.offset(line as isize)).v2).x,
                                     (*(*segs.offset(line as isize)).v2).y);
    
    // Clip to view edges.
    // OPTIMIZE: make constant out of 2*clipangle (FIELDOFVIEW).
    let span = angle1.wrapping_sub(angle2);
    
    // Back side? I.e. backface culling?
    if span >= ANG180 {
        return;
    }

    // Global angle needed by segcalc.
    rc.sc.rw_angle1 = angle1 as i32;
    angle1 = angle1.wrapping_sub(rc.view.viewangle);
    angle2 = angle2.wrapping_sub(rc.view.viewangle);

    let car = R_ClipAngles(&mut rc.view, angle1, angle2);
    if car.is_none() {
        return;
    }
    let ca = car.unwrap();
    
    rc.bc.backsector = (*segs.offset(line as isize)).backsector;
    let mut clipsolid = false;

    // Single sided line?
    if rc.bc.backsector == std::ptr::null_mut() {
        clipsolid = true;

    // Closed door.
    } else if ((*rc.bc.backsector).ceilingheight <= (*rc.bc.frontsector).floorheight)
    || ((*rc.bc.backsector).floorheight >= (*rc.bc.frontsector).ceilingheight) {
        clipsolid = true;

    // Window.
    } else if ((*rc.bc.backsector).ceilingheight != (*rc.bc.frontsector).ceilingheight)
    || ((*rc.bc.backsector).floorheight != (*rc.bc.frontsector).floorheight) {
        clipsolid = false;
        
    // Reject empty lines used for triggers
    //  and special events.
    // Identical floor and ceiling on both sides,
    // identical light levels on both sides,
    // and no middle texture.
    } else if ((*rc.bc.backsector).ceilingpic == (*rc.bc.frontsector).ceilingpic)
    && ((*rc.bc.backsector).floorpic == (*rc.bc.frontsector).floorpic)
    && ((*rc.bc.backsector).lightlevel == (*rc.bc.frontsector).lightlevel)
    && ((*(*segs.offset(rc.bc.curline as isize)).sidedef).midtexture == 0) {
        return;
    }

    if !clipsolid {
        R_ClipPassWallSegment (rc, ca.x1, ca.x2-1);
    } else {
        R_ClipSolidWallSegment (rc, ca.x1, ca.x2-1);
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


unsafe fn R_CheckBBox (rc: &mut RenderContext_t, bspcoord: *mut fixed_t) -> boolean {
    // Find the corners of the box
    // that define the edges from current viewpoint.
    let boxx =
        if rc.view.viewx <= *bspcoord.offset(BOXLEFT as isize) { 0 }
        else if rc.view.viewx < *bspcoord.offset(BOXRIGHT as isize) { 1 }
        else { 2 };
    
    let boxy =
        if rc.view.viewy >= *bspcoord.offset(BOXTOP as isize) { 0 }
        else if rc.view.viewy > *bspcoord.offset(BOXBOTTOM as isize) { 1 }
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
    let angle1 = R_PointToAngle (&mut rc.view, x1, y1).wrapping_sub(rc.view.viewangle);
    let angle2 = R_PointToAngle (&mut rc.view, x2, y2).wrapping_sub(rc.view.viewangle);

    let span = angle1.wrapping_sub(angle2);

    // Sitting on a line?
    if span >= ANG180 {
        return c_true;
    }
    
    let car = R_ClipAngles(&mut rc.view, angle1, angle2);
    if car.is_none() {
        return c_false;
    }
    let ca = car.unwrap();
    let sx1 = ca.x1;
    let mut sx2 = ca.x2;
    sx2 -= 1;
    
    let mut start: solidsegs_index_t = 0;
    while rc.bc.solidsegs[start as usize].last < sx2 {
        start += 1;
    }
    
    if (sx1 >= rc.bc.solidsegs[start as usize].first)
    && (sx2 <= rc.bc.solidsegs[start as usize].last) {
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
unsafe fn R_Subsector (rc: &mut RenderContext_t, num: i32) {
    if num>=numsubsectors {
        panic!("R_Subsector: ss {} with numss = {}",
             num,
             numsubsectors);
    }

    rc.sscount += 1;
    let sub: *mut subsector_t = subsectors.offset(num as isize);
    rc.bc.frontsector = (*sub).sector;
    let count = (*sub).numlines;
    let mut line: seg_index_t = (*sub).firstline as seg_index_t;

    if (*rc.bc.frontsector).floorheight < rc.view.viewz {
        rc.pc.floorplane_index = R_FindPlane (&mut rc.pc,
                        (*rc.bc.frontsector).floorheight,
                        (*rc.bc.frontsector).floorpic as i32,
                        (*rc.bc.frontsector).lightlevel as i32);
    } else {
        rc.pc.floorplane_index = INVALID_PLANE;
    }
        
    if ((*rc.bc.frontsector).ceilingheight > rc.view.viewz)
    || (((*rc.bc.frontsector).ceilingpic as i32) == skyflatnum) {
        rc.pc.ceilingplane_index = R_FindPlane (&mut rc.pc,
                        (*rc.bc.frontsector).ceilingheight,
                        (*rc.bc.frontsector).ceilingpic as i32,
                        (*rc.bc.frontsector).lightlevel as i32);
    } else {
        rc.pc.ceilingplane_index = INVALID_PLANE;
    }
        
    R_AddSprites (rc, rc.bc.frontsector);

    for _ in 0 .. count {
        R_AddLine (rc, line);
        line += 1;
    }
}

//
// RenderBSPNode
// Renders all subsectors below a given node,
//  traversing subtree recursively.
// Just call with BSP root.
pub unsafe fn R_RenderBSPNode (rc: &mut RenderContext_t, bspnum: i32) {
    // Found a subsector?
    if (bspnum & NF_SUBSECTOR as i32) != 0 {
        if bspnum == -1 {
            R_Subsector (rc, 0);
        } else {
            R_Subsector (rc, bspnum & (!NF_SUBSECTOR as i32));
        }
        return;
    }
        
    let bsp: *mut node_t = nodes.offset(bspnum as isize);
    
    // Decide which side the view point is on.
    let side: i32 = R_PointOnSide (rc.view.viewx, rc.view.viewy, bsp);

    // Recursively divide front space.
    R_RenderBSPNode (rc, (*bsp).children[side as usize] as i32); 

    // Possibly divide back space.
    if R_CheckBBox (rc, (*bsp).bbox[(side^1) as usize].as_mut_ptr()) != 0 {
        R_RenderBSPNode (rc, (*bsp).children[(side^1) as usize] as i32);
    }
}


