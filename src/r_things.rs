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



type boolean = i32;
const c_false: boolean = 0;
const c_true: boolean = 1;

#[repr(C)]
struct spriteframe_t {
    // If false use 0 for any position.
    // Note: as eight entries are available,
    //  we might as well insert the same name eight times.
    rotate: boolean,

    // Lump to use for view angles 0-7.
    lump: [i16; 8],

    // Flip bit (1 = flip) to use for view angles 0-7.
    flip: [u8; 8],
    
}

extern {
    static mut maxframe: i32;
    static mut sprtemp: [spriteframe_t; 29];
    static mut spritename: *mut i8;
    static mut firstspritelump: i32;
}
//
// R_InstallSpriteLump
// Local function for R_InitSprites.
//
#[no_mangle]
pub extern "C" fn R_InstallSpriteLump(
        lump: i32, frame: u32, rotation: u32, flipped: boolean) {
    
    unsafe {
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
                    ('A' as u32) + frame);
            }

            if sprtemp[frame as usize].rotate == c_true {
                panic!("R_InitSprites: Sprite {} frame {} has rotations and a rot=0 lump",
                    std::ffi::CStr::from_ptr(spritename).to_str().unwrap(),
                    ('A' as u32) + frame);
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
                    ('A' as u32) + frame);
        }
                
        sprtemp[frame as usize].rotate = c_true;

        // make 0 based
        rotation_tmp -= 1;
        if sprtemp[frame as usize].lump[rotation_tmp] != -1 {
            panic!("R_InitSprites: Sprite {} : {} : {} has two lumps mapped to it",
                    std::ffi::CStr::from_ptr(spritename).to_str().unwrap(),
                    ('A' as u32) + frame,
                    ('1' as u32) + (rotation_tmp as u32));
        }
            
        sprtemp[frame as usize].lump[rotation_tmp] = (lump - firstspritelump) as i16;
        sprtemp[frame as usize].flip[rotation_tmp] = flipped as u8;
    }
}

