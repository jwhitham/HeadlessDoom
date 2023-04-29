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
          PU_STATIC,
          texturecomposite.offset(texnum as isize).as_mut().unwrap());
    memset (block.offset(unpadded_size as isize), 0, pad_size as usize);
    assert!(*texturecomposite.offset(texnum as isize) == block);
    let collump: *mut i16 = *texturecolumnlump.offset(texnum as isize);
    let colofs: *mut u16 = *texturecolumnofs.offset(texnum as isize);

    // Composite the columns together.
    let mut patch: *mut texpatch_t = (*texture).patches.as_mut_ptr();

    for _ in 0 .. (*texture).patchcount {
        let realpatch: *mut patch_t = W_CacheLumpNum ((*patch).patch, PU_CACHE) as *mut patch_t;
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
    Z_ChangeTag2 (block, PU_CACHE);
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
        let realpatch: *mut patch_t = W_CacheLumpNum ((*patch).patch, PU_CACHE) as *mut patch_t;
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
            panic!("R_GenerateLookup: column without a patch ({})\n", W_Name((*texture).name.as_ptr()));
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

//
// R_InitTextures
// Initializes the texture list
//  with the textures from the world map.
//
#[no_mangle]
pub unsafe extern "C" fn R_InitTextures () {
    
    // Load the patch names from pnames.lmp.
    let mut name: [u8; 9] = [0; 9];
    let names: *mut u8 = W_CacheLumpName ("PNAMES\0".as_ptr(), PU_STATIC);
    let nummappatches: i32 = i32::from_le(*(names as *mut i32));
    let name_p: *mut u8 = names.offset(4);
    let mut patchlookup: Vec<i32> = Vec::new();
    
    for i in 0 .. nummappatches {
        memcpy (name.as_mut_ptr(), name_p.offset((i as isize) * 8), 8);
        patchlookup.push(W_CheckNumForName (name.as_ptr()));
    }
    Z_Free (names);
    
    // Load the map texture definitions from textures.lmp.
    // The data is contained in one or two lumps,
    //  TEXTURE1 for shareware, plus TEXTURE2 for commercial.
    let maptex1: *mut i32 = W_CacheLumpName ("TEXTURE1\0".as_ptr(), PU_STATIC) as *mut i32;
    let mut maptex: *mut i32 = maptex1;
    let numtextures1: i32 = i32::from_le(*maptex);
    let mut maxoff: i32 = W_LumpLength (W_GetNumForName ("TEXTURE1\0".as_ptr()));
    let mut directory: *mut i32 = maptex.offset(1);
    let mut maptex2: *mut i32 = std::ptr::null_mut();
    let mut numtextures2: i32 = 0;
    let mut maxoff2: i32 = 0;

    if W_CheckNumForName ("TEXTURE2\0".as_ptr()) != -1 {
        maptex2 = W_CacheLumpName ("TEXTURE2\0".as_ptr(), PU_STATIC) as *mut i32;
        numtextures2 = i32::from_le(*maptex2);
        maxoff2 = W_LumpLength (W_GetNumForName ("TEXTURE2\0".as_ptr()));
    }
    numtextures = numtextures1 + numtextures2;

    let sizeof_ptr = std::mem::size_of::<*mut u8>() as i32;
    textures = Z_Malloc (numtextures * sizeof_ptr, PU_STATIC, std::ptr::null_mut()) as *mut *mut texture_t;
    texturecolumnlump = Z_Malloc (numtextures * sizeof_ptr, PU_STATIC, std::ptr::null_mut()) as *mut *mut i16;
    texturecolumnofs = Z_Malloc (numtextures * sizeof_ptr, PU_STATIC, std::ptr::null_mut()) as *mut *mut u16;
    texturecomposite = Z_Malloc (numtextures * sizeof_ptr, PU_STATIC, std::ptr::null_mut()) as *mut *mut u8;
    texturecompositesize = Z_Malloc (numtextures * sizeof_ptr, PU_STATIC, std::ptr::null_mut()) as *mut i32;
    texturewidthmask = Z_Malloc (numtextures * sizeof_ptr, PU_STATIC, std::ptr::null_mut()) as *mut i32;
    textureheight = Z_Malloc (numtextures * sizeof_ptr, PU_STATIC, std::ptr::null_mut()) as *mut i32;

    //        Really complex printing shit...
    let temp1: i32 = W_GetNumForName ("S_START\0".as_ptr());  // P_???????
    let temp2: i32 = W_GetNumForName ("S_END\0".as_ptr()) - 1;
    let temp3: i32 = ((temp2-temp1+63)/64) + ((numtextures+63)/64);
    print!("[");
    for _ in 0 .. temp3 {
        print!(" ");
    }
    print!("         ]");
    for _ in 0 .. temp3 {
        print!("\x08");
    }
    print!("\x08\x08\x08\x08\x08\x08\x08\x08\x08\x08");        
       
    for i in 0 .. numtextures {
        if 0 == (i&63) {
            print!(".");
        }

        if i == numtextures1 {
            // Start looking in second texture file.
            maptex = maptex2;
            maxoff = maxoff2;
            directory = maptex.offset(1);
        }
                
        let offset: i32 = i32::from_le(*directory);

        if offset > maxoff {
            panic!("R_InitTextures: bad texture directory");
        }
        
        let mtexture: *mut maptexture_t = (maptex as *mut u8).offset(offset as isize) as *mut maptexture_t;

        let texture: *mut texture_t = 
            Z_Malloc ((std::mem::size_of::<texture_t>() as i32)
                      + ((std::mem::size_of::<texpatch_t>() as i32) *
                            ((i16::from_le((*mtexture).patchcount) as i32) - 1)),
                      PU_STATIC, std::ptr::null_mut()) as *mut texture_t;
        *textures.offset(i as isize) = texture;
        (*texture).width = i16::from_le((*mtexture).width);
        (*texture).height = i16::from_le((*mtexture).height);
        (*texture).patchcount = i16::from_le((*mtexture).patchcount);

        memcpy ((*texture).name.as_mut_ptr() as *mut u8,
                (*mtexture).name.as_mut_ptr() as *const u8, 8);
        let mut mpatch: *mut mappatch_t = (*mtexture).patches.as_mut_ptr();
        let mut patch: *mut texpatch_t = (*texture).patches.as_mut_ptr();

        for _ in 0 .. (*texture).patchcount {
            (*patch).originx = i16::from_le((*mpatch).originx) as i32;
            (*patch).originy = i16::from_le((*mpatch).originy) as i32;
            (*patch).patch = *patchlookup.get(i16::from_le((*mpatch).patch) as usize).unwrap();
            if (*patch).patch == -1 {
                panic!("R_InitTextures: Missing patch in texture {}",
                       W_Name((*texture).name.as_ptr()));
            }
            mpatch = mpatch.offset(1);
            patch = patch.offset(1);
        }
        *texturecolumnlump.offset(i as isize) =
            Z_Malloc (((*texture).width as i32) * 2, PU_STATIC, std::ptr::null_mut()) as *mut i16;
        *texturecolumnofs.offset(i as isize) =
            Z_Malloc (((*texture).width as i32) * 2, PU_STATIC, std::ptr::null_mut()) as *mut u16;

        let mut j: i32 = 1;
        while (j * 2) <= ((*texture).width as i32) {
            j<<=1;
        }

        *texturewidthmask.offset(i as isize) = j-1;
        *textureheight.offset(i as isize) = ((*texture).height as i32) << FRACBITS;
                
        directory = directory.offset(1);
    }

    Z_Free (maptex1 as *mut u8);
    if maptex2 != std::ptr::null_mut() {
        Z_Free (maptex2 as *mut u8);
    }
    
    // Precalculate whatever possible.
    for i in 0 .. numtextures {
        R_GenerateLookup (i);
    }
    
    // Create translation table for global animation.
    texturetranslation =
        Z_Malloc ((numtextures + 1)*sizeof_ptr, PU_STATIC, std::ptr::null_mut()) as *mut i32;
    
    for i in 0 .. numtextures {
        *texturetranslation.offset(i as isize) = i;
    }
}

/*
R_Init: Init DOOM refresh daemon - [.....texture AASTINKY patchcount 2
  x1 0 x2 16 rpw 16 w 24
  x1 12 x2 24 rpw 16 w 24
texture BIGDOOR1 patchcount 5
  x1 0 x2 64 rpw 64 w 128
  x1 0 x2 64 rpw 64 w 128
  x1 17 x2 113 rpw 96 w 128
  x1 113 x2 128 rpw 64 w 128
  x1 113 x2 128 rpw 64 w 128
texture BIGDOOR2 patchcount 1
  x1 0 x2 128 rpw 128 w 128
texture BIGDOOR4 patchcount 1
  x1 0 x2 128 rpw 128 w 128
texture BRNBIGC patchcount 1


*/
