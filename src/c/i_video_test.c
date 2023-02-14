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

extern unsigned headless_count;

static FILE * crc_out = NULL;



void I_ShutdownGraphics(void)
{
    if (crc_out) {
        fclose (crc_out);
        crc_out = NULL;
    }
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

byte cur_palette [768];

//
// I_FinishUpdate
//
void I_FinishUpdate (void)
{
    /* Here is where screens[0] is passed to CRC-32 */
    unsigned crc, v1, v2;

    v1 = v2 = 0;
    crc = crc32_8bytes (screens[0], SCREENHEIGHT * SCREENWIDTH, 0);
    headless_count ++;
#ifdef WRITE_CRC
    fprintf (crc_out, "%08x %u\n", crc, headless_count);
    fflush (crc_out);
#else
    if (2 != fscanf (crc_out, "%08x %u", &v1, &v2)) {
        I_Error ("Couldn't read CRC and frame number from 'crc.dat' frame %u",
                headless_count);
    }
    if (v2 != headless_count) {
        I_Error ("Incorrect frame number in 'crc.dat', expected %u got %u",
                headless_count, v2);
    }
    if (v1 != crc) {
        I_Error ("Incorrect CRC-32, frame %u, "
                 "expected %08x got %08x",
                headless_count, v1, crc);
    }
#endif
#if 0
    if (headless_count >= MAX_CHECKED_FRAMES) {
        I_Error ("Unable to access 'crc.dat'");
        printf ( "check %u: %08x: unverified\n",
               headless_count, crc);
    } else if (crc_check[headless_count] != crc) {

        printf ( "check %u: %08x: failed\n",
               headless_count, crc);
        headless_failed ++ ;
    }
#endif

#ifdef WRITE_BINS
    if ((headless_count % 10) == 0) {
        char name [32];
        FILE * fd;

        snprintf (name, sizeof (name), "bins/%05u.bin", headless_count / 10);
        fd = fopen (name, "wb");
        if (fd != NULL) {
            fwrite (screens[ 0 ], SCREENHEIGHT * SCREENWIDTH, 1, fd);
            fwrite (cur_palette, 768, 1 , fd);
            fclose (fd);
        }
    }
#endif
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
    memcpy ( cur_palette, palette, 768 ) ;
    // palette is an array of 256 RGB triples.
    // i.e. 768 bytes
}


void I_InitGraphics(void)
{
    static int		firsttime=1;

    if (!firsttime) return;

    firsttime = 0;

	screens[0] = (unsigned char *) malloc (SCREENWIDTH * SCREENHEIGHT);

#ifdef WRITE_CRC
    printf ("Headless Doom running in Test (write) mode\n");
    crc_out = fopen ("crc.dat", "wt");
#else
    printf ("Headless Doom running in Test mode\n");
    crc_out = fopen ("crc.dat", "rt");
#endif
    if (!crc_out) {
        I_Error ("Unable to access 'crc.dat'");
    }
}

