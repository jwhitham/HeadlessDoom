// Global variables used in Doom
use crate::defs::*;
extern {
    pub static mut curline: *mut seg_t;
    pub static mut frontsector: *mut sector_t;
    pub static mut backsector: *mut sector_t;
    pub static mut texturetranslation: *mut i32;
    pub static mut extralight: i32;
    pub static mut walllights: *mut *mut lighttable_t;
    pub static mut scalelight: [[*mut lighttable_t; MAXLIGHTSCALE as usize]; LIGHTLEVELS as usize];
    pub static mut maskedtexturecol: *mut i16;
    pub static mut dc_texturemid: fixed_t;
    pub static mut dc_x: i32; 
    pub static mut textureheight: *mut fixed_t;
    pub static mut fixedcolormap: *mut lighttable_t;
    pub static mut dc_colormap: *const u8;
    pub static mut dc_iscale: fixed_t; 
    pub static mut ylookup: [*mut u8; SCREENWIDTH as usize];
    pub static mut columnofs: [i32; SCREENWIDTH as usize];
    pub static mut centery: i32; 
    pub static mut dc_yl: i32; 
    pub static mut dc_yh: i32; 
    pub static mut dc_source: *mut u8;
    pub static mut colormaps: *mut u8;
    pub static mut viewheight: i32;
    pub static mut dc_translation: *const u8;
    pub static mut translationtables: *mut u8;
    pub static mut ds_y: i32; 
    pub static mut ds_x1: i32; 
    pub static mut ds_x2: i32;
    pub static mut ds_colormap: *const u8; 
    pub static mut ds_xfrac: fixed_t; 
    pub static mut ds_yfrac: fixed_t; 
    pub static mut ds_xstep: fixed_t; 
    pub static mut ds_ystep: fixed_t;
    pub static mut ds_source: *const u8;
    pub static mut viewwindowx: i32;
    pub static mut viewwindowy: i32;
    pub static mut screens: [*mut u8; 5];
    pub static mut firstspritelump: i32;
    pub static mut lastspritelump: i32;
    pub static mut numsprites: i32;
    pub static mut sprites: *mut spritedef_t;
    pub static mut lumpinfo: *mut lumpinfo_t;
    pub static mut modifiedgame: boolean;
    pub static mut screenheightarray: [i16; SCREENWIDTH as usize];
    pub static mut negonearray: [i16; SCREENWIDTH as usize];
    pub static mut colfunc: unsafe extern "C" fn ();
    pub static mut fuzzcolfunc: unsafe extern "C" fn ();
    pub static mut basecolfunc: unsafe extern "C" fn ();
    pub static mut detailshift: i32; 
    pub static mut centerxfrac: fixed_t; 
    pub static mut centeryfrac: fixed_t; 
    pub static mut viewx: fixed_t;
    pub static mut viewy: fixed_t;
    pub static mut viewz: fixed_t;
    pub static mut viewcos: fixed_t;
    pub static mut viewsin: fixed_t;
    pub static mut viewwidth: i32;
    pub static mut projection: fixed_t;
    pub static mut spriteoffset: *mut fixed_t;
    pub static mut spritetopoffset: *mut fixed_t;
    pub static mut spritewidth: *mut fixed_t;
    pub static mut validcount: i32;
    pub static mut pspritescale: fixed_t;
    pub static mut pspriteiscale: fixed_t;
    pub static mut viewplayer: *mut player_t;
    pub static mut ds_p: *mut drawseg_t;
    pub static mut drawsegs: [drawseg_t; MAXDRAWSEGS as usize];
    pub static mut viewangleoffset: i32;
    pub static mut ceilingclip: [i16; SCREENWIDTH as usize];
    pub static mut markceiling: boolean;
    pub static mut ceilingplane: *mut visplane_t;
    pub static mut floorclip: [i16; SCREENWIDTH as usize];
    pub static mut markfloor: boolean;
    pub static mut floorplane: *mut visplane_t;
    pub static mut segtextured: boolean;
    pub static mut xtoviewangle: [angle_t; (SCREENWIDTH + 1) as usize];
    pub static mut rw_distance: fixed_t;
    pub static mut midtexture: i32;
    pub static mut toptexture: i32;
    pub static mut bottomtexture: i32;
    pub static mut sidedef: *mut side_t;
    pub static mut linedef: *mut line_t;
    pub static mut rw_normalangle: angle_t;
    pub static mut rw_angle1: i32;
    pub static mut viewangle: angle_t;
    pub static mut skyflatnum: i32;
    pub static mut lastopening: *mut i16;
    pub static mut newend: *mut cliprange_t;
    pub static mut solidsegs: [cliprange_t; MAXSEGS as usize];
    pub static mut clipangle: angle_t;
    pub static mut viewangletox: [i32; (FINEANGLES / 2) as usize];
    pub static mut segs: *mut seg_t;
    pub static mut numsubsectors: i32;
    pub static mut subsectors: *mut subsector_t;
    pub static mut sscount: i32;
    pub static mut nodes: *mut node_t;
    pub static mut centerx: i32;
    pub static mut zlight: [[*mut lighttable_t; MAXLIGHTZ as usize]; LIGHTLEVELS as usize];
    pub static mut setsizeneeded: boolean;
    pub static mut setblocks: i32;
    pub static mut setdetail: i32;
    pub static mut finecosine: *mut fixed_t;
    pub static mut scaledviewwidth: i32;
    pub static mut transcolfunc: unsafe extern "C" fn ();
    pub static mut spanfunc: unsafe extern "C" fn ();
    pub static mut yslope: [fixed_t; SCREENHEIGHT as usize];
    pub static mut distscale: [fixed_t; SCREENWIDTH as usize];
    pub static mut detailLevel: i32;
    pub static mut screenblocks: i32;
    pub static mut framecount: i32;
    pub static mut numnodes: i32;
    pub static mut scalelightfixed: [*mut lighttable_t; MAXLIGHTSCALE as usize];
    pub static mut planeheight: fixed_t;
    pub static mut cachedheight: [fixed_t; SCREENHEIGHT as usize];
    pub static mut cacheddistance: [fixed_t; SCREENHEIGHT as usize];
    pub static mut cachedystep: [fixed_t; SCREENHEIGHT as usize];
    pub static mut cachedxstep: [fixed_t; SCREENHEIGHT as usize];
    pub static mut basexscale: fixed_t;
    pub static mut baseyscale: fixed_t;
    pub static mut planezlight: *mut *mut lighttable_t;
}
