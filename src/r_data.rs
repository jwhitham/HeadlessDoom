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
#[no_mangle]
pub unsafe extern "C" fn R_DrawColumnInCache(
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


