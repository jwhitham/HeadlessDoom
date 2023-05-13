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
use crate::funcs::*;
use crate::r_things;
use crate::tables::finetangent;
use crate::tables::finesine;
use crate::m_fixed::FixedMul;
use crate::r_data::R_GetColumn;
use crate::r_data::NULL_COLORMAP;
use crate::r_data::colormap_index_t;
use crate::r_main::R_PointToDist;
use crate::r_main::R_ScaleFromGlobalAngle;
use crate::r_main::RenderContext_t;
use crate::r_plane::R_CheckPlane;
use crate::r_main::viewz;
use crate::r_main::viewangle;
use crate::r_main::scalelight;
use crate::r_main::extralight;
use crate::r_main::xtoviewangle;
use crate::r_plane::ceilingclip;
use crate::r_plane::ceilingplane;
use crate::r_plane::floorclip;
use crate::r_plane::floorplane;
use crate::r_plane::lastopening;
use crate::r_things::negonearray;
use crate::r_things::screenheightarray;
use crate::r_draw::empty_R_DrawColumn_params;
use crate::r_draw::R_DrawColumn_params_t;

static mut markceiling: boolean = c_false;
static mut markfloor: boolean = c_false;
static mut segtextured: boolean = c_false;
pub static mut rw_distance: fixed_t = 0;
static mut midtexture: i32 = 0;
static mut toptexture: i32 = 0;
static mut bottomtexture: i32 = 0;
pub static mut rw_normalangle: angle_t = 0;
pub static mut rw_angle1: i32 = 0;
pub static mut walllights: *mut colormap_index_t = std::ptr::null_mut();
pub static mut maskedtexturecol: *mut i16 = std::ptr::null_mut();

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
        (rc: &mut RenderContext_t, ds: *mut drawseg_t, x1: i32, x2: i32) {
    // Calculate light table.
    // Use different light tables
    //   for horizontal / vertical / diagonal. Diagonal?
    // OPTIMIZE: get rid of LIGHTSEGSHIFT globally
    rc.bc.curline = (*ds).curline;
    rc.bc.frontsector = (*rc.bc.curline).frontsector;
    rc.bc.backsector = (*rc.bc.curline).backsector;
    let texnum = *texturetranslation.offset(
        (*(*rc.bc.curline).sidedef).midtexture as isize);
    
    let mut lightnum = (((*rc.bc.frontsector).lightlevel >> LIGHTSEGSHIFT) as i32)
                    + extralight;

    if (*(*rc.bc.curline).v1).y == (*(*rc.bc.curline).v2).y {
        lightnum -= 1;
    } else if (*(*rc.bc.curline).v1).x == (*(*rc.bc.curline).v2).x {
        lightnum += 1;
    }

    walllights = scalelight[i32::max(0,
                            i32::min((LIGHTLEVELS - 1) as i32, lightnum)) as usize].as_mut_ptr();

    maskedtexturecol = (*ds).maskedtexturecol;

    let rw_scalestep = (*ds).scalestep;
    let mut dmc = r_things::R_DrawMaskedColumn_params_t {
        dc: empty_R_DrawColumn_params,
        column: std::ptr::null_mut(),
        sprtopscreen: 0,
        spryscale: (*ds).scale1 + (x1 - (*ds).x1)*rw_scalestep,
        mfloorclip: (*ds).sprbottomclip,
        mceilingclip: (*ds).sprtopclip,
    };
    
    // find positioning
    if (((*(*rc.bc.curline).linedef).flags as u32) & ML_DONTPEGBOTTOM) != 0 {
        dmc.dc.dc_texturemid =
            if (*rc.bc.frontsector).floorheight > (*rc.bc.backsector).floorheight {
                (*rc.bc.frontsector).floorheight
            } else {
                (*rc.bc.backsector).floorheight
            };
        dmc.dc.dc_texturemid = dmc.dc.dc_texturemid +
                *textureheight.offset(texnum as isize) - viewz;
    } else {
        dmc.dc.dc_texturemid =
            if (*rc.bc.frontsector).ceilingheight < (*rc.bc.backsector).ceilingheight {
                (*rc.bc.frontsector).ceilingheight
            } else {
                (*rc.bc.backsector).ceilingheight
            };
        dmc.dc.dc_texturemid = dmc.dc.dc_texturemid - viewz;
    }
    dmc.dc.dc_texturemid += (*(*rc.bc.curline).sidedef).rowoffset;
            
    if rc.fixedcolormap_index != NULL_COLORMAP {
        dmc.dc.dc_colormap_index = rc.fixedcolormap_index;
    }
    
    // draw the columns
    for x in x1 ..= x2 {
        dmc.dc.dc_x = x;
        // calculate lighting
        let colnum = *maskedtexturecol.offset(dmc.dc.dc_x as isize);
        if colnum != MAXSHORT {
            if rc.fixedcolormap_index == NULL_COLORMAP {
                let index = i32::min((MAXLIGHTSCALE - 1) as i32,
                                    dmc.spryscale>>LIGHTSCALESHIFT);
                dmc.dc.dc_colormap_index = *walllights.offset(index as isize);
            }
                
            dmc.sprtopscreen = rc.centeryfrac - FixedMul(dmc.dc.dc_texturemid, dmc.spryscale);
            dmc.dc.dc_iscale = ((0xffffffff as u32) / (dmc.spryscale as u32)) as i32;
            
            // draw the texture
            dmc.column = (R_GetColumn(&mut rc.rd, texnum, colnum as i32)
                            as *mut u8).offset(-3) as *mut column_t;
                
            r_things::R_DrawMaskedColumn (rc, &mut dmc);
            *maskedtexturecol.offset(dmc.dc.dc_x as isize) = MAXSHORT;
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
                          (ceilingclip[x]+1) as i32);
        
        if markceiling != c_false {
            let top = (ceilingclip[x]+1) as i32;
            let bottom = i32::min(yl-1, (floorclip[x]-1) as i32);

            if top <= bottom {
                (*ceilingplane).top[x] = top as u8;
                (*ceilingplane).bottom[x] = bottom as u8;
            }
        }
            
        let yh = i32::min(rsl.bottomfrac>>HEIGHTBITS, (floorclip[x]-1) as i32);

        if markfloor != c_false {
            let top = i32::max(yh+1, (ceilingclip[x]+1) as i32);
            let bottom = (floorclip[x]-1) as i32;
            if top <= bottom {
                (*floorplane).top[x] = top as u8;
                (*floorplane).bottom[x] = bottom as u8;
            }
        }
        
        // texturecolumn and lighting are independent of wall tiers
        if segtextured != c_false {
            // calculate texture offset
            let mut angle = rsl.rw_centerangle.wrapping_add(xtoviewangle[x])>>ANGLETOFINESHIFT;

            if angle >= (FINEANGLES / 2) { // DSB-23
                angle = 0;
            }

            texturecolumn = rsl.rw_offset-FixedMul(finetangent[angle as usize],rw_distance);
            texturecolumn >>= FRACBITS;
            // calculate lighting
            let index = i32::min(rsl.rw_scale>>LIGHTSCALESHIFT,
                                 (MAXLIGHTSCALE-1) as i32);

            rsl.dc.dc_colormap_index = *walllights.offset(index as isize);
            rsl.dc.dc_x = x as i32;
            rsl.dc.dc_iscale = ((0xffffffff as u32) / (rsl.rw_scale as u32)) as i32;
        }
        
        // draw the wall tiers
        if midtexture != 0 {
            // single sided line
            rsl.dc.dc_yl = yl;
            rsl.dc.dc_yh = yh;
            rsl.dc.dc_texturemid = rsl.rw_midtexturemid;
            rsl.dc.dc_source = R_GetColumn(&mut rc.rd, midtexture, texturecolumn);
            (rc.colfunc) (rc, &mut rsl.dc);
            ceilingclip[x] = viewheight as i16;
            floorclip[x] = -1;
        } else {
            // two sided line
            if toptexture != 0 {
                // top wall
                let mid = i32::min(rsl.pixhigh>>HEIGHTBITS,
                                   (floorclip[x]-1) as i32);
                rsl.pixhigh += rsl.pixhighstep;

                if mid >= yl {
                    rsl.dc.dc_yl = yl;
                    rsl.dc.dc_yh = mid;
                    rsl.dc.dc_texturemid = rsl.rw_toptexturemid;
                    rsl.dc.dc_source = R_GetColumn(&mut rc.rd, toptexture, texturecolumn);
                    (rc.colfunc) (rc, &mut rsl.dc);
                    ceilingclip[x] = mid as i16;
                } else {
                    ceilingclip[x] = (yl-1) as i16;
                }
            } else {
                // no top wall
                if markceiling != c_false {
                    ceilingclip[x] = (yl-1) as i16;
                }
            }
                    
            if bottomtexture != 0 {
                // bottom wall
                let mid = i32::max((rsl.pixlow+HEIGHTUNIT-1)>>HEIGHTBITS,
                                   (ceilingclip[x]+1) as i32);
                rsl.pixlow += rsl.pixlowstep;

                if mid <= yh {
                    rsl.dc.dc_yl = mid;
                    rsl.dc.dc_yh = yh;
                    rsl.dc.dc_texturemid = rsl.rw_bottomtexturemid;
                    rsl.dc.dc_source = R_GetColumn(&mut rc.rd, bottomtexture, texturecolumn);
                    (rc.colfunc) (rc, &mut rsl.dc);
                    floorclip[x] = mid as i16;
                } else {
                    floorclip[x] = (yh+1) as i16;
                }
            } else {
                // no bottom wall
                if markfloor != c_false {
                    floorclip[x] = (yh+1) as i16;
                }
            }
                    
            if rsl.maskedtexture != c_false {
                // save texturecol
                //  for backdrawing of masked mid texture
                *maskedtexturecol.offset(x as isize) = texturecolumn as i16;
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
    if rc.bc.ds_p == rc.bc.drawsegs.as_mut_ptr().offset(MAXDRAWSEGS as isize) {
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
    rc.bc.sidedef = (*rc.bc.curline).sidedef;
    rc.bc.linedef = (*rc.bc.curline).linedef;

    // mark the segment as visible for auto map
    (*rc.bc.linedef).flags |= ML_MAPPED as i16;
    
    // calculate rw_distance for scale calculation
    rw_normalangle = (*rc.bc.curline).angle.wrapping_add(ANG90);
    let offsetangle: angle_t = angle_t::min(ANG90,
                i32::abs(rw_normalangle.wrapping_sub(rw_angle1 as angle_t) as i32) as angle_t);
    
    let distangle: angle_t = ANG90 - offsetangle;
    let hyp: fixed_t = R_PointToDist ((*(*rc.bc.curline).v1).x, (*(*rc.bc.curline).v1).y);
    let sineval: fixed_t = finesine[(distangle>>ANGLETOFINESHIFT) as usize];
    rw_distance = FixedMul (hyp, sineval);
        
    
    (*rc.bc.ds_p).x1 = start;
    (*rc.bc.ds_p).x2 = stop;
    (*rc.bc.ds_p).curline = rc.bc.curline;
    
    // calculate scale at both ends and step
    rsl.rw_scale = R_ScaleFromGlobalAngle (viewangle.wrapping_add(xtoviewangle[start as usize]));
    (*rc.bc.ds_p).scale1 = rsl.rw_scale;
    
    if stop > start {
        (*rc.bc.ds_p).scale2 = R_ScaleFromGlobalAngle (viewangle.wrapping_add(xtoviewangle[stop as usize]));
        rsl.rw_scalestep = ((*rc.bc.ds_p).scale2 - rsl.rw_scale) / (stop-start);
        (*rc.bc.ds_p).scalestep = rsl.rw_scalestep;
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
        (*rc.bc.ds_p).scale2 = (*rc.bc.ds_p).scale1;
    }
    
    // calculate texture boundaries
    //  and decide if floor / ceiling marks are needed
    let mut worldtop: i32 = (*rc.bc.frontsector).ceilingheight - viewz;
    let mut worldbottom: i32 = (*rc.bc.frontsector).floorheight - viewz;
    let mut worldhigh: i32 = 0;
    let mut worldlow: i32 = 0;
    
    midtexture = 0;
    toptexture = 0;
    bottomtexture = 0;
    (*rc.bc.ds_p).maskedtexturecol = std::ptr::null_mut();
    
    if rc.bc.backsector == std::ptr::null_mut() {
        // single sided line
        midtexture = *texturetranslation.offset((*rc.bc.sidedef).midtexture as isize);
        // a single sided line is terminal, so it must mark ends
        markfloor = c_true;
        markceiling = c_true;
        if ((*rc.bc.linedef).flags & (ML_DONTPEGBOTTOM as i16)) != 0 {
            let vtop = (*rc.bc.frontsector).floorheight +
                *textureheight.offset((*rc.bc.sidedef).midtexture as isize);
            // bottom of texture at bottom
            rsl.rw_midtexturemid = vtop - viewz;
        } else {
            // top of texture at top
            rsl.rw_midtexturemid = worldtop;
        }
        rsl.rw_midtexturemid += (*rc.bc.sidedef).rowoffset;

        (*rc.bc.ds_p).silhouette = SIL_BOTH as i32;
        (*rc.bc.ds_p).sprtopclip = screenheightarray.as_mut_ptr();
        (*rc.bc.ds_p).sprbottomclip = negonearray.as_mut_ptr();
        (*rc.bc.ds_p).bsilheight = MAXINT;
        (*rc.bc.ds_p).tsilheight = MININT;
    } else {
        // two sided line
        (*rc.bc.ds_p).sprtopclip = std::ptr::null_mut();
        (*rc.bc.ds_p).sprbottomclip = std::ptr::null_mut();
        (*rc.bc.ds_p).silhouette = 0;
        
        if (*rc.bc.frontsector).floorheight > (*rc.bc.backsector).floorheight {
            (*rc.bc.ds_p).silhouette = SIL_BOTTOM as i32;
            (*rc.bc.ds_p).bsilheight = (*rc.bc.frontsector).floorheight;
        } else if (*rc.bc.backsector).floorheight > viewz {
            (*rc.bc.ds_p).silhouette = SIL_BOTTOM as i32;
            (*rc.bc.ds_p).bsilheight = MAXINT;
            // (*rc.bc.ds_p).sprbottomclip = negonearray;
        }
        
        if (*rc.bc.frontsector).ceilingheight < (*rc.bc.backsector).ceilingheight {
            (*rc.bc.ds_p).silhouette |= SIL_TOP as i32;
            (*rc.bc.ds_p).tsilheight = (*rc.bc.frontsector).ceilingheight;
        } else if (*rc.bc.backsector).ceilingheight < viewz {
            (*rc.bc.ds_p).silhouette |= SIL_TOP as i32;
            (*rc.bc.ds_p).tsilheight = MININT;
            // (*rc.bc.ds_p).sprtopclip = screenheightarray;
        }
            
        if (*rc.bc.backsector).ceilingheight <= (*rc.bc.frontsector).floorheight {
            (*rc.bc.ds_p).sprbottomclip = negonearray.as_mut_ptr();
            (*rc.bc.ds_p).bsilheight = MAXINT;
            (*rc.bc.ds_p).silhouette |= SIL_BOTTOM as i32;
        }
        
        if (*rc.bc.backsector).floorheight >= (*rc.bc.frontsector).ceilingheight {
            (*rc.bc.ds_p).sprtopclip = screenheightarray.as_mut_ptr();
            (*rc.bc.ds_p).tsilheight = MININT;
            (*rc.bc.ds_p).silhouette |= SIL_TOP as i32;
        }
        
        worldhigh = (*rc.bc.backsector).ceilingheight - viewz;
        worldlow = (*rc.bc.backsector).floorheight - viewz;
            
        // hack to allow height changes in outdoor areas
        if ((*rc.bc.frontsector).ceilingpic == (skyflatnum as i16))
        && ((*rc.bc.backsector).ceilingpic == (skyflatnum as i16)) {
            worldtop = worldhigh;
        }
        
                
        if (worldlow != worldbottom)
        || ((*rc.bc.backsector).floorpic != (*rc.bc.frontsector).floorpic)
        || ((*rc.bc.backsector).lightlevel != (*rc.bc.frontsector).lightlevel) {
            markfloor = c_true;
        } else {
            // same plane on both sides
            markfloor = c_false;
        }
        
                
        if (worldhigh != worldtop)
        || ((*rc.bc.backsector).ceilingpic != (*rc.bc.frontsector).ceilingpic)
        || ((*rc.bc.backsector).lightlevel != (*rc.bc.frontsector).lightlevel) {
            markceiling = c_true;
        } else {
            // same plane on both sides
            markceiling = c_false;
        }
        
        if ((*rc.bc.backsector).ceilingheight <= (*rc.bc.frontsector).floorheight)
        || ((*rc.bc.backsector).floorheight >= (*rc.bc.frontsector).ceilingheight) {
            // closed door
            markceiling = c_true;
            markfloor = c_true;
        }
        

        if worldhigh < worldtop {
            // top texture
            toptexture = *texturetranslation.offset((*rc.bc.sidedef).toptexture as isize);
            if ((*rc.bc.linedef).flags & (ML_DONTPEGTOP as i16)) != 0 {
                // top of texture at top
                rsl.rw_toptexturemid = worldtop;
            } else {
                let vtop = (*rc.bc.backsector).ceilingheight
                    + *textureheight.offset((*rc.bc.sidedef).toptexture as isize);
            
                // bottom of texture
                rsl.rw_toptexturemid = vtop - viewz;
            }
        }
        if worldlow > worldbottom {
            // bottom texture
            bottomtexture = *texturetranslation.offset((*rc.bc.sidedef).bottomtexture as isize);

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
            maskedtexturecol = lastopening.offset(-(rsl.rw_x as isize));
            (*rc.bc.ds_p).maskedtexturecol = maskedtexturecol;
            lastopening = lastopening.offset((rsl.rw_stopx - rsl.rw_x) as isize);
        }
    }
    
    // calculate rw_offset (only needed for textured lines)
    segtextured = midtexture | toptexture | bottomtexture | rsl.maskedtexture;

    if segtextured != c_false {
        let mut offsetangle = rw_normalangle.wrapping_sub(rw_angle1 as angle_t);
        
        if offsetangle > ANG180 {
            offsetangle = (0 as angle_t).wrapping_sub(offsetangle); // DSB-20
        }

        if offsetangle > ANG90 {
            offsetangle = ANG90;
        }

        let sineval = finesine[(offsetangle >>ANGLETOFINESHIFT) as usize];
        rsl.rw_offset = FixedMul (hyp, sineval);

        if (rw_normalangle.wrapping_sub(rw_angle1 as angle_t)) < ANG180 {
            rsl.rw_offset = -rsl.rw_offset;
        }

        rsl.rw_offset += (*rc.bc.sidedef).textureoffset + (*rc.bc.curline).offset;
        rsl.rw_centerangle = ANG90.wrapping_add(viewangle).wrapping_sub(rw_normalangle);
        
        // calculate light table
        //  use different light tables
        //  for horizontal / vertical / diagonal
        // OPTIMIZE: get rid of LIGHTSEGSHIFT globally
        if rc.fixedcolormap_index == NULL_COLORMAP {
            let mut lightnum = (((*rc.bc.frontsector).lightlevel >> LIGHTSEGSHIFT) as i32) + extralight;

            if (*(*rc.bc.curline).v1).y == (*(*rc.bc.curline).v2).y {
                lightnum -= 1;
            } else if (*(*rc.bc.curline).v1).x == (*(*rc.bc.curline).v2).x {
                lightnum += 1;
            }

            walllights = scalelight[i32::max(0, i32::min((LIGHTLEVELS - 1) as i32, lightnum)) as usize].as_mut_ptr();
        }
    }
    
    // if a floor / ceiling plane is on the wrong side
    //  of the view plane, it is definitely invisible
    //  and doesn't need to be marked.
    
  
    if (*rc.bc.frontsector).floorheight >= viewz {
        // above view plane
        markfloor = c_false;
    }
    
    if ((*rc.bc.frontsector).ceilingheight <= viewz)
    && (((*rc.bc.frontsector).ceilingpic as i32) != skyflatnum) {
        // below view plane
        markceiling = c_false;
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
    if markceiling != c_false {
        ceilingplane = R_CheckPlane (ceilingplane, rsl.rw_x, rsl.rw_stopx-1);
    }
    
    if markfloor != c_false {
        floorplane = R_CheckPlane (floorplane, rsl.rw_x, rsl.rw_stopx-1);
    }

    R_RenderSegLoop (rc, &mut rsl);

    
    // save sprite clipping info
    if ((0 != ((*rc.bc.ds_p).silhouette & (SIL_TOP as i32)))
        || (rsl.maskedtexture != c_false))
    && ((*rc.bc.ds_p).sprtopclip == std::ptr::null_mut()) {
        memcpy (lastopening as *mut u8,
                ceilingclip.as_mut_ptr().offset(start as isize) as *const u8,
                2*(rsl.rw_stopx-start) as usize);
        (*rc.bc.ds_p).sprtopclip = lastopening.offset(-(start as isize));
        lastopening = lastopening.offset((rsl.rw_stopx - start) as isize);
    }
    
    if ((0 != ((*rc.bc.ds_p).silhouette & (SIL_BOTTOM as i32)))
        || (rsl.maskedtexture != c_false))
    && ((*rc.bc.ds_p).sprbottomclip == std::ptr::null_mut()) {
        memcpy (lastopening as *mut u8,
                floorclip.as_mut_ptr().offset(start as isize) as *const u8,
                2*(rsl.rw_stopx-start) as usize);
        (*rc.bc.ds_p).sprbottomclip = lastopening.offset(-(start as isize));
        lastopening = lastopening.offset((rsl.rw_stopx - start) as isize);
    }

    if (rsl.maskedtexture != c_false)
    && (0 == ((*rc.bc.ds_p).silhouette & (SIL_TOP as i32))) {
        (*rc.bc.ds_p).silhouette |= SIL_TOP as i32;
        (*rc.bc.ds_p).tsilheight = MININT;
    }
    if (rsl.maskedtexture != c_false)
    && (0 == ((*rc.bc.ds_p).silhouette & (SIL_BOTTOM as i32))) {
        (*rc.bc.ds_p).silhouette |= SIL_BOTTOM as i32;
        (*rc.bc.ds_p).bsilheight = MAXINT;
    }
    rc.bc.ds_p = rc.bc.ds_p.offset(1);
}

