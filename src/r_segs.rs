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
//	All the clipping: columns, horizontal spans, sky columns.
//
//-----------------------------------------------------------------------------

use crate::defs::*;
use crate::globals::*;
use crate::r_things;
use crate::tables::finetangent;
use crate::tables::finesine;
use crate::m_fixed::FixedMul;
use crate::r_bsp::drawsegs_index_t;
use crate::r_data::R_GetColumn;
use crate::r_data::NULL_COLORMAP;
use crate::r_data::colormap_index_t;
use crate::r_main::R_PointToDist;
use crate::r_main::R_ScaleFromGlobalAngle;
use crate::r_main::RenderContext_t;
use crate::r_plane::R_CheckPlane;
use crate::r_plane::opening_index_t;
use crate::r_plane::INVALID_OPENING;
use crate::r_plane::SCREEN_HEIGHT_OPENING;
use crate::r_things::negonearray;
use crate::r_draw::empty_R_DrawColumn_params;
use crate::r_draw::R_DrawColumn_params_t;

pub struct SegsContext_t {
    markceiling: boolean,
    markfloor: boolean,
    segtextured: boolean,
    pub rw_distance: fixed_t,
    midtexture: i32,
    toptexture: i32,
    bottomtexture: i32,
    pub rw_normalangle: angle_t,
    pub rw_angle1: i32,
    pub walllights: *mut colormap_index_t,
    pub maskedtexturecol_index: opening_index_t,
}

pub const empty_SegsContext: SegsContext_t = SegsContext_t {
    markceiling: c_false,
    markfloor: c_false,
    segtextured: c_false,
    rw_distance: 0,
    midtexture: 0,
    toptexture: 0,
    bottomtexture: 0,
    rw_normalangle: 0,
    rw_angle1: 0,
    walllights: std::ptr::null_mut(),
    maskedtexturecol_index: INVALID_OPENING,
};

struct R_RenderSegLoop_params_t {
    dc: R_DrawColumn_params_t,
    bottomfrac: fixed_t,
    bottomstep: fixed_t,
    maskedtexture: boolean,
    pixhigh: fixed_t,
    pixhighstep: fixed_t,
    pixlow: fixed_t,
    pixlowstep: fixed_t,
    rw_bottomtexturemid: fixed_t,
    rw_centerangle: angle_t,
    rw_midtexturemid: fixed_t,
    rw_offset: fixed_t,
    rw_scale: fixed_t,
    rw_scalestep: fixed_t,
    rw_toptexturemid: fixed_t,
    rw_x: i32,
    rw_stopx: i32,
    topfrac: fixed_t,
    topstep: fixed_t,
}

//
//
// R_RenderMaskedSegRange
//
pub unsafe fn R_RenderMaskedSegRange
        (rc: &mut RenderContext_t, ds: drawsegs_index_t, x1: i32, x2: i32) {
    // Calculate light table.
    // Use different light tables
    //   for horizontal / vertical / diagonal. Diagonal?
    // OPTIMIZE: get rid of LIGHTSEGSHIFT globally
    rc.bc.curline = rc.bc.drawsegs[ds as usize].curline;
    rc.bc.frontsector = (*segs.offset(rc.bc.curline as isize)).frontsector;
    rc.bc.backsector = (*segs.offset(rc.bc.curline as isize)).backsector;
    let texnum = *texturetranslation.offset(
        (*(*segs.offset(rc.bc.curline as isize)).sidedef).midtexture as isize);
    
    let mut lightnum = (((*rc.bc.frontsector).lightlevel >> LIGHTSEGSHIFT) as i32)
                    + rc.extralight;

    if (*(*segs.offset(rc.bc.curline as isize)).v1).y == (*(*segs.offset(rc.bc.curline as isize)).v2).y {
        lightnum -= 1;
    } else if (*(*segs.offset(rc.bc.curline as isize)).v1).x == (*(*segs.offset(rc.bc.curline as isize)).v2).x {
        lightnum += 1;
    }

    rc.sc.walllights = rc.scalelight[i32::max(0,
                            i32::min((LIGHTLEVELS - 1) as i32, lightnum)) as usize].as_mut_ptr();

    rc.sc.maskedtexturecol_index = rc.bc.drawsegs[ds as usize].maskedtexturecol_index;

    let rw_scalestep = rc.bc.drawsegs[ds as usize].scalestep;
    let mut dmc = r_things::R_DrawMaskedColumn_params_t {
        dc: empty_R_DrawColumn_params,
        column: std::ptr::null_mut(),
        sprtopscreen: 0,
        spryscale: rc.bc.drawsegs[ds as usize].scale1 + (x1 - rc.bc.drawsegs[ds as usize].x1)*rw_scalestep,
        mfloorclip: rc.bc.drawsegs[ds as usize].sprbottomclip,
        mceilingclip: rc.pc.openings.as_mut_ptr().offset(
                    rc.bc.drawsegs[ds as usize].sprtopclip_index as isize),
    };
    
    // find positioning
    if (((*(*segs.offset(rc.bc.curline as isize)).linedef).flags as u32) & ML_DONTPEGBOTTOM) != 0 {
        dmc.dc.dc_texturemid =
            if (*rc.bc.frontsector).floorheight > (*rc.bc.backsector).floorheight {
                (*rc.bc.frontsector).floorheight
            } else {
                (*rc.bc.backsector).floorheight
            };
        dmc.dc.dc_texturemid = dmc.dc.dc_texturemid +
                *textureheight.offset(texnum as isize) - rc.view.viewz;
    } else {
        dmc.dc.dc_texturemid =
            if (*rc.bc.frontsector).ceilingheight < (*rc.bc.backsector).ceilingheight {
                (*rc.bc.frontsector).ceilingheight
            } else {
                (*rc.bc.backsector).ceilingheight
            };
        dmc.dc.dc_texturemid = dmc.dc.dc_texturemid - rc.view.viewz;
    }
    dmc.dc.dc_texturemid += (*(*segs.offset(rc.bc.curline as isize)).sidedef).rowoffset;
            
    if rc.fixedcolormap_index != NULL_COLORMAP {
        dmc.dc.dc_colormap_index = rc.fixedcolormap_index;
    }
    
    // draw the columns
    for x in x1 ..= x2 {
        dmc.dc.dc_x = x;
        // calculate lighting
        let colnum = rc.pc.openings[
                (rc.sc.maskedtexturecol_index + 
                    (dmc.dc.dc_x as opening_index_t)) as usize];
        if colnum != MAXSHORT {
            if rc.fixedcolormap_index == NULL_COLORMAP {
                let index = i32::min((MAXLIGHTSCALE - 1) as i32,
                                    dmc.spryscale>>LIGHTSCALESHIFT);
                dmc.dc.dc_colormap_index = *rc.sc.walllights.offset(index as isize);
            }
                
            dmc.sprtopscreen = rc.centeryfrac - FixedMul(dmc.dc.dc_texturemid, dmc.spryscale);
            dmc.dc.dc_iscale = ((0xffffffff as u32) / (dmc.spryscale as u32)) as i32;
            
            // draw the texture
            dmc.column = (R_GetColumn(&mut rc.rd, texnum, colnum as i32)
                            as *mut u8).offset(-3) as *mut column_t;
                
            r_things::R_DrawMaskedColumn (rc, &mut dmc);
            rc.pc.openings[
                (rc.sc.maskedtexturecol_index + 
                    (dmc.dc.dc_x as opening_index_t)) as usize] = MAXSHORT;
        }
        dmc.spryscale += rw_scalestep;
    }
}

//
// R_RenderSegLoop
// Draws zero, one, or two textures (and possibly a masked
//  texture) for walls.
// Can draw or mark the starting pixel of floor and ceiling
//  textures.
// CALLED: CORE LOOPING ROUTINE.
//
const HEIGHTBITS: i32 = 12;
const HEIGHTUNIT: i32 = 1<<HEIGHTBITS;

unsafe fn R_RenderSegLoop (rc: &mut RenderContext_t, rsl: &mut R_RenderSegLoop_params_t) {
    let mut texturecolumn: fixed_t = 0;
    for x in rsl.rw_x as usize .. rsl.rw_stopx as usize {
        // mark floor / ceiling areas
        let yl = i32::max((rsl.topfrac+HEIGHTUNIT-1)>>HEIGHTBITS,
                          (rc.pc.ceilingclip[x]+1) as i32);
        
        if rc.sc.markceiling != c_false {
            let top = (rc.pc.ceilingclip[x]+1) as i32;
            let bottom = i32::min(yl-1, (rc.pc.floorclip[x]-1) as i32);

            if top <= bottom {
                rc.pc.visplanes[rc.pc.ceilingplane_index as usize]
                        .top_pad[x + 1] = top as u8;
                rc.pc.visplanes[rc.pc.ceilingplane_index as usize]
                        .bottom_pad[x + 1] = bottom as u8;
            }
        }
            
        let yh = i32::min(rsl.bottomfrac>>HEIGHTBITS, (rc.pc.floorclip[x]-1) as i32);

        if rc.sc.markfloor != c_false {
            let top = i32::max(yh+1, (rc.pc.ceilingclip[x]+1) as i32);
            let bottom = (rc.pc.floorclip[x]-1) as i32;
            if top <= bottom {
                rc.pc.visplanes[rc.pc.floorplane_index as usize]
                        .top_pad[x + 1] = top as u8;
                rc.pc.visplanes[rc.pc.floorplane_index as usize]
                        .bottom_pad[x + 1] = bottom as u8;
            }
        }
        
        // texturecolumn and lighting are independent of wall tiers
        if rc.sc.segtextured != c_false {
            // calculate texture offset
            let mut angle = rsl.rw_centerangle.wrapping_add(rc.xtoviewangle[x])>>ANGLETOFINESHIFT;

            if angle >= (FINEANGLES / 2) { // DSB-23
                angle = 0;
            }

            texturecolumn = rsl.rw_offset - FixedMul
                (finetangent[angle as usize], rc.sc.rw_distance);
            texturecolumn >>= FRACBITS;
            // calculate lighting
            let index = i32::min(rsl.rw_scale>>LIGHTSCALESHIFT,
                                 (MAXLIGHTSCALE-1) as i32);

            rsl.dc.dc_colormap_index = *rc.sc.walllights.offset(index as isize);
            rsl.dc.dc_x = x as i32;
            rsl.dc.dc_iscale = ((0xffffffff as u32) / (rsl.rw_scale as u32)) as i32;
        }
        
        // draw the wall tiers
        if rc.sc.midtexture != 0 {
            // single sided line
            rsl.dc.dc_yl = yl;
            rsl.dc.dc_yh = yh;
            rsl.dc.dc_texturemid = rsl.rw_midtexturemid;
            rsl.dc.dc_source = R_GetColumn(&mut rc.rd, rc.sc.midtexture, texturecolumn);
            (rc.colfunc) (rc, &mut rsl.dc);
            rc.pc.ceilingclip[x] = viewheight as i16;
            rc.pc.floorclip[x] = -1;
        } else {
            // two sided line
            if rc.sc.toptexture != 0 {
                // top wall
                let mid = i32::min(rsl.pixhigh>>HEIGHTBITS,
                                   (rc.pc.floorclip[x]-1) as i32);
                rsl.pixhigh += rsl.pixhighstep;

                if mid >= yl {
                    rsl.dc.dc_yl = yl;
                    rsl.dc.dc_yh = mid;
                    rsl.dc.dc_texturemid = rsl.rw_toptexturemid;
                    rsl.dc.dc_source = R_GetColumn(&mut rc.rd, rc.sc.toptexture, texturecolumn);
                    (rc.colfunc) (rc, &mut rsl.dc);
                    rc.pc.ceilingclip[x] = mid as i16;
                } else {
                    rc.pc.ceilingclip[x] = (yl-1) as i16;
                }
            } else {
                // no top wall
                if rc.sc.markceiling != c_false {
                    rc.pc.ceilingclip[x] = (yl-1) as i16;
                }
            }
                    
            if rc.sc.bottomtexture != 0 {
                // bottom wall
                let mid = i32::max((rsl.pixlow+HEIGHTUNIT-1)>>HEIGHTBITS,
                                   (rc.pc.ceilingclip[x]+1) as i32);
                rsl.pixlow += rsl.pixlowstep;

                if mid <= yh {
                    rsl.dc.dc_yl = mid;
                    rsl.dc.dc_yh = yh;
                    rsl.dc.dc_texturemid = rsl.rw_bottomtexturemid;
                    rsl.dc.dc_source = R_GetColumn(&mut rc.rd, rc.sc.bottomtexture, texturecolumn);
                    (rc.colfunc) (rc, &mut rsl.dc);
                    rc.pc.floorclip[x] = mid as i16;
                } else {
                    rc.pc.floorclip[x] = (yh+1) as i16;
                }
            } else {
                // no bottom wall
                if rc.sc.markfloor != c_false {
                    rc.pc.floorclip[x] = (yh+1) as i16;
                }
            }
                    
            if rsl.maskedtexture != c_false {
                // save texturecol
                //  for backdrawing of masked mid texture
                rc.pc.openings[(rc.sc.maskedtexturecol_index +
                        (x as opening_index_t)) as usize] = texturecolumn as i16;
            }
        }
            
        rsl.rw_scale += rsl.rw_scalestep;
        rsl.topfrac += rsl.topstep;
        rsl.bottomfrac += rsl.bottomstep;
    }
}

//
// R_StoreWallRange
// A wall segment will be drawn
//  between start and stop pixels (inclusive).
//
pub unsafe fn R_StoreWallRange (rc: &mut RenderContext_t, start: i32, stop: i32) {
    // don't overflow and crash
    if rc.bc.ds_index >= (MAXDRAWSEGS as drawsegs_index_t) {
        return;
    }
        
    if (start >=viewwidth) || (start > stop) {
        panic!("Bad R_RenderWallRange: {} to {}", start , stop);
    }
    
    let mut rsl = R_RenderSegLoop_params_t {
        dc: empty_R_DrawColumn_params,
        bottomfrac: 0,
        bottomstep: 0,
        topfrac: 0,
        topstep: 0,
        maskedtexture: c_false,
        pixhigh: 0,
        pixhighstep: 0,
        pixlow: 0,
        pixlowstep: 0,
        rw_bottomtexturemid: 0,
        rw_centerangle: 0,
        rw_midtexturemid: 0,
        rw_offset: 0,
        rw_scale: 0,
        rw_scalestep: 0,
        rw_toptexturemid: 0,
        rw_x: start,
        rw_stopx: stop + 1,
    };
    rc.bc.sidedef = (*segs.offset(rc.bc.curline as isize)).sidedef;
    rc.bc.linedef = (*segs.offset(rc.bc.curline as isize)).linedef;

    // mark the segment as visible for auto map
    (*rc.bc.linedef).flags |= ML_MAPPED as i16;
    
    // calculate rw_distance for scale calculation
    rc.sc.rw_normalangle = (*segs.offset(rc.bc.curline as isize)).angle.wrapping_add(ANG90);
    let offsetangle: angle_t = angle_t::min(ANG90,
                i32::abs(rc.sc.rw_normalangle.wrapping_sub(
                            rc.sc.rw_angle1 as angle_t) as i32) as angle_t);
    
    let distangle: angle_t = ANG90 - offsetangle;
    let hyp: fixed_t = R_PointToDist (&mut rc.view,
                                      (*(*segs.offset(rc.bc.curline as isize)).v1).x,
                                      (*(*segs.offset(rc.bc.curline as isize)).v1).y);
    let sineval: fixed_t = finesine[(distangle>>ANGLETOFINESHIFT) as usize];
    rc.sc.rw_distance = FixedMul (hyp, sineval);
        
    
    rc.bc.drawsegs[rc.bc.ds_index as usize].x1 = start;
    rc.bc.drawsegs[rc.bc.ds_index as usize].x2 = stop;
    rc.bc.drawsegs[rc.bc.ds_index as usize].curline = rc.bc.curline;
    
    // calculate scale at both ends and step
    rsl.rw_scale = R_ScaleFromGlobalAngle (rc, rc.view.viewangle.wrapping_add(rc.xtoviewangle[start as usize]));
    rc.bc.drawsegs[rc.bc.ds_index as usize].scale1 = rsl.rw_scale;
    
    if stop > start {
        rc.bc.drawsegs[rc.bc.ds_index as usize].scale2 =
            R_ScaleFromGlobalAngle (rc, rc.view.viewangle.wrapping_add(rc.xtoviewangle[stop as usize]));
        rsl.rw_scalestep = (rc.bc.drawsegs[rc.bc.ds_index as usize].scale2 - rsl.rw_scale) / (stop-start);
        rc.bc.drawsegs[rc.bc.ds_index as usize].scalestep = rsl.rw_scalestep;
    } else {
        // UNUSED: try to fix the stretched line bug
        // #if 0
        //     if (rw_distance < FRACUNIT/2)
        //     {
        //         fixed_t		trx,try;
        //         fixed_t		gxt,gyt;
        // 
        //         trx = curline->v1->x - viewx;
        //         try = curline->v1->y - viewy;
        //             
        //         gxt = FixedMul(trx,viewcos); 
        //         gyt = -FixedMul(try,viewsin); 
        //         ds_p->scale1 = FixedDiv(projection, gxt-gyt)<<detailshift;
        //     }
        // #endif
        rc.bc.drawsegs[rc.bc.ds_index as usize].scale2 = rc.bc.drawsegs[rc.bc.ds_index as usize].scale1;
    }
    
    // calculate texture boundaries
    //  and decide if floor / ceiling marks are needed
    let mut worldtop: i32 = (*rc.bc.frontsector).ceilingheight - rc.view.viewz;
    let mut worldbottom: i32 = (*rc.bc.frontsector).floorheight - rc.view.viewz;
    let mut worldhigh: i32 = 0;
    let mut worldlow: i32 = 0;
    
    rc.sc.midtexture = 0;
    rc.sc.toptexture = 0;
    rc.sc.bottomtexture = 0;
    rc.bc.drawsegs[rc.bc.ds_index as usize].maskedtexturecol_index = INVALID_OPENING;
    
    if rc.bc.backsector == std::ptr::null_mut() {
        // single sided line
        rc.sc.midtexture = *texturetranslation.offset((*rc.bc.sidedef).midtexture as isize);
        // a single sided line is terminal, so it must mark ends
        rc.sc.markfloor = c_true;
        rc.sc.markceiling = c_true;
        if ((*rc.bc.linedef).flags & (ML_DONTPEGBOTTOM as i16)) != 0 {
            let vtop = (*rc.bc.frontsector).floorheight +
                *textureheight.offset((*rc.bc.sidedef).midtexture as isize);
            // bottom of texture at bottom
            rsl.rw_midtexturemid = vtop - rc.view.viewz;
        } else {
            // top of texture at top
            rsl.rw_midtexturemid = worldtop;
        }
        rsl.rw_midtexturemid += (*rc.bc.sidedef).rowoffset;

        rc.bc.drawsegs[rc.bc.ds_index as usize].silhouette = SIL_BOTH as i32;
        rc.bc.drawsegs[rc.bc.ds_index as usize].sprtopclip_index = SCREEN_HEIGHT_OPENING;
        rc.bc.drawsegs[rc.bc.ds_index as usize].sprbottomclip = negonearray.as_mut_ptr();
        rc.bc.drawsegs[rc.bc.ds_index as usize].bsilheight = MAXINT;
        rc.bc.drawsegs[rc.bc.ds_index as usize].tsilheight = MININT;
    } else {
        // two sided line
        rc.bc.drawsegs[rc.bc.ds_index as usize].sprtopclip_index = INVALID_OPENING;
        rc.bc.drawsegs[rc.bc.ds_index as usize].sprbottomclip = std::ptr::null_mut();
        rc.bc.drawsegs[rc.bc.ds_index as usize].silhouette = 0;
        
        if (*rc.bc.frontsector).floorheight > (*rc.bc.backsector).floorheight {
            rc.bc.drawsegs[rc.bc.ds_index as usize].silhouette = SIL_BOTTOM as i32;
            rc.bc.drawsegs[rc.bc.ds_index as usize].bsilheight = (*rc.bc.frontsector).floorheight;
        } else if (*rc.bc.backsector).floorheight > rc.view.viewz {
            rc.bc.drawsegs[rc.bc.ds_index as usize].silhouette = SIL_BOTTOM as i32;
            rc.bc.drawsegs[rc.bc.ds_index as usize].bsilheight = MAXINT;
            // rc.bc.drawsegs[rc.bc.ds_index as usize].sprbottomclip = negonearray;
        }
        
        if (*rc.bc.frontsector).ceilingheight < (*rc.bc.backsector).ceilingheight {
            rc.bc.drawsegs[rc.bc.ds_index as usize].silhouette |= SIL_TOP as i32;
            rc.bc.drawsegs[rc.bc.ds_index as usize].tsilheight = (*rc.bc.frontsector).ceilingheight;
        } else if (*rc.bc.backsector).ceilingheight < rc.view.viewz {
            rc.bc.drawsegs[rc.bc.ds_index as usize].silhouette |= SIL_TOP as i32;
            rc.bc.drawsegs[rc.bc.ds_index as usize].tsilheight = MININT;
            // rc.bc.drawsegs[rc.bc.ds_index as usize].sprtopclip_index = SCREEN_HEIGHT_OPENING;
        }
            
        if (*rc.bc.backsector).ceilingheight <= (*rc.bc.frontsector).floorheight {
            rc.bc.drawsegs[rc.bc.ds_index as usize].sprbottomclip = negonearray.as_mut_ptr();
            rc.bc.drawsegs[rc.bc.ds_index as usize].bsilheight = MAXINT;
            rc.bc.drawsegs[rc.bc.ds_index as usize].silhouette |= SIL_BOTTOM as i32;
        }
        
        if (*rc.bc.backsector).floorheight >= (*rc.bc.frontsector).ceilingheight {
            rc.bc.drawsegs[rc.bc.ds_index as usize].sprtopclip_index = SCREEN_HEIGHT_OPENING;
            rc.bc.drawsegs[rc.bc.ds_index as usize].tsilheight = MININT;
            rc.bc.drawsegs[rc.bc.ds_index as usize].silhouette |= SIL_TOP as i32;
        }
        
        worldhigh = (*rc.bc.backsector).ceilingheight - rc.view.viewz;
        worldlow = (*rc.bc.backsector).floorheight - rc.view.viewz;
            
        // hack to allow height changes in outdoor areas
        if ((*rc.bc.frontsector).ceilingpic == (skyflatnum as i16))
        && ((*rc.bc.backsector).ceilingpic == (skyflatnum as i16)) {
            worldtop = worldhigh;
        }
        
                
        if (worldlow != worldbottom)
        || ((*rc.bc.backsector).floorpic != (*rc.bc.frontsector).floorpic)
        || ((*rc.bc.backsector).lightlevel != (*rc.bc.frontsector).lightlevel) {
            rc.sc.markfloor = c_true;
        } else {
            // same plane on both sides
            rc.sc.markfloor = c_false;
        }
        
                
        if (worldhigh != worldtop)
        || ((*rc.bc.backsector).ceilingpic != (*rc.bc.frontsector).ceilingpic)
        || ((*rc.bc.backsector).lightlevel != (*rc.bc.frontsector).lightlevel) {
            rc.sc.markceiling = c_true;
        } else {
            // same plane on both sides
            rc.sc.markceiling = c_false;
        }
        
        if ((*rc.bc.backsector).ceilingheight <= (*rc.bc.frontsector).floorheight)
        || ((*rc.bc.backsector).floorheight >= (*rc.bc.frontsector).ceilingheight) {
            // closed door
            rc.sc.markceiling = c_true;
            rc.sc.markfloor = c_true;
        }
        

        if worldhigh < worldtop {
            // top texture
            rc.sc.toptexture = *texturetranslation.offset((*rc.bc.sidedef).toptexture as isize);
            if ((*rc.bc.linedef).flags & (ML_DONTPEGTOP as i16)) != 0 {
                // top of texture at top
                rsl.rw_toptexturemid = worldtop;
            } else {
                let vtop = (*rc.bc.backsector).ceilingheight
                    + *textureheight.offset((*rc.bc.sidedef).toptexture as isize);
            
                // bottom of texture
                rsl.rw_toptexturemid = vtop - rc.view.viewz;
            }
        }
        if worldlow > worldbottom {
            // bottom texture
            rc.sc.bottomtexture = *texturetranslation.offset((*rc.bc.sidedef).bottomtexture as isize);

            if ((*rc.bc.linedef).flags & (ML_DONTPEGBOTTOM as i16)) != 0 {
                // bottom of texture at bottom
                // top of texture at top
                rsl.rw_bottomtexturemid = worldtop;
            } else { // top of texture at top
                rsl.rw_bottomtexturemid = worldlow;
            }
        }
        rsl.rw_toptexturemid += (*rc.bc.sidedef).rowoffset;
        rsl.rw_bottomtexturemid += (*rc.bc.sidedef).rowoffset;
        
        // allocate space for masked texture tables
        if (*rc.bc.sidedef).midtexture != 0 {
            // masked midtexture
            rsl.maskedtexture = c_true;
            rc.sc.maskedtexturecol_index = 
                rc.pc.lastopening_index - (rsl.rw_x as opening_index_t);
            rc.bc.drawsegs[rc.bc.ds_index as usize].maskedtexturecol_index =
                rc.sc.maskedtexturecol_index;
            rc.pc.lastopening_index += (rsl.rw_stopx - rsl.rw_x) as opening_index_t;
        }
    }
    
    // calculate rw_offset (only needed for textured lines)
    rc.sc.segtextured = rc.sc.midtexture | rc.sc.toptexture
                    | rc.sc.bottomtexture | rsl.maskedtexture;

    if rc.sc.segtextured != c_false {
        let mut offsetangle = rc.sc.rw_normalangle.wrapping_sub(rc.sc.rw_angle1 as angle_t);
        
        if offsetangle > ANG180 {
            offsetangle = (0 as angle_t).wrapping_sub(offsetangle); // DSB-20
        }

        if offsetangle > ANG90 {
            offsetangle = ANG90;
        }

        let sineval = finesine[(offsetangle >>ANGLETOFINESHIFT) as usize];
        rsl.rw_offset = FixedMul (hyp, sineval);

        if (rc.sc.rw_normalangle.wrapping_sub(rc.sc.rw_angle1 as angle_t)) < ANG180 {
            rsl.rw_offset = -rsl.rw_offset;
        }

        rsl.rw_offset += (*rc.bc.sidedef).textureoffset + (*segs.offset(rc.bc.curline as isize)).offset;
        rsl.rw_centerangle = ANG90.wrapping_add(rc.view.viewangle).wrapping_sub(rc.sc.rw_normalangle);
        
        // calculate light table
        //  use different light tables
        //  for horizontal / vertical / diagonal
        // OPTIMIZE: get rid of LIGHTSEGSHIFT globally
        if rc.fixedcolormap_index == NULL_COLORMAP {
            let mut lightnum = (((*rc.bc.frontsector).lightlevel >> LIGHTSEGSHIFT) as i32) + rc.extralight;

            if (*(*segs.offset(rc.bc.curline as isize)).v1).y == (*(*segs.offset(rc.bc.curline as isize)).v2).y {
                lightnum -= 1;
            } else if (*(*segs.offset(rc.bc.curline as isize)).v1).x == (*(*segs.offset(rc.bc.curline as isize)).v2).x {
                lightnum += 1;
            }

            rc.sc.walllights = rc.scalelight[i32::max(0, i32::min((LIGHTLEVELS - 1) as i32, lightnum)) as usize].as_mut_ptr();
        }
    }
    
    // if a floor / ceiling plane is on the wrong side
    //  of the view plane, it is definitely invisible
    //  and doesn't need to be marked.
    
  
    if (*rc.bc.frontsector).floorheight >= rc.view.viewz {
        // above view plane
        rc.sc.markfloor = c_false;
    }
    
    if ((*rc.bc.frontsector).ceilingheight <= rc.view.viewz)
    && (((*rc.bc.frontsector).ceilingpic as i32) != skyflatnum) {
        // below view plane
        rc.sc.markceiling = c_false;
    }

    
    // calculate incremental stepping values for texture edges
    worldtop >>= 4;
    worldbottom >>= 4;
    
    rsl.topstep = -FixedMul (rsl.rw_scalestep, worldtop);
    rsl.topfrac = (rc.centeryfrac>>4) - FixedMul (worldtop, rsl.rw_scale);

    rsl.bottomstep = -FixedMul (rsl.rw_scalestep,worldbottom);
    rsl.bottomfrac = (rc.centeryfrac>>4) - FixedMul (worldbottom, rsl.rw_scale);
    
    if rc.bc.backsector != std::ptr::null_mut() {
        worldhigh >>= 4;
        worldlow >>= 4;

        if worldhigh < worldtop {
            rsl.pixhigh = (rc.centeryfrac>>4) - FixedMul (worldhigh, rsl.rw_scale);
            rsl.pixhighstep = -FixedMul (rsl.rw_scalestep,worldhigh);
        }
        
        if worldlow > worldbottom {
            rsl.pixlow = (rc.centeryfrac>>4) - FixedMul (worldlow, rsl.rw_scale);
            rsl.pixlowstep = -FixedMul (rsl.rw_scalestep,worldlow);
        }
    }
    
    // render it
    if rc.sc.markceiling != c_false {
        let index = rc.pc.ceilingplane_index;
        rc.pc.ceilingplane_index = R_CheckPlane (&mut rc.pc, index,
                                                 rsl.rw_x, rsl.rw_stopx-1);
    }
    
    if rc.sc.markfloor != c_false {
        let index = rc.pc.floorplane_index;
        rc.pc.floorplane_index = R_CheckPlane (&mut rc.pc, index,
                                               rsl.rw_x, rsl.rw_stopx-1);
    }

    R_RenderSegLoop (rc, &mut rsl);

    
    // save sprite clipping info
    if ((0 != (rc.bc.drawsegs[rc.bc.ds_index as usize].silhouette & (SIL_TOP as i32)))
        || (rsl.maskedtexture != c_false))
    && (rc.bc.drawsegs[rc.bc.ds_index as usize].sprtopclip_index == INVALID_OPENING) {
        let copy_size: usize = (rsl.rw_stopx - start) as usize;
        for i in 0 .. copy_size {
            rc.pc.openings[(rc.pc.lastopening_index as usize) + i] = 
                rc.pc.ceilingclip[(start as usize) + i];
        }
        rc.bc.drawsegs[rc.bc.ds_index as usize].sprtopclip_index =
                rc.pc.lastopening_index - (start as opening_index_t);
        rc.pc.lastopening_index += (rsl.rw_stopx - start) as opening_index_t;
    }
    
    if ((0 != (rc.bc.drawsegs[rc.bc.ds_index as usize].silhouette & (SIL_BOTTOM as i32)))
        || (rsl.maskedtexture != c_false))
    && (rc.bc.drawsegs[rc.bc.ds_index as usize].sprbottomclip == std::ptr::null_mut()) {
        let copy_size: usize = (rsl.rw_stopx - start) as usize;
        for i in 0 .. copy_size {
            rc.pc.openings[(rc.pc.lastopening_index as usize) + i] = 
                rc.pc.floorclip[(start as usize) + i];
        }
        rc.bc.drawsegs[rc.bc.ds_index as usize].sprbottomclip =
                rc.pc.openings.as_mut_ptr().offset(
                    (rc.pc.lastopening_index as isize) - (start as isize));
        rc.pc.lastopening_index += (rsl.rw_stopx - start) as opening_index_t;
    }

    if (rsl.maskedtexture != c_false)
    && (0 == (rc.bc.drawsegs[rc.bc.ds_index as usize].silhouette & (SIL_TOP as i32))) {
        rc.bc.drawsegs[rc.bc.ds_index as usize].silhouette |= SIL_TOP as i32;
        rc.bc.drawsegs[rc.bc.ds_index as usize].tsilheight = MININT;
    }
    if (rsl.maskedtexture != c_false)
    && (0 == (rc.bc.drawsegs[rc.bc.ds_index as usize].silhouette & (SIL_BOTTOM as i32))) {
        rc.bc.drawsegs[rc.bc.ds_index as usize].silhouette |= SIL_BOTTOM as i32;
        rc.bc.drawsegs[rc.bc.ds_index as usize].bsilheight = MAXINT;
    }
    rc.bc.ds_index += 1;
}

