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
//	DOOM graphics stuff for X11, UNIX.
//
//-----------------------------------------------------------------------------


#include <stdlib.h>
#include <unistd.h>
#if 0
#include <sys/ipc.h>
#include <sys/shm.h>

#include <sys/types.h>
#include <sys/stat.h>
#include <fcntl.h>


#include <stdarg.h>
#include <sys/time.h>
#include <sys/types.h>
#include <sys/socket.h>

#include <netinet/in.h> 
#include <errno.h>
#include <signal.h> 
#endif

#include "doomstat.h"
#include "i_system.h"
#include "v_video.h"
#include "m_argv.h"
#include "d_main.h"

#include "crc.h"

#include "doomdef.h"

#define POINTER_WARP_COUNTDOWN	1

extern unsigned headless_count;

void I_ShutdownGraphics(void)
{
}



void I_StartFrame (void)
{
}

void I_GetEvent(void)
{
    /* Where events would be received from the X server */
}

//
// I_StartTic
//
void I_StartTic (void)
{
    /* Poll for events (calling I_GetEvent while events are in
     * the queue) and stop the mouse pointer leaving the window
     * as a dirty keep-focus hack
     */
}


//
// I_UpdateNoBlit
//
void I_UpdateNoBlit (void)
{
    // what is this?
}

//
// I_FinishUpdate
//
void I_FinishUpdate (void)
{
    headless_count ++ ;
}


//
// I_ReadScreen
//
void I_ReadScreen (byte* scr)
{
    memcpy (scr, screens[0], SCREENWIDTH*SCREENHEIGHT);
}


//
// Palette stuff.
//

//
// I_SetPalette
//
void I_SetPalette (byte* palette)
{
}


void I_InitGraphics(void)
{
    static int		firsttime=1;

    if (!firsttime) return;

    firsttime = 0;

	screens[0] = (unsigned char *) malloc (SCREENWIDTH * SCREENHEIGHT);
    printf ("Headless Doom running in Benchmark mode\n");
}

