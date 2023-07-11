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

use crate::funcs::*;

use libc::toupper;
use std::fs::File;
use std::io::Seek;
use std::io::Read;

struct wad_lumpinfo_t {
    cache: *mut u8,
    position: i32,
    size: i32,
    name: [u8; 8],
    handle: usize,
}

static mut lumpinfo: Vec<wad_lumpinfo_t> = Vec::new();
static mut file_handles: Vec<File> = Vec::new();

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
    let mut index = lumpinfo.len() as usize;
    while index != 0 {
        index -= 1;
        let lump_p = lumpinfo.get(index).unwrap();
        let name_p = lump_p.name.as_ptr() as *const i32;

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

pub unsafe fn W_GetNameForNum (lump: i32) -> *const u8 {
    if (lump < 0) || ((lump as usize) >= lumpinfo.len()) {
        panic!("W_GetNameForNum: {} >= numlumps", lump);
    }

    return lumpinfo.get(lump as usize).unwrap().name.as_ptr();
}

//
// W_LumpLength
// Returns the buffer size needed to load the given lump.
//
#[no_mangle]
pub unsafe extern "C" fn W_LumpLength (lump: i32) -> i32 {

    if (lump < 0) || ((lump as usize) >= lumpinfo.len()) {
        panic!("W_LumpLength: {} >= numlumps", lump);
    }

    return lumpinfo.get(lump as usize).unwrap().size;
}

//
// W_CacheLumpNum
//
#[no_mangle]
pub unsafe extern "C" fn W_CacheLumpNum(lump: i32, tag: u32) -> *mut u8 {

    if (lump < 0) || ((lump as usize) >= lumpinfo.len()) {
        panic!("W_CacheLumpNum: {} >= numlumps", lump);
    }

    let lump_p = lumpinfo.get(lump as usize).unwrap();
    if lump_p.cache == std::ptr::null_mut() {
        // read the lump in
        
        //printf ("cache miss on lump %i\n",lump);
        let len = lump_p.size;
        let ptr = Z_Malloc (len + 128, tag, std::ptr::null_mut()) as *mut u8;
        lumpinfo.get_mut(lump as usize).unwrap().cache = ptr;
        W_ReadLump (lump, ptr);
        memset (ptr.offset(len as isize), 0, 128); // DSB-21
        return ptr;
    } else {
        //printf ("cache hit on lump %i\n",lump);
        let ptr = lump_p.cache as *mut u8;
        Z_ChangeTag2 (ptr, tag);
        return ptr;
    }
}



//
// W_CacheLumpName
//
#[no_mangle]
pub unsafe extern "C" fn W_CacheLumpName(name: *const u8, tag: u32) -> *mut u8 {
    return W_CacheLumpNum (W_GetNumForName(name), tag);
}

//
// W_ReadLump
// Loads the lump into the given buffer,
//  which must be >= W_LumpLength().
//
#[no_mangle]
pub unsafe extern "C" fn W_ReadLump(lump: i32, dest: *mut u8) {
    if (lump < 0) || ((lump as usize) >= lumpinfo.len()) {
        panic!("W_ReadLump: {} >= numlumps", lump);
    }

    let l = lumpinfo.get(lump as usize).unwrap();
    
    // ??? I_BeginRead ();
    //
    let handle = file_handles.get_mut(l.handle).unwrap();

    handle.seek(std::io::SeekFrom::Start(l.position as u64));
    let slice = std::slice::from_raw_parts_mut(dest, l.size as usize);
    let c = handle.read(slice).unwrap_or(0);
    
    if c != (l.size as usize) {
        panic!("W_ReadLump: only read {} of {} on lump {}",
                c, l.size, lump);
    }
}

unsafe fn ExtractFileBase(path: &str, dest: &mut [u8; 8]) {

    let path_bytes = path.as_bytes();
    let mut start_index = path_bytes.len();

    while (start_index > 0)
    && (path_bytes[start_index - 1] != ('\\' as u8))
    && (path_bytes[start_index - 1] != ('/' as u8)) {
        start_index -= 1;
    }

    *dest = [0; 8];
    for i in 0 .. 8 {
        if (i + start_index) >= path_bytes.len() {
            break;
        }
        if path_bytes[i + start_index] == ('.' as u8) {
            break;
        }
        dest[i] = toupper(path_bytes[i + start_index] as i32) as u8;
    }
}


//
// W_AddFile
// All files are optional, but at least one file must be
//  found (PWAD, if all required lumps are present).
// Files with a .wad extension are wadlink files
//  with multiple lumps.
// Other files are single lumps with the base filename
//  for the lump name.
//
// If filename starts with a tilde, the file is handled
//  specially to allow map reloads.
// But: the reload feature is a fragile hack...

unsafe fn W_AddFile (filename: &str) {
    // open the file and add to directory

    // handle reload indicator.
    if filename.starts_with("~") {
        panic!("no support for reloadable files");
    }
    
    let handle_or_err = File::open(filename);
    if !handle_or_err.is_ok() {
        println!(" couldn't open {}", filename);
        return;
    }
    let mut handle = handle_or_err.unwrap();

    #[derive(Copy,Clone)]
    struct filelump_t {
        filepos: i32,
        size: i32,
        name: [u8; 8],
    }
    println!(" adding {}",filename);
    let mut fileinfo: Vec<filelump_t> = Vec::new();
    
    if !filename.to_lowercase().ends_with("wad") {
        // single lump file
        let mut singleinfo: filelump_t = filelump_t {
            filepos: 0,
            size: i32::to_le(handle.metadata().unwrap().len() as i32),
            name: [0; 8],
        };
        ExtractFileBase (filename, &mut singleinfo.name);
        fileinfo.push(singleinfo);
    } else {
        // WAD file
        #[derive(Copy,Clone)]
        struct wadinfo_t {
            identification: [u8; 4],
            numlumps: i32,
            infotableofs: i32,
        }
        #[derive(Copy,Clone)]
        union read_wadinfo_t {
            w: wadinfo_t,
            d: [u8; 12],
        }
        let mut read_wadinfo: read_wadinfo_t = read_wadinfo_t {
            d: [0; 12],
        };
        handle.read_exact(&mut read_wadinfo.d);
        let mut header = read_wadinfo.w;
        if header.identification != "IWAD".as_bytes() {
            // Homebrew levels?
            if header.identification != "PWAD".as_bytes() {
                panic!("Wad file {} doesn't have IWAD or PWAD id", filename);
            }

            // ???modifiedgame = true;
        }
        header.numlumps = i32::from_le(header.numlumps);
        header.infotableofs = i32::from_le(header.infotableofs);
        handle.seek(std::io::SeekFrom::Start(header.infotableofs as u64));

        #[derive(Copy,Clone)]
        union read_filelump_t {
            w: filelump_t,
            d: [u8; 16],
        }
        let mut read_filelump: read_filelump_t = read_filelump_t {
            d: [0; 16],
        };
        for _ in 0 .. header.numlumps {
            handle.read(&mut read_filelump.d);
            fileinfo.push(read_filelump.w);
        }
    }

    // Fill in lumpinfo
    for info in fileinfo {
        let lump: wad_lumpinfo_t = wad_lumpinfo_t {
            position: i32::from_le(info.filepos),
            size: i32::from_le(info.size),
            name: info.name as [u8; 8],
            handle: file_handles.len(),
            cache: std::ptr::null_mut(),
        };
        lumpinfo.push(lump);
    }
    file_handles.push(handle);
}


//
// W_Reload
// Flushes any of the reloadable lumps in memory
//  and reloads the directory.
//
#[no_mangle]
pub unsafe extern "C" fn W_Reload () {
    panic!("W_Reload() not implemented");
}



//
// W_InitMultipleFiles
// Pass a null terminated list of files to use.
// All files are optional, but at least one file
//  must be found.
// Files with a .wad extension are idlink files
//  with multiple lumps.
// Other files are single lumps with the base filename
//  for the lump name.
// Lump names can appear multiple times.
// The name searcher looks backwards, so a later file
//  does override all earlier ones.
//
#[no_mangle]
pub unsafe extern "C" fn W_InitMultipleFiles (filenames: *const *const u8) { 
    // open all the files, load headers, and count lumps
    let mut i: isize = 0;

    while *filenames.offset(i) != std::ptr::null() {
        W_AddFile(&W_Str_C2R(*filenames.offset(i)));
        i += 1;
    }

    if lumpinfo.is_empty() {
        panic!("W_InitFiles: no files found");
    }
}


