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
use crate::r_main;
use crate::w_wad::W_CacheLumpNum;
use crate::w_wad::W_CacheLumpName;
use crate::w_wad::W_CheckNumForName;
use crate::w_wad::W_LumpLength;
use crate::w_wad::W_GetNumForName;


pub const COLORMAP_SIZE: usize = 256;
pub type colormap_index_t = u16;
pub const NULL_COLORMAP: colormap_index_t = colormap_index_t::MAX;
pub const WAD_NUMCOLORMAPS: usize = 34; // NUMCOLORMAPS is 32, this is not correct, the WAD has 34.
pub type colormaps_t = [u8; WAD_NUMCOLORMAPS * COLORMAP_SIZE];

// A maptexturedef_t describes a rectangular texture,
//  which is composed of one or more mappatch_t structures
//  that arrange graphic patches.
struct texture_t {
    // Keep name for switch changing, etc.
    name: String,
    width: i16,
    height: i16,
    // All the patches[patchcount]
    //  are drawn back to front into the cached texture.
    patches: Vec<texpatch_t>,
    // These were previously in separate arrays
    texturecompositesize: i32,
    texturewidthmask: i32,
    texturecolumnlump: Vec<i16>,    // Lump containing each column (if not generated)
    texturecolumnofs: Vec<u16>,     // Offset of each column in texturecolumnofs or lump
    texturecomposite: Vec<u8>,      // Contains a patched texture once generated
}

pub struct sprite_t {
    pub offset: fixed_t,
    pub topoffset: fixed_t,
    pub width: fixed_t,
}

pub struct RenderData_t {
    pub colormaps: colormaps_t,
    pub firstflat: i32,
    lastflat: i32,
    numflats: i32,
    textures: Vec<texture_t>,
    pub lastspritelump: i32,
    pub sprite: Vec<sprite_t>,
}

pub const empty_RenderData: RenderData_t = RenderData_t {
    colormaps: [0; WAD_NUMCOLORMAPS * COLORMAP_SIZE],
    textures: Vec::new(),
    firstflat: 0,
    lastflat: 0,
    numflats: 0,
    lastspritelump: 0,
    sprite: Vec::new(),
};


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
        cache: &mut Vec<u8>,
        cache_ofs: u16,
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
            for i in 0 .. count as usize {
                *cache.get_mut((position as usize) +
                                (i as usize) + (cache_ofs as usize)).unwrap() =
                    *source.offset(i as isize);
            }
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
unsafe fn R_GenerateComposite (rd: &mut RenderData_t, texnum: i32) {

    let texture: &mut texture_t = rd.textures.get_mut(texnum as usize).unwrap();

    assert!(texture.texturecomposite.is_empty());
    let collump = &mut texture.texturecolumnlump;
    let colofs = &mut texture.texturecolumnofs;

    // Composite the columns together.
    for patch in texture.patches.iter() {
        let realpatch: *mut patch_t = W_CacheLumpNum (patch.patch, PU_CACHE) as *mut patch_t;
        let x1: i32 = patch.originx;
        let x2: i32 = i32::min(x1 + i16::from_le((*realpatch).width) as i32,
                               texture.width as i32);

        // Ensure there is enough space in the texturecomposite vector
        // Pad with 128 zero bytes due to DSB-21
        let end = ((x2 as usize) * (texture.height as usize)) + 128;
        if texture.texturecomposite.len() < end {
            texture.texturecomposite.resize(end, 0);
        }

        for x in i32::max(0, x1) .. x2 {
            // Column does not have multiple patches?
            if *collump.get(x as usize).unwrap() >= 0 {
                continue;
            }

            let patchcol: *mut column_t =
                (realpatch as *mut u8).offset(
                    i32::from_le(*(*realpatch).columnofs.as_ptr().
                                    offset((x - x1) as isize)) as isize)
                        as *mut column_t;
            let ofs = *colofs.get(x as usize).unwrap();
            R_DrawColumnInCache (patchcol,
                     &mut texture.texturecomposite,
                     ofs,
                     patch.originy,
                     texture.height as i32);
        }
    }
}

//
// R_GenerateLookup
//
unsafe fn R_GenerateLookup (rd: &mut RenderData_t, texnum: i32) {
    
    let texture: &mut texture_t = rd.textures.get_mut(texnum as usize).unwrap();

    // Composited texture not created yet.
    texture.texturecomposite.clear();
    
    let mut size: i32 = 0;
    let collump = &mut texture.texturecolumnlump;
    let colofs = &mut texture.texturecolumnofs;

    // Now count the number of columns
    //  that are covered by more than one patch.
    // Fill in the lump / offset, so columns
    //  with only a single patch are all done.
    let mut patchcount = [0 as u8; 256];
    assert!((*texture).width <= 256);

    for patch in texture.patches.iter() {
        let realpatch: *mut patch_t = W_CacheLumpNum (patch.patch, PU_CACHE) as *mut patch_t;
        let x1: i32 = patch.originx;
        let x2: i32 = i32::min(x1 + i16::from_le((*realpatch).width) as i32,
                               texture.width as i32);

        for x in i32::max(0, x1) .. x2 {
            patchcount[x as usize] += 1;
            *collump.get_mut(x as usize).unwrap() = patch.patch as i16;
            *colofs.get_mut(x as usize).unwrap() = i32::from_le(*(*realpatch).columnofs.as_ptr().
                                    offset((x - x1) as isize)) as u16 + 3;
        }
    }
    for x in 0 .. texture.width {
        if patchcount[x as usize] == 0 {
            panic!("R_GenerateLookup: column without a patch ({})\n", texture.name);
        }
        // I_Error ("R_GenerateLookup: column without a patch");

        if patchcount[x as usize] > 1 {
            // Use the cached block.
            *collump.get_mut(x as usize).unwrap() = -1;
            *colofs.get_mut(x as usize).unwrap() = size as u16;
            size += texture.height as i32;

            if size > 0x10000 {
                panic!("R_GenerateLookup: texture {} is >64k", texnum);
            }
        }
    }        
    texture.texturecompositesize = size;
}

//
// R_GetColumn
//
pub unsafe fn R_GetColumn(rd: &mut RenderData_t, tex: i32, pcol: i32) -> *mut u8 {
    let texture: &texture_t = rd.textures.get(tex as usize).unwrap();

    let col: i32 = pcol & texture.texturewidthmask;
    let collump = &texture.texturecolumnlump;
    let colofs = &texture.texturecolumnofs;
    let lump: i16 = *collump.get(col as usize).unwrap();
    let ofs: u16 = *colofs.get(col as usize).unwrap();

    if lump > 0 {
        return (W_CacheLumpNum(lump as i32, PU_CACHE) as *mut u8).offset(ofs as isize);
    }

    if texture.texturecomposite.is_empty() {
        R_GenerateComposite (rd, tex);
    }

    return rd.textures.get_mut(tex as usize).unwrap().texturecomposite.as_mut_ptr().offset(ofs as isize);
}

//
// R_InitTextures
// Initializes the texture list
//  with the textures from the world map.
//
unsafe fn R_InitTextures (rd: &mut RenderData_t) {
    
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
    let numtextures = numtextures1 + numtextures2;

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
        let patchcount = i16::from_le((*mtexture).patchcount);

        let mut texture = texture_t {
            width: i16::from_le((*mtexture).width),
            height: i16::from_le((*mtexture).height),
            name: W_Name((*mtexture).name.as_mut_ptr() as *const u8).to_uppercase(),
            patches: Vec::new(),
            texturecompositesize: 0,
            texturewidthmask: 0,
            texturecolumnlump: Vec::new(),
            texturecomposite: Vec::new(),
            texturecolumnofs: Vec::new(),
        };

        let mut mpatch: *mut mappatch_t = (*mtexture).patches.as_mut_ptr();

        for _ in 0 .. patchcount {
            let patch = texpatch_t {
                originx: i16::from_le((*mpatch).originx) as i32,
                originy: i16::from_le((*mpatch).originy) as i32,
                patch: *patchlookup.get(i16::from_le((*mpatch).patch) as usize).unwrap(),
            };
            if patch.patch == -1 {
                panic!("R_InitTextures: Missing patch in texture {}", texture.name);
            }
            mpatch = mpatch.offset(1);
            texture.patches.push(patch);
        }

        let mut j: i32 = 1;
        while (j * 2) <= (texture.width as i32) {
            j<<=1;
        }

        texture.texturewidthmask = j-1;
        *textureheight.offset(i as isize) = (texture.height as i32) << FRACBITS;
        texture.texturecolumnlump.resize(texture.width as usize, 0);
        texture.texturecolumnofs.resize(texture.width as usize, 0);
        rd.textures.push(texture);
                
        directory = directory.offset(1);
    }

    Z_Free (maptex1 as *mut u8);
    if maptex2 != std::ptr::null_mut() {
        Z_Free (maptex2 as *mut u8);
    }
    
    // Precalculate whatever possible.
    for i in 0 .. numtextures {
        R_GenerateLookup (rd, i);
    }
    
    // Create translation table for global animation.
    texturetranslation =
        Z_Malloc ((numtextures + 1)*sizeof_ptr, PU_STATIC, std::ptr::null_mut()) as *mut i32;
    
    for i in 0 .. numtextures {
        *texturetranslation.offset(i as isize) = i;
    }
}

//
// R_InitFlats
//
unsafe fn R_InitFlats (rd: &mut RenderData_t) {
        
    rd.firstflat = W_GetNumForName ("F_START\0".as_ptr()) + 1;
    rd.lastflat = W_GetNumForName ("F_END\0".as_ptr()) - 1;
    rd.numflats = rd.lastflat - rd.firstflat + 1;
        
    // Create translation table for global animation.
    flattranslation = Z_Malloc ((rd.numflats+1)*sizeof_ptr, PU_STATIC, std::ptr::null_mut()) as *mut i32;
   
    for i in 0 .. rd.numflats {
        *flattranslation.offset(i as isize) = i;
    }
}


//
// R_InitSpriteLumps
// Finds the width and hoffset of all sprites in the wad,
//  so the sprite does not need to be cached completely
//  just for having the header info ready during rendering.
//
unsafe fn R_InitSpriteLumps (rd: &mut RenderData_t) {
        
    firstspritelump = W_GetNumForName ("S_START\0".as_ptr()) + 1;
    rd.lastspritelump = W_GetNumForName ("S_END\0".as_ptr()) - 1;
    
    let numspritelumps = rd.lastspritelump - firstspritelump + 1;
        
    for i in 0 .. numspritelumps {
        if 0 == (i&63) {
            print!(".");
        }

        let patch: *mut patch_t = W_CacheLumpNum (firstspritelump + i, PU_CACHE) as *mut patch_t;
        rd.sprite.push(sprite_t {
            width: (i16::from_le((*patch).width) as i32) << FRACBITS,
            offset: (i16::from_le((*patch).leftoffset) as i32) << FRACBITS,
            topoffset: (i16::from_le((*patch).topoffset) as i32) << FRACBITS,
        });
    }
}

//
// R_InitColormaps
//
unsafe fn R_InitColormaps (rd: &mut RenderData_t) {
    // Load in the light tables, 
    let lump = W_GetNumForName("COLORMAP\0".as_ptr()); 
    let length = W_LumpLength (lump) as usize;
    let available_memory = std::mem::size_of::<colormaps_t>();
    if length > available_memory {
        panic!("Unable to load COLORMAP (size {}) into vc.colormaps (size {})",
                length, available_memory);
    }
    W_ReadLump (lump, rd.colormaps.as_mut_ptr()); 
}



//
// R_InitData
// Locates all the lumps
//  that will be used by all views
// Must be called after W_Init.
//
pub unsafe fn R_InitData (rd: &mut RenderData_t) {
    R_InitTextures (rd);
    print!("\nInitTextures");
    R_InitFlats (rd);
    print!("\nInitFlats");
    R_InitSpriteLumps (rd);
    print!("\nInitSprites");
    R_InitColormaps (rd);
    print!("\nInitColormaps");
}


//
// R_FlatNumForName
// Retrieval, get a flat number for a flat name.
//
#[no_mangle] // called from P_LoadSectors
pub unsafe extern "C" fn R_FlatNumForName (name: *const u8) -> i32 {
    let rd = &mut r_main::remove_this_rc_global.rd;
    let i = W_CheckNumForName (name);

    if i == -1 {
        panic!("R_FlatNumForName: {} not found", W_Name(name));
    }
    return i - rd.firstflat;
}




//
// R_CheckTextureNumForName
// Check whether texture is available.
// Filter out NoTexture indicator.
//
#[no_mangle] // called from P_InitPicAnims
pub unsafe extern "C" fn R_CheckTextureNumForName (name: *const u8) -> i32 {

    let rd = &mut r_main::remove_this_rc_global.rd;

    // "NoTexture" marker.
    if *name.offset(0) == ('-' as u8) {
        return 0;
    }

    let find = W_Name(name).to_uppercase();
    
    for i in 0 .. rd.textures.len() {
        if find == rd.textures.get(i).unwrap().name {
            return i as i32;
        }
    }
                
    return -1;
}



//
// R_TextureNumForName
// Calls R_CheckTextureNumForName,
//  aborts with error message.
//
#[no_mangle] // called from P_LoadSideDefs
pub unsafe extern "C" fn R_TextureNumForName (name: *const u8) -> i32 {
        
    let i = R_CheckTextureNumForName (name);

    if i == -1 {
        panic!("R_TextureNumForName: {} not found", W_Name(name));
    }
    return i;
}


#[no_mangle] // called from P_LoadSideDefs
pub unsafe extern "C" fn R_PrecacheLevel () {
    if demoplayback == c_false {
        panic!("No implementation for R_PrecacheLevel");
    }
}
