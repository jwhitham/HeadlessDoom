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

extern {
    static mut worldtop: i32;
    static mut worldbottom: i32;
    static mut worldhigh: i32;
    static mut worldlow: i32;
}

//
//
// R_RenderMaskedSegRange
//
#[no_mangle]
pub unsafe extern "C" fn R_RenderMaskedSegRange
        (ds: *mut drawseg_t, x1: i32, x2: i32) {
    // Calculate light table.
    // Use different light tables
    //   for horizontal / vertical / diagonal. Diagonal?
    // OPTIMIZE: get rid of LIGHTSEGSHIFT globally
    curline = (*ds).curline;
    frontsector = (*curline).frontsector;
    backsector = (*curline).backsector;
    let texnum = *texturetranslation.offset(
        (*(*curline).sidedef).midtexture as isize);
    
    let mut lightnum = (((*frontsector).lightlevel >> LIGHTSEGSHIFT) as i32)
                    + extralight;

    if (*(*curline).v1).y == (*(*curline).v2).y {
        lightnum -= 1;
    } else if (*(*curline).v1).x == (*(*curline).v2).x {
        lightnum += 1;
    }

    walllights = scalelight[i32::max(0,
                            i32::min((LIGHTLEVELS - 1) as i32, lightnum)) as usize].as_mut_ptr();

    maskedtexturecol = (*ds).maskedtexturecol;

    rw_scalestep = (*ds).scalestep;		
    spryscale = (*ds).scale1 + (x1 - (*ds).x1)*rw_scalestep;
    mfloorclip = (*ds).sprbottomclip;
    mceilingclip = (*ds).sprtopclip;
    
    // find positioning
    if (((*(*curline).linedef).flags as u32) & ML_DONTPEGBOTTOM) != 0 {
        dc_texturemid =
            if (*frontsector).floorheight > (*backsector).floorheight {
                (*frontsector).floorheight
            } else {
                (*backsector).floorheight
            };
        dc_texturemid = dc_texturemid +
                *textureheight.offset(texnum as isize) - viewz;
    } else {
        dc_texturemid =
            if (*frontsector).ceilingheight < (*backsector).ceilingheight {
                (*frontsector).ceilingheight
            } else {
                (*backsector).ceilingheight
            };
        dc_texturemid = dc_texturemid - viewz;
    }
    dc_texturemid += (*(*curline).sidedef).rowoffset;
            
    if fixedcolormap != std::ptr::null_mut() {
        dc_colormap = fixedcolormap;
    }
    
    // draw the columns
    for x in x1 ..= x2 {
        dc_x = x;
        // calculate lighting
        let colnum = *maskedtexturecol.offset(dc_x as isize);
        if colnum != MAXSHORT {
            if fixedcolormap == std::ptr::null_mut() {
                let index = i32::min((MAXLIGHTSCALE - 1) as i32,
                                    spryscale>>LIGHTSCALESHIFT);
                dc_colormap = *walllights.offset(index as isize);
            }
                
            sprtopscreen = centeryfrac - FixedMul(dc_texturemid, spryscale);
            dc_iscale = ((0xffffffff as u32) / (spryscale as u32)) as i32;
            
            // draw the texture
            let col = (R_GetColumn(texnum, colnum as i32)
                            as *mut u8).offset(-3) as *mut column_t;
                
            r_things::R_DrawMaskedColumn (col);
            *maskedtexturecol.offset(dc_x as isize) = MAXSHORT;
        }
        spryscale += rw_scalestep;
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

unsafe fn R_RenderSegLoop () {
    let mut texturecolumn: fixed_t = 0;
    for x in rw_x as usize .. rw_stopx as usize {
        rw_x = x as i32;
        // mark floor / ceiling areas
        let yl = i32::max((topfrac+HEIGHTUNIT-1)>>HEIGHTBITS,
                          (ceilingclip[x]+1) as i32);
        
        if markceiling != c_false {
            let top = (ceilingclip[x]+1) as i32;
            let bottom = i32::min(yl-1, (floorclip[x]-1) as i32);

            if top <= bottom {
                (*ceilingplane).top[x] = top as u8;
                (*ceilingplane).bottom[x] = bottom as u8;
            }
        }
            
        let yh = i32::min(bottomfrac>>HEIGHTBITS, (floorclip[x]-1) as i32);

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
            let mut angle = rw_centerangle.wrapping_add(xtoviewangle[x])>>ANGLETOFINESHIFT;

            if angle >= (FINEANGLES / 2) { // DSB-23
                angle = 0;
            }

            texturecolumn = rw_offset-FixedMul(finetangent[angle as usize],rw_distance);
            texturecolumn >>= FRACBITS;
            // calculate lighting
            let index = i32::min(rw_scale>>LIGHTSCALESHIFT,
                                 (MAXLIGHTSCALE-1) as i32);

            dc_colormap = *walllights.offset(index as isize);
            dc_x = rw_x;
            dc_iscale = ((0xffffffff as u32) / (rw_scale as u32)) as i32;
        }
        
        // draw the wall tiers
        if midtexture != 0 {
            // single sided line
            dc_yl = yl;
            dc_yh = yh;
            dc_texturemid = rw_midtexturemid;
            dc_source = R_GetColumn(midtexture,texturecolumn);
            colfunc ();
            ceilingclip[x] = viewheight as i16;
            floorclip[x] = -1;
        } else {
            // two sided line
            if toptexture != 0 {
                // top wall
                let mid = i32::min(pixhigh>>HEIGHTBITS,
                                   (floorclip[x]-1) as i32);
                pixhigh += pixhighstep;

                if mid >= yl {
                    dc_yl = yl;
                    dc_yh = mid;
                    dc_texturemid = rw_toptexturemid;
                    dc_source = R_GetColumn(toptexture,texturecolumn);
                    colfunc ();
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
                let mid = i32::max((pixlow+HEIGHTUNIT-1)>>HEIGHTBITS,
                                   (ceilingclip[x]+1) as i32);
                pixlow += pixlowstep;

                if mid <= yh {
                    dc_yl = mid;
                    dc_yh = yh;
                    dc_texturemid = rw_bottomtexturemid;
                    dc_source = R_GetColumn(bottomtexture,
                                texturecolumn);
                    colfunc ();
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
                    
            if maskedtexture != 0 {
                // save texturecol
                //  for backdrawing of masked mid texture
                *maskedtexturecol.offset(x as isize) = texturecolumn as i16;
            }
        }
            
        rw_scale += rw_scalestep;
        topfrac += topstep;
        bottomfrac += bottomstep;
    }
}

//
// R_StoreWallRange
// A wall segment will be drawn
//  between start and stop pixels (inclusive).
//
#[no_mangle]
pub unsafe extern "C" fn R_StoreWallRange (start: i32, stop: i32) {
    //fixed_t		hyp;
    //fixed_t		sineval;
    //angle_t		distangle, offsetangle;
    //fixed_t		vtop;
    //int			lightnum;

    // don't overflow and crash
    if ds_p == drawsegs.as_mut_ptr().offset(MAXDRAWSEGS as isize) {
        return;
    }
        
    if (start >=viewwidth) || (start > stop) {
        panic!("Bad R_RenderWallRange: {} to {}", start , stop);
    }
    
    sidedef = (*curline).sidedef;
    linedef = (*curline).linedef;

    // mark the segment as visible for auto map
    (*linedef).flags |= ML_MAPPED as i16;
    
    // calculate rw_distance for scale calculation
    rw_normalangle = (*curline).angle.wrapping_add(ANG90);
    let offsetangle: angle_t = angle_t::min(ANG90,
                i32::abs(rw_normalangle.wrapping_sub(rw_angle1 as angle_t) as i32) as angle_t);
    
    let distangle: angle_t = ANG90 - offsetangle;
    let hyp: fixed_t = R_PointToDist ((*(*curline).v1).x, (*(*curline).v1).y);
    let sineval: fixed_t = finesine[(distangle>>ANGLETOFINESHIFT) as usize];
    rw_distance = FixedMul (hyp, sineval);
        
    
    rw_x = start;
    (*ds_p).x1 = start;
    (*ds_p).x2 = stop;
    (*ds_p).curline = curline;
    rw_stopx = stop+1;
    
    // calculate scale at both ends and step
    rw_scale = R_ScaleFromGlobalAngle (viewangle.wrapping_add(xtoviewangle[start as usize]));
    (*ds_p).scale1 = rw_scale;
    
    if stop > start {
        (*ds_p).scale2 = R_ScaleFromGlobalAngle (viewangle.wrapping_add(xtoviewangle[stop as usize]));
        rw_scalestep = ((*ds_p).scale2 - rw_scale) / (stop-start);
        (*ds_p).scalestep = rw_scalestep;
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
        (*ds_p).scale2 = (*ds_p).scale1;
    }
    
    // calculate texture boundaries
    //  and decide if floor / ceiling marks are needed
    worldtop = (*frontsector).ceilingheight - viewz;
    worldbottom = (*frontsector).floorheight - viewz;
    
    midtexture = 0;
    toptexture = 0;
    bottomtexture = 0;
    maskedtexture = 0;
    (*ds_p).maskedtexturecol = std::ptr::null_mut();
    
    if backsector == std::ptr::null_mut() {
        // single sided line
        midtexture = *texturetranslation.offset((*sidedef).midtexture as isize);
        // a single sided line is terminal, so it must mark ends
        markfloor = c_true;
        markceiling = c_true;
        if ((*linedef).flags & (ML_DONTPEGBOTTOM as i16)) != 0 {
            let vtop = (*frontsector).floorheight +
                *textureheight.offset((*sidedef).midtexture as isize);
            // bottom of texture at bottom
            rw_midtexturemid = vtop - viewz;	
        } else {
            // top of texture at top
            rw_midtexturemid = worldtop;
        }
        rw_midtexturemid += (*sidedef).rowoffset;

        (*ds_p).silhouette = SIL_BOTH as i32;
        (*ds_p).sprtopclip = screenheightarray.as_mut_ptr();
        (*ds_p).sprbottomclip = negonearray.as_mut_ptr();
        (*ds_p).bsilheight = MAXINT;
        (*ds_p).tsilheight = MININT;
    } else {
        // two sided line
        (*ds_p).sprtopclip = std::ptr::null_mut();
        (*ds_p).sprbottomclip = std::ptr::null_mut();
        (*ds_p).silhouette = 0;
        
        if (*frontsector).floorheight > (*backsector).floorheight {
            (*ds_p).silhouette = SIL_BOTTOM as i32;
            (*ds_p).bsilheight = (*frontsector).floorheight;
        } else if (*backsector).floorheight > viewz {
            (*ds_p).silhouette = SIL_BOTTOM as i32;
            (*ds_p).bsilheight = MAXINT;
            // (*ds_p).sprbottomclip = negonearray;
        }
        
        if (*frontsector).ceilingheight < (*backsector).ceilingheight {
            (*ds_p).silhouette |= SIL_TOP as i32;
            (*ds_p).tsilheight = (*frontsector).ceilingheight;
        } else if (*backsector).ceilingheight < viewz {
            (*ds_p).silhouette |= SIL_TOP as i32;
            (*ds_p).tsilheight = MININT;
            // (*ds_p).sprtopclip = screenheightarray;
        }
            
        if (*backsector).ceilingheight <= (*frontsector).floorheight {
            (*ds_p).sprbottomclip = negonearray.as_mut_ptr();
            (*ds_p).bsilheight = MAXINT;
            (*ds_p).silhouette |= SIL_BOTTOM as i32;
        }
        
        if (*backsector).floorheight >= (*frontsector).ceilingheight {
            (*ds_p).sprtopclip = screenheightarray.as_mut_ptr();
            (*ds_p).tsilheight = MININT;
            (*ds_p).silhouette |= SIL_TOP as i32;
        }
        
        worldhigh = (*backsector).ceilingheight - viewz;
        worldlow = (*backsector).floorheight - viewz;
            
        // hack to allow height changes in outdoor areas
        if ((*frontsector).ceilingpic == (skyflatnum as i16))
        && ((*backsector).ceilingpic == (skyflatnum as i16)) {
            worldtop = worldhigh;
        }
        
                
        if (worldlow != worldbottom)
        || ((*backsector).floorpic != (*frontsector).floorpic)
        || ((*backsector).lightlevel != (*frontsector).lightlevel) {
            markfloor = c_true;
        } else {
            // same plane on both sides
            markfloor = c_false;
        }
        
                
        if (worldhigh != worldtop)
        || ((*backsector).ceilingpic != (*frontsector).ceilingpic)
        || ((*backsector).lightlevel != (*frontsector).lightlevel) {
            markceiling = c_true;
        } else {
            // same plane on both sides
            markceiling = c_false;
        }
        
        if ((*backsector).ceilingheight <= (*frontsector).floorheight)
        || ((*backsector).floorheight >= (*frontsector).ceilingheight) {
            // closed door
            markceiling = c_true;
            markfloor = c_true;
        }
        

        if worldhigh < worldtop {
            // top texture
            toptexture = *texturetranslation.offset((*sidedef).toptexture as isize);
            if ((*linedef).flags & (ML_DONTPEGTOP as i16)) != 0 {
                // top of texture at top
                rw_toptexturemid = worldtop;
            } else {
                let vtop = (*backsector).ceilingheight
                    + *textureheight.offset((*sidedef).toptexture as isize);
            
                // bottom of texture
                rw_toptexturemid = vtop - viewz;	
            }
        }
        if worldlow > worldbottom {
            // bottom texture
            bottomtexture = *texturetranslation.offset((*sidedef).bottomtexture as isize);

            if ((*linedef).flags & (ML_DONTPEGBOTTOM as i16)) != 0 {
                // bottom of texture at bottom
                // top of texture at top
                rw_bottomtexturemid = worldtop;
            } else { // top of texture at top
                rw_bottomtexturemid = worldlow;
            }
        }
        rw_toptexturemid += (*sidedef).rowoffset;
        rw_bottomtexturemid += (*sidedef).rowoffset;
        
        // allocate space for masked texture tables
        if (*sidedef).midtexture != 0 {
            // masked midtexture
            maskedtexture = c_true;
            maskedtexturecol = lastopening.offset(-(rw_x as isize));
            (*ds_p).maskedtexturecol = maskedtexturecol;
            lastopening = lastopening.offset((rw_stopx - rw_x) as isize);
        }
    }
    
    // calculate rw_offset (only needed for textured lines)
    segtextured = midtexture | toptexture | bottomtexture | maskedtexture;

    if segtextured != c_false {
        let mut offsetangle = rw_normalangle.wrapping_sub(rw_angle1 as angle_t);
        
        if offsetangle > ANG180 {
            offsetangle = (0 as angle_t).wrapping_sub(offsetangle); // DSB-20
        }

        if offsetangle > ANG90 {
            offsetangle = ANG90;
        }

        let sineval = finesine[(offsetangle >>ANGLETOFINESHIFT) as usize];
        rw_offset = FixedMul (hyp, sineval);

        if (rw_normalangle.wrapping_sub(rw_angle1 as angle_t)) < ANG180 {
            rw_offset = -rw_offset;
        }

        rw_offset += (*sidedef).textureoffset + (*curline).offset;
        rw_centerangle = ANG90.wrapping_add(viewangle).wrapping_sub(rw_normalangle);
        
        // calculate light table
        //  use different light tables
        //  for horizontal / vertical / diagonal
        // OPTIMIZE: get rid of LIGHTSEGSHIFT globally
        if fixedcolormap == std::ptr::null_mut() {
            let mut lightnum = (((*frontsector).lightlevel >> LIGHTSEGSHIFT) as i32) + extralight;

            if (*(*curline).v1).y == (*(*curline).v2).y {
                lightnum -= 1;
            } else if (*(*curline).v1).x == (*(*curline).v2).x {
                lightnum += 1;
            }

            walllights = scalelight[i32::max(0, i32::min((LIGHTLEVELS - 1) as i32, lightnum)) as usize].as_mut_ptr();
        }
    }
    
    // if a floor / ceiling plane is on the wrong side
    //  of the view plane, it is definitely invisible
    //  and doesn't need to be marked.
    
  
    if (*frontsector).floorheight >= viewz {
        // above view plane
        markfloor = c_false;
    }
    
    if ((*frontsector).ceilingheight <= viewz)
    && (((*frontsector).ceilingpic as i32) != skyflatnum) {
        // below view plane
        markceiling = c_false;
    }

    
    // calculate incremental stepping values for texture edges
    worldtop >>= 4;
    worldbottom >>= 4;
    
    topstep = -FixedMul (rw_scalestep, worldtop);
    topfrac = (centeryfrac>>4) - FixedMul (worldtop, rw_scale);

    bottomstep = -FixedMul (rw_scalestep,worldbottom);
    bottomfrac = (centeryfrac>>4) - FixedMul (worldbottom, rw_scale);
    
    if backsector != std::ptr::null_mut() {
        worldhigh >>= 4;
        worldlow >>= 4;

        if worldhigh < worldtop {
            pixhigh = (centeryfrac>>4) - FixedMul (worldhigh, rw_scale);
            pixhighstep = -FixedMul (rw_scalestep,worldhigh);
        }
        
        if worldlow > worldbottom {
            pixlow = (centeryfrac>>4) - FixedMul (worldlow, rw_scale);
            pixlowstep = -FixedMul (rw_scalestep,worldlow);
        }
    }
    
    // render it
    if markceiling != c_false {
        ceilingplane = R_CheckPlane (ceilingplane, rw_x, rw_stopx-1);
    }
    
    if markfloor != c_false {
        floorplane = R_CheckPlane (floorplane, rw_x, rw_stopx-1);
    }

    R_RenderSegLoop ();

    
    // save sprite clipping info
    if ((0 != ((*ds_p).silhouette & (SIL_TOP as i32)))
        || (maskedtexture != c_false))
    && ((*ds_p).sprtopclip == std::ptr::null_mut()) {
        memcpy (lastopening as *mut u8,
                ceilingclip.as_mut_ptr().offset(start as isize) as *const u8,
                2*(rw_stopx-start) as usize);
        (*ds_p).sprtopclip = lastopening.offset(-(start as isize));
        lastopening = lastopening.offset((rw_stopx - start) as isize);
    }
    
    if ((0 != ((*ds_p).silhouette & (SIL_BOTTOM as i32)))
        || (maskedtexture != c_false))
    && ((*ds_p).sprbottomclip == std::ptr::null_mut()) {
        memcpy (lastopening as *mut u8,
                floorclip.as_mut_ptr().offset(start as isize) as *const u8,
                2*(rw_stopx-start) as usize);
        (*ds_p).sprbottomclip = lastopening.offset(-(start as isize));
        lastopening = lastopening.offset((rw_stopx - start) as isize);
    }

    if (maskedtexture != c_false)
    && (0 == ((*ds_p).silhouette & (SIL_TOP as i32))) {
        (*ds_p).silhouette |= SIL_TOP as i32;
        (*ds_p).tsilheight = MININT;
    }
    if (maskedtexture != c_false)
    && (0 == ((*ds_p).silhouette & (SIL_BOTTOM as i32))) {
        (*ds_p).silhouette |= SIL_BOTTOM as i32;
        (*ds_p).bsilheight = MAXINT;
    }
    ds_p = ds_p.offset(1);
}

