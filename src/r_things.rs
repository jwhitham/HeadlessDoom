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
// Refresh of things, i.e. objects represented by sprites.
//
//-----------------------------------------------------------------------------


use crate::defs::*;
use crate::r_draw::R_DrawTranslatedColumn;
use crate::defs::mobjflag_t::*;
use crate::defs::powertype_t::*;
use crate::defs::psprnum_t::*;
use crate::globals::*;
use crate::funcs::*;
use crate::r_main::R_PointToAngle;
use crate::r_main::R_PointOnSegSide;
use crate::r_segs::R_RenderMaskedSegRange;
use crate::m_fixed::FixedMul;
use crate::m_fixed::FixedDiv;
use crate::r_bsp::ds_p;
use crate::r_bsp::drawsegs;

pub struct R_DrawMaskedColumn_params_t {
    pub column: *mut column_t,
    pub sprtopscreen: fixed_t,
    pub spryscale: fixed_t,
    pub mfloorclip: *mut i16,
    pub mceilingclip: *mut i16,
}

struct R_DrawVisSprite_params_t {
    vis: *mut vissprite_t,
    mfloorclip: *mut i16,
    mceilingclip: *mut i16,
}

struct R_DrawPSprite_params_t {
    psp: *mut pspdef_t,
    mfloorclip: *mut i16,
    mceilingclip: *mut i16,
}

const SPRTEMP_SIZE: usize = 29;
type sprtemp_t = [spriteframe_t; SPRTEMP_SIZE];

const BLANK_VISSPRITE: vissprite_t = vissprite_t {
    prev: std::ptr::null_mut(),
    next: std::ptr::null_mut(),
    gx: 0,
    gy: 0,
    gz: 0,
    gzt: 0,
    patch: 0,
    colormap: std::ptr::null_mut(),
    mobjflags: 0,
    texturemid: 0,
    x1: 0,
    x2: 0,
    scale: 0,
    xiscale: 0,
    startfrac: 0,
};
const BLANK_SPRITEFRAME: spriteframe_t = spriteframe_t {
    rotate: c_false,
    lump: [0; 8],
    flip: [0; 8],
};
static mut spritelights: *mut *mut lighttable_t = std::ptr::null_mut();
static mut sprtemp: sprtemp_t = [BLANK_SPRITEFRAME; SPRTEMP_SIZE];
static mut maxframe: i32 = 0;
static mut spritename: *mut u8 = std::ptr::null_mut();
static mut vissprites: [vissprite_t; MAXVISSPRITES as usize] = [BLANK_VISSPRITE; MAXVISSPRITES as usize];
static mut vissprite_p: *mut vissprite_t = std::ptr::null_mut();
static mut overflowsprite: vissprite_t = BLANK_VISSPRITE;

const MINZ: fixed_t = (FRACUNIT*4) as fixed_t;
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
                W_Name(spritename),
                char::from_u32(('A' as u32) + frame).unwrap());
        }

        if sprtemp[frame as usize].rotate == c_true {
            panic!("R_InitSprites: Sprite {} frame {} has rotations and a rot=0 lump",
                W_Name(spritename),
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
                W_Name(spritename),
                char::from_u32(('A' as u32) + frame).unwrap());
    }
            
    sprtemp[frame as usize].rotate = c_true;

    // make 0 based
    rotation_tmp -= 1;
    if sprtemp[frame as usize].lump[rotation_tmp] != -1 {
        panic!("R_InitSprites: Sprite {} : {} : {} has two lumps mapped to it",
                W_Name(spritename),
                char::from_u32(('A' as u32) + frame).unwrap(),
                char::from_u32(('1' as u32) + (rotation_tmp as u32)).unwrap());
    }
        
    sprtemp[frame as usize].lump[rotation_tmp] = (lump - firstspritelump) as i16;
    sprtemp[frame as usize].flip[rotation_tmp] = flipped as u8;
}

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
unsafe fn R_InitSpriteDefs (namelist: *mut *mut u8) { 
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
                PU_STATIC, std::ptr::null_mut()) as *mut spritedef_t;
    
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
                    patched = W_GetNumForName ((*lump).name.as_ptr() as *const u8);
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
                        W_Name(spritename),
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
                                W_Name(spritename),
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
                      PU_STATIC, std::ptr::null_mut()) as *mut spriteframe_t;
        memcpy ((*sprite).spriteframes as *mut u8, sprtemp.as_ptr() as *const u8,
                        (maxframe as usize) * (std::mem::size_of::<spriteframe_t>() as usize));
    }
}


//
// R_InitSprites
// Called at program start.
//
#[no_mangle]    // called from P_Init
pub unsafe extern "C" fn R_InitSprites (namelist: *mut *mut u8) { 
    for i in 0 .. SCREENWIDTH as usize {
        negonearray[i] = -1;
    }
    
    R_InitSpriteDefs (namelist);
}

//
// R_ClearSprites
// Called at frame start.
//
pub unsafe fn R_ClearSprites () {
    vissprite_p = vissprites.as_mut_ptr();
}


//
// R_NewVisSprite
//
unsafe fn R_NewVisSprite () -> *mut vissprite_t {
    if vissprite_p == vissprites.as_mut_ptr().offset(MAXVISSPRITES as isize) {
        return &mut overflowsprite;
    }
    
    vissprite_p = vissprite_p.offset(1);
    return vissprite_p.offset(-1);
}

//
// R_DrawMaskedColumn
// Used for sprites and masked mid textures.
// Masked means: partly transparent, i.e. stored
//  in posts/runs of opaque pixels.
//
pub unsafe fn R_DrawMaskedColumn (dmc: &mut R_DrawMaskedColumn_params_t) {
    let basetexturemid = dc_texturemid;

    while (*dmc.column).topdelta != 0xff {
        // calculate unclipped screen coordinates
        //  for post
        let topscreen = dmc.sprtopscreen.wrapping_add(dmc.spryscale.wrapping_mul((*dmc.column).topdelta as fixed_t));
        let bottomscreen = topscreen.wrapping_add(dmc.spryscale.wrapping_mul((*dmc.column).length as fixed_t));

        dc_yl = ((topscreen as i32) + (FRACUNIT as i32) - 1) >> FRACBITS;
        dc_yh = ((bottomscreen as i32) - 1) >> FRACBITS;
            
        dc_yh = i32::min(dc_yh, (*dmc.mfloorclip.offset(dc_x as isize) as i32) - 1);
        dc_yl = i32::max(dc_yl, (*dmc.mceilingclip.offset(dc_x as isize) as i32) + 1);

        if dc_yl <= dc_yh {
            dc_source = (dmc.column as *mut u8).offset(3);
            dc_texturemid = basetexturemid.wrapping_sub(((*dmc.column).topdelta as fixed_t) << FRACBITS);

            // Drawn by either R_DrawColumn
            //  or (SHADOW) R_DrawFuzzColumn.
            colfunc ();
        }
        dmc.column = (dmc.column as *mut u8).offset(((*dmc.column).length as isize) + 4) as *mut column_t;
    }

    dc_texturemid = basetexturemid;
}

//
// R_DrawVisSprite
//  mfloorclip and mceilingclip should also be set.
//
unsafe fn R_DrawVisSprite (dvs: &mut R_DrawVisSprite_params_t) {
    let vis = dvs.vis;
    let patch: *mut patch_t = W_CacheLumpNum ((*vis).patch + firstspritelump, PU_CACHE) as *mut patch_t;

    dc_colormap = (*vis).colormap;
    
    if dc_colormap == std::ptr::null() {
        // NULL colormap = shadow draw
        colfunc = fuzzcolfunc;
    } else if ((*vis).mobjflags & MF_TRANSLATION) != 0 {
        colfunc = R_DrawTranslatedColumn;
        dc_translation = translationtables.offset(
                - 256 +
            ( ((*vis).mobjflags & MF_TRANSLATION) >> (MF_TRANSSHIFT-8) ) as isize);
    }
    
    dc_iscale = i32::abs((*vis).xiscale as i32) >> detailshift;
    dc_texturemid = (*vis).texturemid;
    let mut frac = (*vis).startfrac;

    let mut dmc = R_DrawMaskedColumn_params_t {
        column: std::ptr::null_mut(),
        spryscale: (*vis).scale,
        sprtopscreen: centeryfrac.wrapping_sub(FixedMul(dc_texturemid, (*vis).scale)),
        mfloorclip: dvs.mfloorclip,
        mceilingclip: dvs.mceilingclip,
    };
 
    for x in (*vis).x1 ..= (*vis).x2 {
        dc_x = x;
        let texturecolumn = frac>>FRACBITS;
        dmc.column = (patch as *mut u8).offset(
                       i32::from_le(
                           *(*patch).columnofs.as_ptr().offset(texturecolumn as isize))
                       as isize) as *mut column_t;
        R_DrawMaskedColumn (&mut dmc);
        frac = frac.wrapping_add((*vis).xiscale);
    }

    colfunc = basecolfunc;
}

//
// R_ProjectSprite
// Generates a vissprite for a thing
//  if it might be visible.
//
unsafe fn R_ProjectSprite (thing: *mut mobj_t) {
    // transform the origin point
    let tr_x = (* thing).x - viewx;
    let tr_y = (* thing).y - viewy;
 
    let mut gxt = FixedMul(tr_x,viewcos); 
    let mut gyt = -FixedMul(tr_y,viewsin);
    
    let tz = gxt-gyt; 

    // thing is behind view plane?
    if tz < MINZ {
        return;
    }
    
    let xscale = FixedDiv(projection, tz);
 
    gxt = -FixedMul(tr_x,viewsin); 
    gyt = FixedMul(tr_y,viewcos); 
    let mut tx = -(gyt+gxt); 

    // too far off the side?
    if i32::abs(tx)>(tz<<2) {
        return;
    }
    
    // decide which patch to use for sprite relative to player
    if ((*thing).sprite as u32) >= (numsprites as u32) {
        panic!("R_ProjectSprite: invalid sprite number {}", (*thing).sprite);
    }
    let sprdef = sprites.offset((*thing).sprite as isize);
    let masked_frame = ((*thing).frame as isize) & (FF_FRAMEMASK as isize);
    if masked_frame >= ((*sprdef).numframes as isize) {
        panic!("R_ProjectSprite: invalid sprite frame {} : {}",
            (*thing).sprite, (*thing).frame);
    }
    let sprframe = (*sprdef).spriteframes.offset(masked_frame);

    let lump: i16;
    let flip: boolean;
    if (*sprframe).rotate != 0 {
         // choose a different rotation based on player view
         let ang = R_PointToAngle ((*thing).x, (*thing).y);
         let rot = ((ang.wrapping_sub((*thing).angle)).wrapping_add((ANG45/2)*9))>>29;
         lump = (*sprframe).lump[rot as usize];
         flip = (*sprframe).flip[rot as usize] as boolean;
    } else {
        // use single rotation for all views
        lump = (*sprframe).lump[0];
        flip = (*sprframe).flip[0] as boolean;
    }
    
    // calculate edges of the shape
    tx -= *spriteoffset.offset(lump as isize); 
    let x1 = (centerxfrac + FixedMul (tx,xscale) ) >>FRACBITS;

    // off the right side?
    if x1 > viewwidth {
        return;
    }
    
    tx +=  *spritewidth.offset(lump as isize);
    let x2 = ((centerxfrac + FixedMul (tx,xscale) ) >>FRACBITS) - 1;

    // off the left side
    if x2 < 0 {
        return;
    }
    
    // store information in a vissprite
    let vis = R_NewVisSprite ();
    (*vis).mobjflags = (*thing).flags;
    (*vis).scale = xscale<<detailshift;
    (*vis).gx = (*thing).x;
    (*vis).gy = (*thing).y;
    (*vis).gz = (*thing).z;
    (*vis).gzt = (*thing).z + *spritetopoffset.offset(lump as isize);
    (*vis).texturemid = (*vis).gzt - viewz;
    (*vis).x1 = i32::max(0, x1);
    (*vis).x2 = i32::min(viewwidth - 1, x2);
    let iscale = FixedDiv (FRACUNIT as fixed_t, xscale);

    if flip != c_false {
        (*vis).startfrac = *spritewidth.offset(lump as isize)-1;
        (*vis).xiscale = -iscale;
    } else {
        (*vis).startfrac = 0;
        (*vis).xiscale = iscale;
    }

    if (*vis).x1 > x1 {
        (*vis).startfrac += (*vis).xiscale*((*vis).x1-x1);
    }
    (*vis).patch = lump as i32;

    // get light level
    if ((*thing).flags & MF_SHADOW) != 0 {
        // shadow draw
        (*vis).colormap = std::ptr::null_mut();
    } else if fixedcolormap != std::ptr::null_mut() {
        // fixed map
        (*vis).colormap = fixedcolormap;
    } else if ((*thing).frame & (FF_FULLBRIGHT as i32)) != 0 {
        // full bright
        (*vis).colormap = colormaps;
    } else {
        // diminished light
        let index = i32::min((MAXLIGHTSCALE - 1) as i32,
                             xscale>>(LIGHTSCALESHIFT-(detailshift as u32)));

        (*vis).colormap = *spritelights.offset(index as isize);
    }
} 

//
// R_AddSprites
// During BSP traversal, this adds sprites by sector.
//
pub unsafe fn R_AddSprites (sec: *mut sector_t) {

    // BSP is traversed by subsector.
    // A sector might have been split into several
    //  subsectors during BSP building.
    // Thus we check whether its already added.
    if (*sec).validcount == validcount {
        return;
    }

    // Well, now it will be done.
    (*sec).validcount = validcount;
    
    let lightnum = i32::min((LIGHTLEVELS - 1) as i32,
            i32::max((((*sec).lightlevel >> LIGHTSEGSHIFT) as i32) + extralight, 0));
    spritelights = scalelight[lightnum as usize].as_mut_ptr();

    // Handle all things in sector.
    let mut thing = (*sec).thinglist;
    while thing != std::ptr::null_mut() {
        R_ProjectSprite (thing);
        thing = (*thing).snext;
    }
}

const BASEYCENTER: i32 = 100;
//
// R_DrawPSprite
//
// e.g. current weapon
unsafe fn R_DrawPSprite (dps: &mut R_DrawPSprite_params_t) {
    // decide which patch to use
    if ((*(*dps.psp).state).sprite as u32) >= (numsprites as u32) {
        panic!("R_DrawPSprite: invalid sprite number {}",
             (*(*dps.psp).state).sprite);
    }
    let sprdef = sprites.offset((*(*dps.psp).state).sprite as isize);
    let maskframe = (((*(*dps.psp).state).frame as u32) & FF_FRAMEMASK) as u32;
    if maskframe >= ((*sprdef).numframes as u32) {
        panic!("R_DrawPSprite: invalid sprite frame {} : {} ",
             (*(*dps.psp).state).sprite, (*(*dps.psp).state).frame);
    }
    let sprframe = (*sprdef).spriteframes.offset(maskframe as isize);

    let lump = (*sprframe).lump[0];
    let flip = (*sprframe).flip[0] as boolean;
    
    // calculate edges of the shape
    let mut tx = (*dps.psp).sx.wrapping_sub((160 * FRACUNIT) as i32);
    
    tx -= *spriteoffset.offset(lump as isize); 
    let x1 = (centerxfrac + FixedMul (tx,pspritescale) ) >>FRACBITS;

    // off the right side
    if x1 > viewwidth {
        return;  
    }

    tx += *spritewidth.offset(lump as isize);
    let x2 = ((centerxfrac + FixedMul (tx, pspritescale) ) >>FRACBITS) - 1;

    // off the left side
    if x2 < 0 {
        return;
    }
    
    // store information in a vissprite
    let mut avis: [vissprite_t; 1] = [vissprite_t {
        prev: std::ptr::null_mut(),
        next: std::ptr::null_mut(),
        gx: 0,
        gy: 0,
        gz: 0,
        gzt: 0,
        patch: lump as i32,
        colormap: std::ptr::null_mut(),
        mobjflags: 0,
        texturemid: ((BASEYCENTER<<FRACBITS) as i32 + (FRACUNIT/2) as i32).wrapping_sub(
                        (*dps.psp).sy.wrapping_sub(*spritetopoffset.offset(lump as isize))),
        x1: i32::max(x1, 0),
        x2: i32::min(x2, viewwidth - 1),
        scale: pspritescale<<detailshift,
        xiscale: if flip != c_false { -pspriteiscale } else { pspriteiscale },
        startfrac: if flip != c_false { *spritewidth.offset(lump as isize) - 1 } else { 0 },
    }];
    let mut vis = avis.as_mut_ptr();
    if (*vis).x1 > x1 {
        (*vis).startfrac += (*vis).xiscale*((*vis).x1-x1);
    }

    if ((*viewplayer).powers[pw_invisibility as usize] > 4*32)
    || (((*viewplayer).powers[pw_invisibility as usize] & 8) != 0) {
        // shadow draw
        (*vis).colormap = std::ptr::null_mut();
    } else if fixedcolormap != std::ptr::null_mut() {
        // fixed color
        (*vis).colormap = fixedcolormap;
    } else if (((*(*dps.psp).state).frame as u32) & FF_FULLBRIGHT) != 0 {
        // full bright
        (*vis).colormap = colormaps;
    } else {
        // local light
        (*vis).colormap = *spritelights.offset((MAXLIGHTSCALE - 1) as isize);
    }
    
    let mut dvs = R_DrawVisSprite_params_t {
        vis: vis,
        mfloorclip: dps.mfloorclip,
        mceilingclip: dps.mceilingclip,
    };
    R_DrawVisSprite (&mut dvs);
}

//
// R_DrawPlayerSprites
//
unsafe fn R_DrawPlayerSprites () {
    // get light level
    let lightnum =
    ((*(*(*(*viewplayer).mo).subsector).sector).lightlevel >> LIGHTSEGSHIFT) as i32
    +extralight;

    spritelights = scalelight[i32::max(0, i32::min((LIGHTLEVELS - 1) as i32, lightnum)) as usize].as_mut_ptr();
    
    let mut dps = R_DrawPSprite_params_t {
        // clip to screen bounds
        mfloorclip: screenheightarray.as_mut_ptr(),
        mceilingclip: negonearray.as_mut_ptr(),
        // add all active psprites
        psp: (*viewplayer).psprites.as_mut_ptr(),
    };
 
    
    for _ in 0 .. NUMPSPRITES {
        if (*dps.psp).state != std::ptr::null_mut() {
            R_DrawPSprite (&mut dps);
        }
        dps.psp = dps.psp.offset(1);
    }
}

//
// R_DrawSprite
//
unsafe fn R_DrawSprite (spr: *mut vissprite_t) {
    // Only (*spr).x1 ..= (*spr).x2 is actually used
    let mut clipbot: [i16; SCREENWIDTH as usize] = [-2; SCREENWIDTH as usize];
    let mut cliptop: [i16; SCREENWIDTH as usize] = [-2; SCREENWIDTH as usize];
    
    // Scan drawsegs from end to start for obscuring segs.
    // The first drawseg that has a greater scale
    //  is the clip seg.
    let mut ds: *mut drawseg_t = ds_p;
    loop {
        ds = ds.offset(-1);
        if ds < drawsegs.as_mut_ptr() {
            break;
        }

        // determine if the drawseg obscures the sprite
        if ((*ds).x1 > (*spr).x2)
        || ((*ds).x2 < (*spr).x1)
        || (((*ds).silhouette == 0) && ((*ds).maskedtexturecol == std::ptr::null_mut())) {
            // does not cover sprite
            continue;
        }
                
        let r1 = i32::max((*ds).x1, (*spr).x1) as usize;
        let r2 = i32::min((*ds).x2, (*spr).x2) as usize;
        let scale: fixed_t;
        let lowscale: fixed_t;

        if (*ds).scale1 > (*ds).scale2 {
            lowscale = (*ds).scale2;
            scale = (*ds).scale1;
        } else {
            lowscale = (*ds).scale1;
            scale = (*ds).scale2;
        }
            
        if (scale < (*spr).scale)
        || ((lowscale < (*spr).scale) && 0 == R_PointOnSegSide ((*spr).gx, (*spr).gy, (*ds).curline)) {
            // masked mid texture?
            if (*ds).maskedtexturecol != std::ptr::null_mut() {
                R_RenderMaskedSegRange (ds, r1 as i32, r2 as i32);
            }
            // seg is behind sprite
            continue;
        }

        
        // clip this piece of the sprite
        let mut silhouette = (*ds).silhouette;
        
        if (*spr).gz >= (*ds).bsilheight {
            silhouette &= !SIL_BOTTOM as i32;
        }

        if (*spr).gzt <= (*ds).tsilheight {
            silhouette &= !SIL_TOP as i32;
        }
                
        if silhouette == 1 {
            // bottom sil
            for x in r1 ..= r2 {
                if clipbot[x] == -2 {
                    clipbot[x] = *(*ds).sprbottomclip.offset(x as isize);
                }
            }
        } else if silhouette == 2 {
            // top sil
            for x in r1 ..= r2 {
                if cliptop[x] == -2 {
                    cliptop[x] = *(*ds).sprtopclip.offset(x as isize);
                }
            }
        } else if silhouette == 3 {
            // both
            for x in r1 ..= r2 {
                if clipbot[x] == -2 {
                    clipbot[x] = *(*ds).sprbottomclip.offset(x as isize);
                }
                if cliptop[x] == -2 {
                    cliptop[x] = *(*ds).sprtopclip.offset(x as isize);
                }
            }
        }
    }
    
    // all clipping has been performed, so draw the sprite

    // check for unclipped columns
    for x in (*spr).x1 as usize ..= (*spr).x2 as usize {
        if clipbot[x] == -2 {
            clipbot[x] = viewheight as i16;
        }

        if cliptop[x] == -2 {
            cliptop[x] = -1;
        }
    }
    
    let mut dvs = R_DrawVisSprite_params_t {
        vis: spr,
        mfloorclip: clipbot.as_mut_ptr(),
        mceilingclip: cliptop.as_mut_ptr(),
    };
    R_DrawVisSprite (&mut dvs);
}




//
// R_DrawMasked
//
pub unsafe fn R_DrawMasked () {
    // Sort sprites according to scale
    let mut sorted_sprites: Vec<*mut vissprite_t> = Vec::new();
    let mut iter: *mut vissprite_t = vissprites.as_mut_ptr();

    while iter != vissprite_p {
        sorted_sprites.push(iter);
        iter = iter.offset(1);
    }
    sorted_sprites.sort_by(|a, b| (*(*a)).scale.cmp(&(*(*b)).scale));

    // draw all vissprites back to front
    for spr in sorted_sprites {
	    R_DrawSprite (spr);
    }
    
    // render any remaining masked mid textures
    let mut ds: *mut drawseg_t = ds_p.offset(-1);
    while ds >= drawsegs.as_mut_ptr() {
        if (*ds).maskedtexturecol != std::ptr::null_mut() {
            R_RenderMaskedSegRange (ds, (*ds).x1, (*ds).x2);
        }
        ds = ds.offset(-1);
    }

    // draw the psprites on top of everything
    //  but does not draw on side views
    if viewangleoffset == 0 {
        R_DrawPlayerSprites ();
    }
}



