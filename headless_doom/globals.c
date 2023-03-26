
#include "doomdef.h"

#include "i_system.h"
#include "z_zone.h"
#include "w_wad.h"

#include "r_local.h"

// Needs access to LFB (guess what).
#include "v_video.h"

// State.
#include "doomstat.h"


// from r_draw.c
//
 #define MAXWIDTH			1120
 #define MAXHEIGHT			832
byte*		viewimage; 
int		viewwidth;
int		scaledviewwidth;
int		viewheight;
int		viewwindowx;
int		viewwindowy; 
byte*		ylookup[MAXHEIGHT]; 
int		columnofs[MAXWIDTH]; 

// Color tables for different players,
//  translate a limited part to another
//  (color ramps used for  suit colors).
//
byte		translations[3][256];	
 
 


//
// R_DrawColumn
// Source is the top of the column to scale.
//
lighttable_t*		dc_colormap; 
int			dc_x; 
int			dc_yl; 
int			dc_yh; 
fixed_t			dc_iscale; 
fixed_t			dc_texturemid;

// first pixel in a column (possibly virtual) 
byte*			dc_source;		

// just for profiling 
int			dccount;


int                    ds_y;
int                    ds_x1;
int                    ds_x2;

lighttable_t*          ds_colormap;

fixed_t                        ds_xfrac;
fixed_t                        ds_yfrac;
fixed_t                        ds_xstep;
fixed_t                        ds_ystep;

// start of a 64*64 tile image
byte*                  ds_source;


byte*  dc_translation;
byte*  translationtables;

