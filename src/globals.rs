// Global variables used in Doom
use crate::defs::*;
extern {
    pub static mut walllights: *mut *mut lighttable_t;
    pub static mut maskedtexturecol: *mut i16;
    pub static mut screens: [*mut u8; 5];
    pub static mut numsprites: i32;
    pub static mut sprites: *mut spritedef_t;
    pub static mut lumpinfo: *mut lumpinfo_t;
    pub static mut modifiedgame: boolean;
    pub static mut screenheightarray: [i16; SCREENWIDTH as usize];
    pub static mut negonearray: [i16; SCREENWIDTH as usize];
    pub static mut pspritescale: fixed_t;
    pub static mut pspriteiscale: fixed_t;
    pub static mut ceilingclip: [i16; SCREENWIDTH as usize];
    pub static mut markceiling: boolean;
    pub static mut ceilingplane: *mut visplane_t;
    pub static mut floorclip: [i16; SCREENWIDTH as usize];
    pub static mut markfloor: boolean;
    pub static mut floorplane: *mut visplane_t;
    pub static mut segtextured: boolean;
    pub static mut rw_distance: fixed_t;
    pub static mut midtexture: i32;
    pub static mut toptexture: i32;
    pub static mut bottomtexture: i32;
    pub static mut rw_normalangle: angle_t;
    pub static mut rw_angle1: i32;
    pub static mut skyflatnum: i32;
    pub static mut lastopening: *mut i16;
    pub static mut segs: *mut seg_t;
    pub static mut numsubsectors: i32;
    pub static mut subsectors: *mut subsector_t;
    pub static mut nodes: *mut node_t;
    pub static mut finecosine: *mut fixed_t;
    pub static mut yslope: [fixed_t; SCREENHEIGHT as usize];
    pub static mut distscale: [fixed_t; SCREENWIDTH as usize];
    pub static mut detailLevel: i32;
    pub static mut screenblocks: i32;
    pub static mut numnodes: i32;
    pub static mut planeheight: fixed_t;
    pub static mut cachedheight: [fixed_t; SCREENHEIGHT as usize];
    pub static mut cacheddistance: [fixed_t; SCREENHEIGHT as usize];
    pub static mut cachedystep: [fixed_t; SCREENHEIGHT as usize];
    pub static mut cachedxstep: [fixed_t; SCREENHEIGHT as usize];
    pub static mut basexscale: fixed_t;
    pub static mut baseyscale: fixed_t;
    pub static mut planezlight: *mut *mut lighttable_t;
    pub static mut visplanes: [visplane_t; MAXVISPLANES as usize];
    pub static mut lastvisplane: *mut visplane_t;
    pub static mut openings: [i16; MAXOPENINGS as usize];
    pub static mut spanstart: [i32; SCREENHEIGHT as usize];
    pub static mut skytexturemid: i32;
    pub static mut skytexture: i32;
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
}

