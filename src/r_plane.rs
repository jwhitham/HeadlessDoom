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
use crate::r_bsp::drawsegs_index_t;
use crate::r_data::R_GetColumn;
use crate::r_data::NULL_COLORMAP;
use crate::r_draw::empty_R_DrawColumn_params;
use crate::r_draw::R_DrawColumn_params_t;
use crate::r_draw::empty_R_DrawSpan_params;
use crate::r_draw::R_DrawSpan_params_t;
use crate::r_main::RenderContext_t;
use crate::r_sky::skytexturemid;
use crate::r_things::pspriteiscale;

type visplane_index_t = u16;
pub const INVALID_PLANE: visplane_index_t = visplane_index_t::MAX;
pub type opening_index_t = i16; // may be negative
pub const INVALID_OPENING: opening_index_t = opening_index_t::MIN;
pub const SCREEN_HEIGHT_OPENING: opening_index_t = 0;
const NEGATIVE_ONE_OPENING: opening_index_t = SCREENWIDTH as opening_index_t;
const FIRST_DYNAMIC_OPENING: opening_index_t = (SCREENWIDTH as opening_index_t) * 2;


pub struct visplane_t {
    pub height: fixed_t,
    pub picnum: i32,
    pub lightlevel: i32,
    pub minx: i32,
    pub maxx: i32,
    // top_pad and bottom_pad have 2 extra elements for padding:
    // the index for screen column X is actually X+1
    pub top_pad: [byte; (SCREENWIDTH + 2) as usize],
    pub bottom_pad: [byte; (SCREENWIDTH + 2) as usize],
}

const empty_visplane: visplane_t = visplane_t {
    height: 0,
    picnum: 0,
    lightlevel: 0,
    minx: 0,
    maxx: 0,
    top_pad: [0; (SCREENWIDTH + 2) as usize],
    bottom_pad: [0; (SCREENWIDTH + 2) as usize],
};

pub struct PlaneContext_t {
    planeheight: fixed_t,
    cachedheight: [fixed_t; SCREENHEIGHT as usize],
    cacheddistance: [fixed_t; SCREENHEIGHT as usize],
    cachedystep: [fixed_t; SCREENHEIGHT as usize],
    cachedxstep: [fixed_t; SCREENHEIGHT as usize],
    basexscale: fixed_t,
    baseyscale: fixed_t,
    planezlight_index: usize,
    pub visplanes: [visplane_t; MAXVISPLANES as usize],
    lastvisplane_index: visplane_index_t,
    pub openings: [i16; (MAXOPENINGS as usize) + (FIRST_DYNAMIC_OPENING as usize)],
    spanstart: [i32; SCREENHEIGHT as usize],
    pub ceilingclip: [i16; SCREENWIDTH as usize],
    pub ceilingplane_index: visplane_index_t,
    pub floorclip: [i16; SCREENWIDTH as usize],
    pub floorplane_index: visplane_index_t,
    pub lastopening_index: opening_index_t,
    pub yslope: [fixed_t; SCREENHEIGHT as usize],
    pub distscale: [fixed_t; SCREENWIDTH as usize],
}

pub const empty_PlaneContext: PlaneContext_t = PlaneContext_t {
    planeheight: 0,
    cachedheight: [0; SCREENHEIGHT as usize],
    cacheddistance: [0; SCREENHEIGHT as usize],
    cachedystep: [0; SCREENHEIGHT as usize],
    cachedxstep: [0; SCREENHEIGHT as usize],
    basexscale: 0,
    baseyscale: 0,
    planezlight_index: 0,
    visplanes: [empty_visplane; MAXVISPLANES as usize],
    lastvisplane_index: 0,
    openings: [0; (MAXOPENINGS as usize) + (FIRST_DYNAMIC_OPENING as usize)],
    spanstart: [0; SCREENHEIGHT as usize],
    ceilingclip: [0; SCREENWIDTH as usize],
    ceilingplane_index: INVALID_PLANE,
    floorclip: [0; SCREENWIDTH as usize],
    floorplane_index: INVALID_PLANE,
    lastopening_index: INVALID_OPENING,
    yslope: [0; SCREENHEIGHT as usize],
    distscale: [0; SCREENWIDTH as usize],
};

//
// R_InitPlanes
// Only at game startup.
//
pub fn R_InitPlanes (rc: &mut RenderContext_t) {
    // Doh!
    for i in SCREEN_HEIGHT_OPENING .. NEGATIVE_ONE_OPENING {
        rc.pc.openings[i as usize] = SCREENHEIGHT as i16;
    }
    for i in NEGATIVE_ONE_OPENING .. FIRST_DYNAMIC_OPENING {
        rc.pc.openings[i as usize] = -1;
    }
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
unsafe fn R_MapPlane(rc: &mut RenderContext_t, ds: &mut R_DrawSpan_params_t, y: i32, x1: i32, x2: i32) {
    if (x2 < x1)
    || (x1 < 0)
    || (x2 >= viewwidth)
    || ((y as u32) > (viewheight as u32)) {
        panic!("R_MapPlane: {}, {} at {}",x1,x2,y);
    }

    let distance: fixed_t;

    if rc.pc.planeheight != rc.pc.cachedheight[y as usize] {
        distance = FixedMul (rc.pc.planeheight, rc.pc.yslope[y as usize]);
        ds.ds_xstep = FixedMul (distance,rc.pc.basexscale);
        ds.ds_ystep = FixedMul (distance,rc.pc.baseyscale);
        rc.pc.cachedheight[y as usize] = rc.pc.planeheight;
        rc.pc.cacheddistance[y as usize] = distance;
        rc.pc.cachedxstep[y as usize] = ds.ds_xstep;
        rc.pc.cachedystep[y as usize] = ds.ds_ystep;
    } else {
        distance = rc.pc.cacheddistance[y as usize];
        ds.ds_xstep = rc.pc.cachedxstep[y as usize];
        ds.ds_ystep = rc.pc.cachedystep[y as usize];
    }
    
    let length: fixed_t = FixedMul (distance,rc.pc.distscale[x1 as usize]);
    let angle: angle_t = rc.view.viewangle.wrapping_add(rc.xtoviewangle[x1 as usize])>>ANGLETOFINESHIFT;
    ds.ds_xfrac = rc.view.viewx + FixedMul(*finecosine.offset(angle as isize), length);
    ds.ds_yfrac = -rc.view.viewy - FixedMul(finesine[angle as usize], length);

    if rc.fixedcolormap_index != NULL_COLORMAP {
        ds.ds_colormap_index = rc.fixedcolormap_index;
    } else {
        let index: u32 = u32::min((distance >> LIGHTZSHIFT) as u32, MAXLIGHTZ - 1);
        ds.ds_colormap_index = rc.zlight[
                rc.pc.planezlight_index][index as usize];
    }
    
    ds.ds_y = y;
    ds.ds_x1 = x1;
    ds.ds_x2 = x2;

    // high or low detail
    (rc.spanfunc) (rc, ds);
}

//
// R_ClearPlanes
// At begining of frame.
//
pub unsafe fn R_ClearPlanes (rc: &mut RenderContext_t) {
    // opening / clipping determination
    for i in 0 .. viewwidth as usize {
        rc.pc.floorclip[i] = viewheight as i16;
        rc.pc.ceilingclip[i] = -1;
    }

    rc.pc.lastvisplane_index = 0;
    rc.pc.lastopening_index = FIRST_DYNAMIC_OPENING;
    
    // texture calculation
    rc.pc.cachedheight = [0; SCREENHEIGHT as usize];

    // left to right mapping
    let angle: angle_t = rc.view.viewangle.wrapping_sub(ANG90)>>ANGLETOFINESHIFT;
    
    // scale will be unit scale at SCREENWIDTH/2 distance
    rc.pc.basexscale = FixedDiv (*finecosine.offset(angle as isize),rc.centerxfrac);
    rc.pc.baseyscale = -FixedDiv (finesine[angle as usize],rc.centerxfrac);
}

//
// R_FindPlane
//
pub unsafe fn R_FindPlane(pc: &mut PlaneContext_t,
                          pheight: fixed_t, picnum: i32,
                          plightlevel: i32) -> visplane_index_t {
    
    let mut height = pheight;
    let mut lightlevel = plightlevel;

    if picnum == skyflatnum {
        height = 0;			// all skys map together
        lightlevel = 0;
    }
    
    let mut check: visplane_index_t = 0;
    while check < pc.lastvisplane_index {
        if (height == pc.visplanes.get(check as usize).unwrap().height)
        && (picnum == pc.visplanes.get(check as usize).unwrap().picnum)
        && (lightlevel == pc.visplanes.get(check as usize).unwrap().lightlevel) {
            break;
        }
        check += 1;
    }

    if check < pc.lastvisplane_index {
        return check;
    }
        
    if pc.lastvisplane_index >= (MAXVISPLANES as visplane_index_t) {
        panic!("R_FindPlane: no more visplanes");
    }
        
    pc.lastvisplane_index += 1;

    let mut last = &mut pc.visplanes.get_mut(check as usize).unwrap();
    last.height = height;
    last.picnum = picnum;
    last.lightlevel = lightlevel;
    last.minx = SCREENWIDTH as i32;
    last.maxx = -1;
    last.top_pad = [0xff; (SCREENWIDTH + 2) as usize];
    return check;
}


//
// R_CheckPlane
//
pub unsafe fn R_CheckPlane (pc: &mut PlaneContext_t,
                            ppl_index: visplane_index_t,
                            start: i32, stop: i32) -> visplane_index_t {
    
    let intrl: i32;
    let unionl: i32;
    let pl = pc.visplanes.get_mut(ppl_index as usize).unwrap();

    if start < pl.minx {
        intrl = pl.minx;
        unionl = start;
    } else {
        unionl = pl.minx;
        intrl = start;
    }

    let intrh: i32;
    let unionh: i32;
    
    if stop > pl.maxx {
        intrh = pl.maxx;
        unionh = stop;
    } else {
        unionh = pl.maxx;
        intrh = stop;
    }

    let mut use_same_one = true;
    for x in intrl ..= intrh {
        if pl.top_pad[(x + 1) as usize] != 0xff {
            use_same_one = false;
            break;
        }
    }

    if use_same_one {
        pl.minx = unionl;
        pl.maxx = unionh;

        // use the same one
        return ppl_index;		
    }
    
    if pc.lastvisplane_index == (MAXVISPLANES as visplane_index_t) {
        panic!("R_CheckPlane: no more visplanes");
    }
    // make a new visplane
    let height_copy = pl.height;
    let picnum_copy = pl.picnum;
    let lightlevel_copy = pl.lightlevel;
    {
        let mut npl = pc.visplanes.get_mut(pc.lastvisplane_index as usize).unwrap();
        pc.lastvisplane_index += 1;
        npl.height = height_copy;
        npl.picnum = picnum_copy;
        npl.lightlevel = lightlevel_copy;
        npl.minx = start;
        npl.maxx = stop;
        npl.top_pad = [0xff; (SCREENWIDTH + 2) as usize];
    }
        
    return pc.lastvisplane_index - 1;
}


//
// R_MakeSpans
//
unsafe fn R_MakeSpans(rc: &mut RenderContext_t, ds: &mut R_DrawSpan_params_t, x: i32, pt1: i32, pb1: i32, pt2: i32, pb2: i32) {
    let mut t1 = pt1;
    let mut t2 = pt2;
    let mut b1 = pb1;
    let mut b2 = pb2;

    while (t1 < t2) && (t1<=b1) {
        R_MapPlane (rc,ds,t1,rc.pc.spanstart[t1 as usize],x-1);
        t1 += 1;
    }
    while (b1 > b2) && (b1>=t1) {
        R_MapPlane (rc,ds,b1,rc.pc.spanstart[b1 as usize],x-1);
        b1 -= 1;
    }
	
    while (t2 < t1) && (t2<=b2) {
        rc.pc.spanstart[t2 as usize] = x;
        t2 += 1;
    }
    while (b2 > b1) && (b2>=t2) {
        rc.pc.spanstart[b2 as usize] = x;
        b2 -= 1;
    }
}

//
// R_DrawPlanes
// At the end of each frame.
//
pub unsafe fn R_DrawPlanes (rc: &mut RenderContext_t) {
    if rc.bc.ds_index > (MAXDRAWSEGS as drawsegs_index_t) {
        panic!("R_DrawPlanes: drawsegs overflow");
    }
    
    if rc.pc.lastvisplane_index > (MAXVISPLANES as visplane_index_t) {
        panic!("R_DrawPlanes: visplane overflow");
    }
    
    if rc.pc.lastopening_index > ((MAXOPENINGS as opening_index_t) + FIRST_DYNAMIC_OPENING) {
        panic!("R_DrawPlanes: opening overflow");
    }

    let mut ds: R_DrawSpan_params_t = empty_R_DrawSpan_params;
    for pl_index in 0 .. rc.pc.lastvisplane_index {
        let minx = rc.pc.visplanes[pl_index as usize].minx;
        let maxx = rc.pc.visplanes[pl_index as usize].maxx;
        if minx > maxx {
            continue;
        }
    
        // sky flat
        let picnum = rc.pc.visplanes[pl_index as usize].picnum;
        if picnum == skyflatnum {
            let mut dc: R_DrawColumn_params_t = empty_R_DrawColumn_params;
            dc.dc_iscale = pspriteiscale>>rc.detailshift;
            
            // Sky is allways drawn full bright,
            //  i.e. colormaps[0] is used.
            // Because of this hack, sky is not affected
            //  by INVUL inverse mapping.
            dc.dc_colormap_index = 0;
            dc.dc_texturemid = skytexturemid;
            for x in minx ..= maxx {
                dc.dc_yl = rc.pc.visplanes[pl_index as usize]
                            .top_pad[(x + 1) as usize] as i32;
                dc.dc_yh = rc.pc.visplanes[pl_index as usize]
                            .bottom_pad[(x + 1) as usize] as i32;

                if dc.dc_yl <= dc.dc_yh {
                    let angle = rc.view.viewangle.wrapping_add(rc.xtoviewangle[x as usize])>>ANGLETOSKYSHIFT;
                    dc.dc_x = x;
                    dc.dc_source = R_GetColumn(&mut rc.rd, skytexture, angle as i32);
                    (rc.colfunc) (rc, &mut dc);
                }
            }
            continue;
        }
    
        // regular flat
        ds.ds_source = W_CacheLumpNum(rc.rd.firstflat +
                       *flattranslation.offset(picnum as isize),
                       PU_STATIC) as *mut u8;
        
        let height = rc.pc.visplanes[pl_index as usize].height;
        let lightlevel = rc.pc.visplanes[pl_index as usize].lightlevel;
        rc.pc.planeheight = i32::abs(height-rc.view.viewz);
        let light = i32::max(0, i32::min((LIGHTLEVELS - 1) as i32,
                                 (lightlevel >> LIGHTSEGSHIFT)+rc.extralight));


        rc.pc.planezlight_index = light as usize;

        // top and bottom are arrays of length SCREENWIDTH + 2 (padding)
        rc.pc.visplanes[pl_index as usize]
            .top_pad[(maxx as usize) + 2] = 0xff;
        rc.pc.visplanes[pl_index as usize]
            .top_pad[minx as usize] = 0xff;
            
        for x in minx ..= maxx + 1 {
            R_MakeSpans(rc, &mut ds, x,
                rc.pc.visplanes[pl_index as usize]
                    .top_pad[x as usize] as i32,
                rc.pc.visplanes[pl_index as usize]
                    .bottom_pad[x as usize] as i32,
                rc.pc.visplanes[pl_index as usize]
                    .top_pad[(x as usize) + 1] as i32,
                rc.pc.visplanes[pl_index as usize]
                    .bottom_pad[(x as usize) + 1] as i32);
        }
        
        Z_ChangeTag2(ds.ds_source, PU_CACHE);
    }
}
