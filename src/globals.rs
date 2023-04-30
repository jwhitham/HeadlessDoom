// Global variables used in Doom
use crate::defs::*;
extern {
    pub static mut screens: [*mut u8; 5];
    pub static mut numsprites: i32;
    pub static mut sprites: *mut spritedef_t;
    pub static mut lumpinfo: *mut lumpinfo_t;
    pub static mut modifiedgame: boolean;
    pub static mut screenheightarray: [i16; SCREENWIDTH as usize];
    pub static mut negonearray: [i16; SCREENWIDTH as usize];
    pub static mut pspritescale: fixed_t;
    pub static mut pspriteiscale: fixed_t;
    pub static mut segs: *mut seg_t;
    pub static mut numsubsectors: i32;
    pub static mut subsectors: *mut subsector_t;
    pub static mut nodes: *mut node_t;
    pub static mut finecosine: *mut fixed_t;
    pub static mut detailLevel: i32;
    pub static mut screenblocks: i32;
    pub static mut numnodes: i32;
    pub static mut demoplayback: boolean;

    // These are used from C code
    pub static mut firstspritelump: i32;
    pub static mut flattranslation: *mut i32;
    // used from p_spec
    pub static mut texturetranslation: *mut i32;
    // used from p_floor
    pub static mut textureheight: *mut fixed_t;
    // used from hu_lib and d_main
    pub static mut viewwindowx: i32;
    pub static mut viewwindowy: i32;
    pub static mut viewheight: i32;
    pub static mut viewwidth: i32;
    pub static mut scaledviewwidth: i32;
    // used from p_map and others
    pub static mut validcount: i32;
    // used from d_main
    pub static mut setsizeneeded: boolean;
    // use from g_game, p_map, p_mobj
    pub static mut skyflatnum: i32;
    // used from g_game
    pub static mut skytexture: i32;
}

