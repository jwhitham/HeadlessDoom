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
use crate::funcs::*;
use crate::m_fixed::FixedMul;
use crate::m_fixed::FixedDiv;
use crate::tables::finesine;
use crate::r_data::R_GetColumn;
use crate::r_bsp::ds_p;
use crate::r_bsp::drawsegs;
use crate::r_data::colormaps;
use crate::r_data::firstflat;
use crate::r_draw::empty_R_DrawColumn_params;
use crate::r_draw::R_DrawColumn_params_t;
use crate::r_draw::empty_R_DrawSpan_params;
use crate::r_draw::R_DrawSpan_params_t;
use crate::r_draw::VideoContext_t;
use crate::r_main::fixedcolormap;
use crate::r_main::spanfunc;
use crate::r_main::centerxfrac;
use crate::r_main::detailshift;
use crate::r_main::viewangle;
use crate::r_main::xtoviewangle;
use crate::r_main::colfunc;
use crate::r_main::extralight;
use crate::r_main::zlight;
use crate::r_main::viewx;
use crate::r_main::viewy;
use crate::r_main::viewz;
use crate::r_sky::skytexturemid;
use crate::r_things::pspriteiscale;

const empty_visplane: visplane_t = visplane_t {
  height: 0,
  picnum: 0,
  lightlevel: 0,
  minx: 0,
  maxx: 0,
  pad1: 0,
  top: [0; SCREENWIDTH as usize],
  pad2: 0,
  pad3: 0,
  bottom: [0; SCREENWIDTH as usize],
  pad4: 0,
};

static mut planeheight: fixed_t = 0;
static mut cachedheight: [fixed_t; SCREENHEIGHT as usize] = [0; SCREENHEIGHT as usize];
static mut cacheddistance: [fixed_t; SCREENHEIGHT as usize] = [0; SCREENHEIGHT as usize];
static mut cachedystep: [fixed_t; SCREENHEIGHT as usize] = [0; SCREENHEIGHT as usize];
static mut cachedxstep: [fixed_t; SCREENHEIGHT as usize] = [0; SCREENHEIGHT as usize];
static mut basexscale: fixed_t = 0;
static mut baseyscale: fixed_t = 0;
static mut planezlight: *mut *mut lighttable_t = std::ptr::null_mut();
static mut visplanes: [visplane_t; MAXVISPLANES as usize] = [empty_visplane; MAXVISPLANES as usize];
static mut lastvisplane: *mut visplane_t = std::ptr::null_mut();
static mut openings: [i16; MAXOPENINGS as usize] = [0; MAXOPENINGS as usize];
static mut spanstart: [i32; SCREENHEIGHT as usize] = [0; SCREENHEIGHT as usize];
pub static mut ceilingclip: [i16; SCREENWIDTH as usize] = [0; SCREENWIDTH as usize];
pub static mut ceilingplane: *mut visplane_t = std::ptr::null_mut();
pub static mut floorclip: [i16; SCREENWIDTH as usize] = [0; SCREENWIDTH as usize];
pub static mut floorplane: *mut visplane_t = std::ptr::null_mut();
pub static mut lastopening: *mut i16 = std::ptr::null_mut();
pub static mut yslope: [fixed_t; SCREENHEIGHT as usize] = [0; SCREENHEIGHT as usize];
pub static mut distscale: [fixed_t; SCREENWIDTH as usize] = [0; SCREENWIDTH as usize];


//
// R_InitPlanes
// Only at game startup.
//
pub fn R_InitPlanes () {
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
unsafe fn R_MapPlane(vc: &mut VideoContext_t, ds: &mut R_DrawSpan_params_t, y: i32, x1: i32, x2: i32) {
    if (x2 < x1)
    || (x1 < 0)
    || (x2 >= viewwidth)
    || ((y as u32) > (viewheight as u32)) {
        panic!("R_MapPlane: {}, {} at {}",x1,x2,y);
    }

    let distance: fixed_t;

    if planeheight != cachedheight[y as usize] {
        distance = FixedMul (planeheight, yslope[y as usize]);
        ds.ds_xstep = FixedMul (distance,basexscale);
        ds.ds_ystep = FixedMul (distance,baseyscale);
        cachedheight[y as usize] = planeheight;
        cacheddistance[y as usize] = distance;
        cachedxstep[y as usize] = ds.ds_xstep;
        cachedystep[y as usize] = ds.ds_ystep;
    } else {
        distance = cacheddistance[y as usize];
        ds.ds_xstep = cachedxstep[y as usize];
        ds.ds_ystep = cachedystep[y as usize];
    }
    
    let length: fixed_t = FixedMul (distance,distscale[x1 as usize]);
    let angle: angle_t = viewangle.wrapping_add(xtoviewangle[x1 as usize])>>ANGLETOFINESHIFT;
    ds.ds_xfrac = viewx + FixedMul(*finecosine.offset(angle as isize), length);
    ds.ds_yfrac = -viewy - FixedMul(finesine[angle as usize], length);

    if fixedcolormap != std::ptr::null_mut() {
        ds.ds_colormap = fixedcolormap;
    } else {
        let index: u32 = u32::min((distance >> LIGHTZSHIFT) as u32, MAXLIGHTZ - 1);
        ds.ds_colormap = *planezlight.offset(index as isize);
    }
    
    ds.ds_y = y;
    ds.ds_x1 = x1;
    ds.ds_x2 = x2;

    // high or low detail
    spanfunc (vc, ds);
}

//
// R_ClearPlanes
// At begining of frame.
//
pub unsafe fn R_ClearPlanes () {
    // opening / clipping determination
    for i in 0 .. viewwidth as usize {
        floorclip[i] = viewheight as i16;
        ceilingclip[i] = -1;
    }

    lastvisplane = visplanes.as_mut_ptr();
    lastopening = openings.as_mut_ptr();
    
    // texture calculation
    cachedheight = [0; SCREENHEIGHT as usize];

    // left to right mapping
    let angle: angle_t = viewangle.wrapping_sub(ANG90)>>ANGLETOFINESHIFT;
    
    // scale will be unit scale at SCREENWIDTH/2 distance
    basexscale = FixedDiv (*finecosine.offset(angle as isize),centerxfrac);
    baseyscale = -FixedDiv (finesine[angle as usize],centerxfrac);
}

//
// R_FindPlane
//
pub unsafe fn R_FindPlane(pheight: fixed_t, picnum: i32, plightlevel: i32) -> *mut visplane_t {
    
    let mut height = pheight;
    let mut lightlevel = plightlevel;

    if picnum == skyflatnum {
        height = 0;			// all skys map together
        lightlevel = 0;
    }
    
    let mut check: *mut visplane_t = visplanes.as_mut_ptr();
    while check < lastvisplane {
        if (height == (*check).height)
        && (picnum == (*check).picnum)
        && (lightlevel == (*check).lightlevel) {
            break;
        }
        check = check.offset(1);
    }

    if check < lastvisplane {
        return check;
    }
        
    if lastvisplane == visplanes.as_mut_ptr().offset(MAXVISPLANES as isize) {
        panic!("R_FindPlane: no more visplanes");
    }
        
    lastvisplane = lastvisplane.offset(1);

    (*check).height = height;
    (*check).picnum = picnum;
    (*check).lightlevel = lightlevel;
    (*check).minx = SCREENWIDTH as i32;
    (*check).maxx = -1;
    (*check).top = [0xff; SCREENWIDTH as usize];
    return check;
}


//
// R_CheckPlane
//
pub unsafe fn R_CheckPlane (ppl: *mut visplane_t, start: i32, stop: i32) -> *mut visplane_t {
    
    let intrl: i32;
    let unionl: i32;
    let mut pl = ppl;

    if start < (*pl).minx {
        intrl = (*pl).minx;
        unionl = start;
    } else {
        unionl = (*pl).minx;
        intrl = start;
    }

    let intrh: i32;
    let unionh: i32;
    
    if stop > (*pl).maxx {
        intrh = (*pl).maxx;
        unionh = stop;
    } else {
        unionh = (*pl).maxx;
        intrh = stop;
    }

    let mut use_same_one = true;
    for x in intrl ..= intrh {
        if (*pl).top[x as usize] != 0xff {
            use_same_one = false;
            break;
        }
    }

    if use_same_one {
        (*pl).minx = unionl;
        (*pl).maxx = unionh;

        // use the same one
        return pl;		
    }
    
    if lastvisplane == visplanes.as_mut_ptr().offset(MAXVISPLANES as isize) {
        panic!("R_CheckPlane: no more visplanes");
    }
    // make a new visplane
    (*lastvisplane).height = (*pl).height;
    (*lastvisplane).picnum = (*pl).picnum;
    (*lastvisplane).lightlevel = (*pl).lightlevel;
   
    pl = lastvisplane;
    lastvisplane = lastvisplane.offset(1);
    (*pl).minx = start;
    (*pl).maxx = stop;

    (*pl).top = [0xff; SCREENWIDTH as usize];
        
    return pl;
}


//
// R_MakeSpans
//
unsafe fn R_MakeSpans(vc: &mut VideoContext_t, ds: &mut R_DrawSpan_params_t, x: i32, pt1: i32, pb1: i32, pt2: i32, pb2: i32) {
    let mut t1 = pt1;
    let mut t2 = pt2;
    let mut b1 = pb1;
    let mut b2 = pb2;

    while (t1 < t2) && (t1<=b1) {
        R_MapPlane (vc,ds,t1,spanstart[t1 as usize],x-1);
        t1 += 1;
    }
    while (b1 > b2) && (b1>=t1) {
        R_MapPlane (vc,ds,b1,spanstart[b1 as usize],x-1);
        b1 -= 1;
    }
	
    while (t2 < t1) && (t2<=b2) {
        spanstart[t2 as usize] = x;
        t2 += 1;
    }
    while (b2 > b1) && (b2>=t2) {
        spanstart[b2 as usize] = x;
        b2 -= 1;
    }
}

//
// R_DrawPlanes
// At the end of each frame.
//
pub unsafe fn R_DrawPlanes (vc: &mut VideoContext_t) {
    if ds_p > drawsegs.as_mut_ptr().offset(MAXDRAWSEGS as isize) {
        panic!("R_DrawPlanes: drawsegs overflow");
    }
    
    if lastvisplane > visplanes.as_mut_ptr().offset(MAXVISPLANES as isize) {
        panic!("R_DrawPlanes: visplane overflow");
    }
    
    if lastopening > openings.as_mut_ptr().offset(MAXOPENINGS as isize) {
        panic!("R_DrawPlanes: opening overflow");
    }

    let mut ds: R_DrawSpan_params_t = empty_R_DrawSpan_params;
    let mut pl = visplanes.as_mut_ptr().offset(-1);
    loop {
        pl = pl.offset(1);
        if pl >= lastvisplane {
            break;
        }
        if (*pl).minx > (*pl).maxx {
            continue;
        }
    
        // sky flat
        if (*pl).picnum == skyflatnum {
            let mut dc: R_DrawColumn_params_t = empty_R_DrawColumn_params;
            dc.dc_iscale = pspriteiscale>>detailshift;
            
            // Sky is allways drawn full bright,
            //  i.e. colormaps[0] is used.
            // Because of this hack, sky is not affected
            //  by INVUL inverse mapping.
            dc.dc_colormap = colormaps;
            dc.dc_texturemid = skytexturemid;
            for x in (*pl).minx ..= (*pl).maxx {
                dc.dc_yl = (*pl).top[x as usize] as i32;
                dc.dc_yh = (*pl).bottom[x as usize] as i32;

                if dc.dc_yl <= dc.dc_yh {
                    let angle = viewangle.wrapping_add(xtoviewangle[x as usize])>>ANGLETOSKYSHIFT;
                    dc.dc_x = x;
                    dc.dc_source = R_GetColumn(skytexture, angle as i32);
                    colfunc (vc, &mut dc);
                }
            }
            continue;
        }
    
        // regular flat
        ds.ds_source = W_CacheLumpNum(firstflat +
                       *flattranslation.offset((*pl).picnum as isize),
                       PU_STATIC) as *mut u8;
        
        planeheight = i32::abs((*pl).height-viewz);
        let light = i32::max(0, i32::min((LIGHTLEVELS - 1) as i32,
                                 ((*pl).lightlevel >> LIGHTSEGSHIFT)+extralight));


        planezlight = zlight[light as usize].as_mut_ptr();

        // top and bottom are arrays but indexes of -1 and SCREENWIDTH need to be valid
        *(*pl).top.as_mut_ptr().offset(((*pl).maxx as isize) + 1) = 0xff;
        *(*pl).top.as_mut_ptr().offset(((*pl).minx as isize) - 1) = 0xff;
            
        for x in (*pl).minx ..= (*pl).maxx + 1 {
            R_MakeSpans(vc, &mut ds, x,
                *(*pl).top.as_ptr().offset((x as isize) - 1) as i32,
                *(*pl).bottom.as_ptr().offset((x as isize) - 1) as i32,
                *(*pl).top.as_ptr().offset(x as isize) as i32,
                *(*pl).bottom.as_ptr().offset(x as isize) as i32);
        }
        
        Z_ChangeTag2(ds.ds_source, PU_CACHE);
    }
}
