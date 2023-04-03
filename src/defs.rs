

pub type boolean = i32;
pub const c_false: boolean = 0;
pub const c_true: boolean = 1;

pub const FRACBITS: i32 = 16;
pub const FRACUNIT: i32 = 1 << FRACBITS;
pub const SCREENWIDTH: usize = 320;
pub const SCREENHEIGHT: usize = 200;

pub type fixed_t = u32;
pub const MAXVISSPRITES: usize = 128;

pub const PU_CACHE: i32 = 101;

pub const MF_TRANSLATION: i32 = 0xc000000;

pub const MF_TRANSSHIFT: i32 = 26;


// 
// Sprites are patches with a special naming convention
//  so they can be recognized by R_InitSprites.
// The base name is NNNNFx or NNNNFxFx, with
//  x indicating the rotation, x = 0, 1-7.
// The sprite and frame specified by a thing_t
//  is range checked at run time.
// A sprite is a patch_t that is assumed to represent
//  a three dimensional object and may have multiple
//  rotations pre drawn.
// Horizontal flipping is used to save space,
//  thus NNNNF2F5 defines a mirrored patch.
// Some sprites will only have one picture used
// for all views: NNNNF0
//
#[repr(C)]
pub struct spriteframe_t {
    // If false use 0 for any position.
    // Note: as eight entries are available,
    //  we might as well insert the same name eight times.
    pub rotate: i32,

    // Lump to use for view angles 0-7.
    pub lump: [i16; 8],

    // Flip bit (1 = flip) to use for view angles 0-7.
    pub flip: [u8; 8],
    
}

//
// A sprite definition:
//  a number of animation frames.
//
#[repr(C)]
pub struct spritedef_t {
    pub numframes: i32,
    pub spriteframes: *mut spriteframe_t,
}

pub type sprtemp_t = [spriteframe_t; 29];

//
// WADFILE I/O related stuff.
//
#[repr(C)]
pub struct lumpinfo_t {
    pub name: [i8; 8],
    pub handle: *const u8,
    pub position: i32,
    pub size: i32,
}

pub type lighttable_t = u8; 

// Patches.
// A patch holds one or more columns.
// Patches are used for sprites and all masked pictures,
// and we compose textures from the TEXTURE1/2 lists
// of patches.
#[repr(C)]
pub struct patch_t {
    pub width: i16,  // bounding box size 
    pub height: i16, 
    pub leftoffset: i16, // pixels to the left of origin 
    pub topoffset: i16, // pixels below the origin 
    pub columnofs: [i32; 8], // only [width] used
    // the [0] is &columnofs[width] 
}

// A vissprite_t is a thing
//  that will be drawn during a refresh.
// I.e. a sprite object that is partly visible.
#[repr(C)]
pub struct vissprite_t {
    // Doubly linked list.
    pub prev: *mut vissprite_t,
    pub next: *mut vissprite_t,
    
    pub x1: i32,
    pub x2: i32,

    // for line side calculation
    pub gx: fixed_t,
    pub gy: fixed_t,

    // global bottom / top for silhouette clipping
    pub gz: fixed_t,
    pub gzt: fixed_t,

    // horizontal position of x1
    pub startfrac: fixed_t,
    
    pub scale: fixed_t,
    
    // negative if flipped
    pub xiscale: fixed_t,

    pub texturemid: fixed_t,
    pub patch: i32,

    // for color translation and shadow draw,
    //  maxbright frames as well
    pub colormap: *mut lighttable_t,
   
    pub mobjflags: i32,
}

// posts are runs of non masked source pixels
#[repr(C)]
pub struct post_t {
    pub topdelta: u8, // -1 is the last post in a column
    pub length: u8,  // length data bytes follows
}

// column_t is a list of 0 or more post_t, (byte)-1 terminated
pub type column_t = post_t;
