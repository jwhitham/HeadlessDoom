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
//	All the clipping: columns, horizontal spans, sky columns.
//
//-----------------------------------------------------------------------------


static const char
rcsid[] = "$Id: r_segs.c,v 1.3 1997/01/29 20:10:19 b1 Exp $";





#include <stdlib.h>

#include "i_system.h"

#include "doomdef.h"
#include "doomstat.h"

#include "r_local.h"
#include "r_sky.h"


// OPTIMIZE: closed two sided lines as single sided

// True if any of the segs textures might be visible.
boolean		segtextured;	 // shared with r_bsp

// False if the back side is the same plane.
boolean		markfloor;	 // shared with r_bsp
boolean		markceiling; // shared with r_bsp

int		toptexture;     // shared with r_defs, r_data, others
int		bottomtexture;      // shared with r_defs, r_data, others
int		midtexture;     // shared with r_defs, r_data, others


angle_t		rw_normalangle; // shared with r_main, r_state
// angle to line origin
int		rw_angle1;	// shared with r_bsp, r_state

//
// regular wall
//
int		rw_x;       // used in r_bsp
int		rw_stopx;       // used in r_bsp
fixed_t		rw_distance;    // shared with r_main, r_state


lighttable_t**	walllights; // shared with r_main

short*		maskedtexturecol; // shared with r_defs, r_things











