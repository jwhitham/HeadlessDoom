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
const HEIGHTBITS: i32 =	12;
const HEIGHTUNIT: i32 = 1<<HEIGHTBITS;

#[no_mangle]
pub unsafe extern "C" fn R_RenderSegLoop () {
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
