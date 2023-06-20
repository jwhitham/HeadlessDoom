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
// Handles WAD file header, directory, lump I/O.
//
//-----------------------------------------------------------------------------

use crate::globals::*;
use crate::funcs::*;

use libc::toupper;

extern {
    pub static mut numlumps: i32;
    pub static mut lumpcache: *mut *mut u8;
}
//
// W_CheckNumForName
// Returns -1 if name not found.
//
#[no_mangle]
pub unsafe extern "C" fn W_CheckNumForName (name: *const u8) -> i32 {
    #[repr(C)]
    pub union name8 {
        pub s: [u8; 9],
        pub x: [i32; 2],
    }

    let mut name8 = name8 {
        x: [0, 0],
    };
    
    // make the name into two integers for easy compares
    // case insensitive
    for i in 0 .. 8 {
        let c = toupper(*name.offset(i) as i32) as u8;
        *name8.s.as_mut_ptr().offset(i) = c;
        if c == 0 {
            break;
        }
    }

    // in case the name was a fill 8 chars
    *name8.s.as_mut_ptr().offset(8) = 0;

    let v1 = name8.x[0];
    let v2 = name8.x[1];

    // scan backwards so patch lump files take precedence
    let mut index = numlumps as isize;
    while index != 0 {
        index -= 1;
        let lump_p = lumpinfo.offset(index);
        let name_p = (*lump_p).name.as_ptr() as *const i32;

        if *name_p.offset(0) == v1 && *name_p.offset(1) == v2 {
            return index as i32;
        }
    }

    // TFB. Not found.
    return -1;
}




//
// W_GetNumForName
// Calls W_CheckNumForName, but bombs out if not found.
//
#[no_mangle]
pub unsafe extern "C" fn W_GetNumForName (name: *const u8) -> i32 {

    let i = W_CheckNumForName (name);
    
    if i == -1 {
        panic!("W_GetNumForName: {} not found!", W_Name(name));
    }
      
    return i;
}


//
// W_LumpLength
// Returns the buffer size needed to load the given lump.
//
#[no_mangle]
pub unsafe extern "C" fn W_LumpLength (lump: i32) -> i32 {

    if (lump < 0) || (lump >= numlumps) {
        panic!("W_LumpLength: {} >= numlumps", lump);
    }

    return (*lumpinfo.offset(lump as isize)).size;
}

//
// W_CacheLumpNum
//
#[no_mangle]
pub unsafe extern "C" fn W_CacheLumpNum(lump: i32, tag: u32) -> *mut u8 {

    if (lump < 0) || (lump >= numlumps) {
        panic!("W_CacheLumpNum: {} >= numlumps", lump);
    }

    if *lumpcache.offset(lump as isize) == std::ptr::null_mut() {
        // read the lump in
        
        //printf ("cache miss on lump %i\n",lump);
        let len = W_LumpLength (lump);
        let ptr = Z_Malloc (len + 128, tag,
                            lumpcache.offset(lump as isize));
        W_ReadLump (lump, *lumpcache.offset(lump as isize));
        memset (ptr.offset(len as isize), 0, 128); // DSB-21
    } else {
        //printf ("cache hit on lump %i\n",lump);
        Z_ChangeTag2 (*lumpcache.offset(lump as isize), tag);
    }

    return *lumpcache.offset(lump as isize);
}



//
// W_CacheLumpName
//
#[no_mangle]
pub unsafe extern "C" fn W_CacheLumpName(name: *const u8, tag: u32) -> *mut u8 {
    return W_CacheLumpNum (W_GetNumForName(name), tag);
}

