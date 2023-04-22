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
// Revision 1.3  1997/01/29 20:10
// DESCRIPTION:
//	Preparation of data for rendering,
//	generation of lookups, caching, retrieval by name.
//
//-----------------------------------------------------------------------------

use crate::defs::*;
use crate::globals::*;
use crate::funcs::*;


//
// Graphics.
// DOOM graphics for walls and sprites
// is stored in vertical runs of opaque pixels (posts).
// A column is composed of zero or more posts,
// a patch or sprite is composed of zero or more columns.
// 






//
// MAPTEXTURE_T CACHING
// When a texture is first needed,
//  it counts the number of composite columns
//  required in the texture and allocates space
//  for a column directory and any new columns.
// The directory will simply point inside other patches
//  if there is only one patch in a given column,
//  but any columns with multiple patches
//  will have new column_ts generated.
//



//
// R_DrawColumnInCache
// Clip and draw a column
//  from a patch into a cached post.
//
unsafe fn R_DrawColumnInCache(
        ppatch: *mut column_t,
        cache: *mut u8,
        originy: i32,
        cacheheight: i32) {

    let mut patch = ppatch;

    while (*patch).topdelta != 0xff {
        let source: *mut u8 = (patch as *mut u8).offset(3);
        let mut count: i32 = (*patch).length as i32;
        let mut position: i32 = originy + (*patch).topdelta as i32;

        if position < 0 {
            count += position;
            position = 0;
        }

        if (position + count) > cacheheight {
            count = cacheheight - position;
        }

        if count > 0 {
            memcpy (cache.offset(position as isize), source, count as usize);
        }

        patch = (patch as *mut u8).offset(((*patch).length + 4) as isize)
                    as *mut column_t;
    }
}


//
// R_GenerateComposite
// Using the texture definition,
//  the composite texture is created from the patches,
//  and each column is cached.
//
unsafe fn R_GenerateComposite (texnum: i32) {

    let texture: *mut texture_t = *textures.offset(texnum as isize);

    const pad_size: i32 = 128;
    let unpadded_size: i32 = *texturecompositesize.offset(texnum as isize);
    let block: *mut u8 = Z_Malloc
        (unpadded_size + pad_size, // DSB-21
          PU_STATIC as i32,
          texturecomposite.offset(texnum as isize).as_mut().unwrap());
    memset (block.offset(unpadded_size as isize), 0, pad_size as usize);
    assert!(*texturecomposite.offset(texnum as isize) == block);
    let collump: *mut i16 = *texturecolumnlump.offset(texnum as isize);
    let colofs: *mut u16 = *texturecolumnofs.offset(texnum as isize);

    // Composite the columns together.
    let mut patch: *mut texpatch_t = (*texture).patches.as_mut_ptr();

    for _ in 0 .. (*texture).patchcount {
        let realpatch: *mut patch_t = W_CacheLumpNum ((*patch).patch, PU_CACHE);
        let x1: i32 = (*patch).originx;
        let x2: i32 = i32::min(x1 + i16::from_le((*realpatch).width) as i32,
                               (*texture).width as i32);

        for x in i32::max(0, x1) .. x2 {
            // Column does not have multiple patches?
            if *collump.offset(x as isize) >= 0 {
                continue;
            }

            let patchcol: *mut column_t =
                (realpatch as *mut u8).offset(
                    i32::from_le(*(*realpatch).columnofs.as_ptr().
                                    offset((x - x1) as isize)) as isize)
                        as *mut column_t;
            R_DrawColumnInCache (patchcol,
                     block.offset(*colofs.offset(x as isize) as isize),
                     (*patch).originy,
                     (*texture).height as i32);
        }
        patch = patch.offset(1);
    }

    // Now that the texture has been built in column cache,
    //  it is purgable from zone memory.
    Z_ChangeTag2 (block, PU_CACHE as i32);
}

//
// R_GenerateLookup
//
#[no_mangle]
pub unsafe extern "C" fn R_GenerateLookup (texnum: i32) {
    
    let texture: *mut texture_t = *textures.offset(texnum as isize);

    // Composited texture not created yet.
    *texturecomposite.offset(texnum as isize) = std::ptr::null_mut();
    
    let mut size: i32 = 0;
    let collump: *mut i16 = *texturecolumnlump.offset(texnum as isize);
    let colofs: *mut u16 = *texturecolumnofs.offset(texnum as isize);
    
    // Now count the number of columns
    //  that are covered by more than one patch.
    // Fill in the lump / offset, so columns
    //  with only a single patch are all done.
    let mut patchcount = [0 as u8; 256];
    assert!((*texture).width <= 256);
    let mut patch: *mut texpatch_t = (*texture).patches.as_mut_ptr();

    for _ in 0 .. (*texture).patchcount {
        let realpatch: *mut patch_t = W_CacheLumpNum ((*patch).patch, PU_CACHE);
        let x1: i32 = (*patch).originx;
        let x2: i32 = i32::min(x1 + i16::from_le((*realpatch).width) as i32,
                               (*texture).width as i32);

        for x in i32::max(0, x1) .. x2 {
            patchcount[x as usize] += 1;
            *collump.offset(x as isize) = (*patch).patch as i16;
            *colofs.offset(x as isize) = i32::from_le(*(*realpatch).columnofs.as_ptr().
                                    offset((x - x1) as isize)) as u16 + 3;
        }
        patch = patch.offset(1);
    }
    for x in 0 .. (*texture).width {
        if patchcount[x as usize] == 0 {
            panic!("R_GenerateLookup: column without a patch ({})\n",
                    std::ffi::CStr::from_ptr((*texture).name.as_ptr()).to_str().unwrap());
        }
        // I_Error ("R_GenerateLookup: column without a patch");

        if patchcount[x as usize] > 1 {
            // Use the cached block.
            *collump.offset(x as isize) = -1;
            *colofs.offset(x as isize) = size as u16;
            size += (*texture).height as i32;

            if size > 0x10000 {
                panic!("R_GenerateLookup: texture {} is >64k", texnum);
            }
        }
    }	
    *texturecompositesize.offset(texnum as isize) = size;
}

//
// R_GetColumn
//
#[no_mangle]
pub unsafe extern "C" fn R_GetColumn(tex: i32, pcol: i32) -> *mut u8 {
    let col: i32 = pcol & (*texturewidthmask.offset(tex as isize) as i32);
    let collump: *mut i16 = *texturecolumnlump.offset(tex as isize);
    let colofs: *mut u16 = *texturecolumnofs.offset(tex as isize);
    let lump: i16 = *collump.offset(col as isize);
    let ofs: u16 = *colofs.offset(col as isize);

    if lump > 0 {
        return (W_CacheLumpNum(lump as i32, PU_CACHE) as *mut u8).offset(ofs as isize);
    }

    if *texturecomposite.offset(tex as isize) == std::ptr::null_mut() {
        R_GenerateComposite (tex);
    }

    return (*texturecomposite.offset(tex as isize)).offset(ofs as isize);
}

