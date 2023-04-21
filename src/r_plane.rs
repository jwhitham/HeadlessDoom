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
//	Here is a core component: drawing the floors and ceilings,
//	 while maintaining a per column clipping list only.
//	Moreover, the sky areas have to be determined.
//
//-----------------------------------------------------------------------------

use crate::defs::*;
use crate::globals::*;
// use crate::funcs::*;
use crate::m_fixed::FixedMul;
use crate::tables::finesine;


//
// R_InitPlanes
// Only at game startup.
//
#[no_mangle]
pub extern "C" fn R_InitPlanes () {
    // Doh!
}


//
// R_MapPlane
//
// Uses global vars:
//  planeheight
//  ds_source
//  basexscale
//  baseyscale
//  viewx
//  viewy
//
// BASIC PRIMITIVE
//
#[no_mangle]
pub unsafe extern "C" fn R_MapPlane(y: i32, x1: i32, x2: i32) {
    if (x2 < x1)
    || (x1 < 0)
    || (x2 >= viewwidth)
    || ((y as u32) > (viewheight as u32)) {
        panic!("R_MapPlane: {}, {} at {}",x1,x2,y);
    }

    let distance: fixed_t;

    if planeheight != cachedheight[y as usize] {
        distance = FixedMul (planeheight, yslope[y as usize]);
        ds_xstep = FixedMul (distance,basexscale);
        ds_ystep = FixedMul (distance,baseyscale);
        cachedheight[y as usize] = planeheight;
        cacheddistance[y as usize] = distance;
        cachedxstep[y as usize] = ds_xstep;
        cachedystep[y as usize] = ds_ystep;
    } else {
        distance = cacheddistance[y as usize];
        ds_xstep = cachedxstep[y as usize];
        ds_ystep = cachedystep[y as usize];
    }
    
    let length: fixed_t = FixedMul (distance,distscale[x1 as usize]);
    let angle: angle_t = viewangle.wrapping_add(xtoviewangle[x1 as usize])>>ANGLETOFINESHIFT;
    ds_xfrac = viewx + FixedMul(*finecosine.offset(angle as isize), length);
    ds_yfrac = -viewy - FixedMul(finesine[angle as usize], length);

    if fixedcolormap != std::ptr::null_mut() {
        ds_colormap = fixedcolormap;
    } else {
        let index: u32 = u32::min((distance >> LIGHTZSHIFT) as u32, MAXLIGHTZ - 1);
        ds_colormap = *planezlight.offset(index as isize);
    }
    
    ds_y = y;
    ds_x1 = x1;
    ds_x2 = x2;

    // high or low detail
    spanfunc ();	
}

