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
