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
//	Refresh of things, i.e. objects represented by sprites.
//
//-----------------------------------------------------------------------------


use crate::defs::*;

extern {
    static mut maxframe: i32;
    static mut sprtemp: sprtemp_t;
    static mut spritename: *mut i8;
    static mut firstspritelump: i32;
    static mut lastspritelump: i32;
}
//
// R_InstallSpriteLump
// Local function for R_InitSprites.
//
unsafe fn R_InstallSpriteLump(
        lump: i32, frame: u32, rotation: u32, flipped: boolean) {
    
    let mut rotation_tmp = rotation as usize;

    if frame >= 29 || rotation_tmp > 8 {
        panic!("R_InstallSpriteLump: Bad frame characters in lump {}", lump);
    }

    if (frame as i32) > maxframe {
        maxframe = frame as i32;
    }
    
    if rotation_tmp == 0 {
        // the lump should be used for all rotations
        if sprtemp[frame as usize].rotate == c_false {
            panic!("R_InitSprites: Sprite {} frame {} has multip rot=0 lump",
                std::ffi::CStr::from_ptr(spritename).to_str().unwrap(),
                char::from_u32(('A' as u32) + frame).unwrap());
        }

        if sprtemp[frame as usize].rotate == c_true {
            panic!("R_InitSprites: Sprite {} frame {} has rotations and a rot=0 lump",
                std::ffi::CStr::from_ptr(spritename).to_str().unwrap(),
                char::from_u32(('A' as u32) + frame).unwrap());
        }
                
        sprtemp[frame as usize].rotate = c_false;
        for r in 0 .. 8 {
            sprtemp[frame as usize].lump[r] = (lump - firstspritelump) as i16;
            sprtemp[frame as usize].flip[r] = flipped as u8;
        }
        return;
    }
    
    // the lump is only used for one rotation
    if sprtemp[frame as usize].rotate == c_false {
        panic!("R_InitSprites: Sprite {} frame {} has rotations and a rot=0 lump",
                std::ffi::CStr::from_ptr(spritename).to_str().unwrap(),
                char::from_u32(('A' as u32) + frame).unwrap());
    }
            
    sprtemp[frame as usize].rotate = c_true;

    // make 0 based
    rotation_tmp -= 1;
    if sprtemp[frame as usize].lump[rotation_tmp] != -1 {
        panic!("R_InitSprites: Sprite {} : {} : {} has two lumps mapped to it",
                std::ffi::CStr::from_ptr(spritename).to_str().unwrap(),
                char::from_u32(('A' as u32) + frame).unwrap(),
                char::from_u32(('1' as u32) + (rotation_tmp as u32)).unwrap());
    }
        
    sprtemp[frame as usize].lump[rotation_tmp] = (lump - firstspritelump) as i16;
    sprtemp[frame as usize].flip[rotation_tmp] = flipped as u8;
}

extern {
    static mut numsprites: i32;
    static mut sprites: *mut spritedef_t;
    static mut lumpinfo: *mut lumpinfo_t;
    static modifiedgame: boolean;
    fn Z_Malloc(size: i32, tag: i32, user: *const u8) -> *mut u8;
    fn memset(s: *mut u8, c: i32, n: usize) -> *mut u8;
    fn memcpy(d: *mut u8, s: *const u8, n: usize) -> *mut u8;
    fn W_GetNumForName (name: *const i8) -> i32;
}
const PU_STATIC: i32 = 1;

//
// R_InitSpriteDefs
// Pass a null terminated list of sprite names
//  (4 chars exactly) to be used.
// Builds the sprite rotation matrixes to account
//  for horizontally flipped sprites.
// Will report an error if the lumps are inconsistant. 
// Only called at startup.
//
// Sprite lump names are 4 characters for the actor,
//  a letter for the frame, and a number for the rotation.
// A sprite that is flippable will have an additional
//  letter/number appended.
// The rotation character can be 0 to signify no rotations.
//
unsafe fn R_InitSpriteDefs (namelist: *mut *mut i8) { 
    // count the number of sprite names
    numsprites = 0;
    for i in 0 .. i32::MAX {
        if (*namelist.offset(i as isize) as *const i8) == std::ptr::null() {
            numsprites = i;
            break;
        }
    }
    
    if numsprites == 0 {
        return;
    }

    sprites = Z_Malloc(numsprites * std::mem::size_of::<spritedef_t>() as i32,
                PU_STATIC, std::ptr::null()) as *mut spritedef_t;
    
    let start = firstspritelump-1;
    let end = lastspritelump+1;
    
    // scan all the lump names for each of the names,
    //  noting the highest frame letter.
    // Just compare 4 characters as ints
    for i in 0 .. numsprites {
        let sprite = sprites.offset(i as isize);
        spritename = *namelist.offset(i as isize);
        memset (sprtemp.as_ptr() as *mut u8,-1, std::mem::size_of::<sprtemp_t>());

        let intname: i32 = *(spritename as *const i32);
            
        maxframe = -1;
        
        // scan the lumps,
        //  filling in the frames for whatever is found
        for l in start + 1 .. end {
            let lumpinfo_tmp = lumpinfo;
            let lump = lumpinfo_tmp.offset(l as isize);

            if *((*lump).name.as_ptr() as *const i32) == intname {
                let frame = ((*lump).name[4] as u32) - ('A' as u32);
                let rotation = ((*lump).name[5] as u32) - ('0' as u32);
                let patched: i32;

                if modifiedgame != c_false {
                    patched = W_GetNumForName ((*lump).name.as_ptr());
                } else {
                    patched = l;
                }

                R_InstallSpriteLump (patched, frame, rotation, c_false);

                if (*lump).name[6] != 0 {
                    let frame = ((*lump).name[6] as u32) - ('A' as u32);
                    let rotation = ((*lump).name[7] as u32) - ('0' as u32);
                    R_InstallSpriteLump (l, frame, rotation, c_true);
                }
            } 
        }
        
        // check the frames that were found for completeness
        if maxframe == -1 {
            (*sprite).numframes = 0;
            continue;
        }
        
        maxframe += 1;
    
        for frame in 0 .. maxframe as u32 {
            match sprtemp[frame as usize].rotate {
                -1 => {
                    // no rotations were found for that frame at all
                    panic!("R_InitSprites: No patches found for {} frame {}",
                        std::ffi::CStr::from_ptr(spritename).to_str().unwrap(),
                        char::from_u32(('A' as u32) + frame).unwrap());
                },
                0 => {
                    // only the first rotation is needed
                },
                1 => {
                    // must have all 8 frames
                    for rotation in 0 .. 8 {
                        if sprtemp[frame as usize].lump[rotation] == -1 {
                            panic!("R_InitSprites: Sprite {} frame {} is missing rotations",
                                std::ffi::CStr::from_ptr(spritename).to_str().unwrap(),
                                char::from_u32(('A' as u32) + frame).unwrap());
                        }
                    }
                },
                _ => {
                    panic!("R_InitSprites: rotate value {} is not in expected range", sprtemp[frame as usize].rotate);
                },
            }
        }
    
        // allocate space for the frames present and copy sprtemp to it
        (*sprite).numframes = maxframe;
        (*sprite).spriteframes = 
            Z_Malloc ((maxframe as i32) * (std::mem::size_of::<spriteframe_t>() as i32),
                      PU_STATIC, std::ptr::null()) as *mut spriteframe_t;
        memcpy ((*sprite).spriteframes as *mut u8, sprtemp.as_ptr() as *const u8,
                        (maxframe as usize) * (std::mem::size_of::<spriteframe_t>() as usize));
    }
}

extern {
    static mut negonearray: [i16; SCREENWIDTH];
}

//
// R_InitSprites
// Called at program start.
//
#[no_mangle]
pub unsafe extern "C" fn R_InitSprites (namelist: *mut *mut i8) { 
    for i in 0 .. SCREENWIDTH {
        negonearray[i] = -1;
    }
    
    R_InitSpriteDefs (namelist);
}

extern {
    static mut vissprites: [vissprite_t; MAXVISSPRITES];
    static mut vissprite_p: *mut vissprite_t;
    static mut overflowsprite: vissprite_t;
}

//
// R_ClearSprites
// Called at frame start.
//
#[no_mangle]
pub unsafe extern "C" fn R_ClearSprites () {
    vissprite_p = vissprites.as_mut_ptr();
}


//
// R_NewVisSprite
//
#[no_mangle]
pub unsafe extern "C" fn R_NewVisSprite () -> *mut vissprite_t {
    if vissprite_p == vissprites.as_mut_ptr().offset(MAXVISSPRITES as isize) {
        return &mut overflowsprite;
    }
    
    vissprite_p = vissprite_p.offset(1);
    return vissprite_p.offset(-1);
}

extern {
    static dc_x: i32; 
    static mut dc_yl: i32; 
    static mut dc_yh: i32; 
    static mut dc_texturemid: fixed_t;

    static mut dc_source: *const u8;

    static mut mfloorclip: *mut i16;
    static mut mceilingclip: *mut i16;

    static mut spryscale: fixed_t;
    static mut sprtopscreen: fixed_t;
    static colfunc: extern "C" fn ();
}
//
// R_DrawMaskedColumn
// Used for sprites and masked mid textures.
// Masked means: partly transparent, i.e. stored
//  in posts/runs of opaque pixels.
//
#[no_mangle]
pub unsafe extern "C" fn R_DrawMaskedColumn (column: *mut column_t) {
    let basetexturemid = dc_texturemid;
    let mut column_tmp = column;

    while (*column_tmp).topdelta != 0xff {
        // calculate unclipped screen coordinates
        //  for post
        let topscreen = sprtopscreen.wrapping_add(spryscale.wrapping_mul((*column_tmp).topdelta as fixed_t));
        let bottomscreen = topscreen.wrapping_add(spryscale.wrapping_mul((*column_tmp).length as fixed_t));

        dc_yl = ((topscreen as i32) + FRACUNIT - 1) >> FRACBITS;
        dc_yh = ((bottomscreen as i32) - 1) >> FRACBITS;
            
        dc_yh = i32::min(dc_yh, (*mfloorclip.offset(dc_x as isize) as i32) - 1);
        dc_yl = i32::max(dc_yl, (*mceilingclip.offset(dc_x as isize) as i32) + 1);

        if dc_yl <= dc_yh {
            dc_source = (column_tmp as *mut u8).offset(3);
            dc_texturemid = basetexturemid.wrapping_sub(((*column_tmp).topdelta as fixed_t) << FRACBITS);

            // Drawn by either R_DrawColumn
            //  or (SHADOW) R_DrawFuzzColumn.
            colfunc ();
        }
        column_tmp = (column_tmp as *mut u8).offset(((*column_tmp).length as isize) + 4) as *mut column_t;
    }

    dc_texturemid = basetexturemid;
}

