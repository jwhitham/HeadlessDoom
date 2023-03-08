
#include "doomstat.h"
#include "i_system.h"
#include "v_video.h"
#include "m_argv.h"
#include "d_main.h"
#include "m_misc.h"

#include "doomdef.h"
#include "sounds.h"
#include "s_sound.h"
#include "g_game.h"

#include "headless.h"

typedef enum {
    BENCHMARK,
    TEST,
    TEST_PCX,
    TEST_BIN,
    WRITE_CRC,
    WRITE_CRC_PCX,
    WRITE_CRC_BIN,
} t_headless_mode;

static t_headless_mode headless_mode = BENCHMARK;
static byte cur_palette [768];
unsigned headless_count;
static unsigned test_start_frame = 0;
static unsigned test_end_frame = 99999;
static FILE * crc_out = NULL;
static FILE * bin_out = NULL;
static FILE * bin_in = NULL;
static int fake_time = 0;
static uint64_t start_time = 0;
byte* save_p;


extern boolean singletics; // d_main: debug flag to cancel adaptiveness
extern boolean advancedemo; // from d_main
extern int demosequence;
extern int pagetic;
extern char *pagename;
extern void WritePCXfile(char* filename, byte* data, int width, int height, byte* palette);
extern int detailLevel;
extern int screenblocks;
extern int showMessages;



void I_ShutdownGraphics(void)
{
    if (crc_out) {
        fclose (crc_out);
        crc_out = NULL;
    }
}

void I_FinishUpdate (void)
{
    unsigned crc, v1, v2, x, y, diff;

    headless_count ++;
    if (headless_mode == BENCHMARK) {
        return;
    }

    /* read/write screenshots and reference files */
    switch (headless_mode) {
        case WRITE_CRC_PCX:
        case TEST_PCX:
            /* write PCX screenshots */
            if (headless_count >= test_start_frame) {
                char name [32];

                snprintf (name, sizeof (name), "%05u.pcx", headless_count);
                WritePCXfile (name, screens[0],
                      SCREENWIDTH, SCREENHEIGHT,
                      cur_palette);
            }
            break;
        case WRITE_CRC_BIN:
            /* write reference binary file */
            if (fwrite (screens[0], SCREENHEIGHT * SCREENWIDTH, 1, bin_out) != 1) {
                I_Error("Unable to write frame %u to reference.dat",
                        headless_count);
            }
            break;
        case TEST_BIN:
            /* read reference binary file, compare */
            diff = 0;
            for (y = 0; y < SCREENHEIGHT; y++) {
                byte ref[SCREENWIDTH];
                byte* now = &screens[0][y * SCREENWIDTH];

                if (fread (ref, SCREENWIDTH, 1, bin_in) != 1) {
                    I_Error("Unable to read frame %u y %u from reference.dat",
                            headless_count, y);
                }
                for (x = 0; x < SCREENWIDTH; x++) {
                    if (ref[x] != now[x]) {
                        if (diff == 0) {
                            printf("different: frame %u x %u y %u expected 0x%02x got 0x%02x\n",
                                    headless_count, x, y, ref[x], now[x]);
                            fflush(stdout);
                        }
                        diff++;
                    }
                }
            }
            if (diff > 1) {
                printf("frame %u has %u more differences\n",
                        headless_count, diff - 1);
                fflush(stdout);
            }
            if (diff > (SCREENWIDTH * SCREENHEIGHT / 4)) {
                I_Error("Too many differences! Game desynced?");
            }
            break;
        case TEST:
        case WRITE_CRC:
        case BENCHMARK:
            break;
    }

    /* Here is where screens[0] is passed to CRC-32 */
    v1 = v2 = 0;
    crc = crc32_8bytes (screens[0], SCREENHEIGHT * SCREENWIDTH, 0);
    switch (headless_mode) {
        case TEST:
        case TEST_PCX:
            if (2 != fscanf (crc_out, "%08x %u", &v1, &v2)) {
                I_Error ("Couldn't read CRC and frame number from 'crc.dat' frame %u",
                        headless_count);
            }
            if (v2 != headless_count) {
                I_Error ("Incorrect frame number in 'crc.dat', expected %u got %u",
                        headless_count, v2);
            }
            if ((headless_count >= test_start_frame) && (v1 != crc)) {
                I_Error ("Incorrect CRC-32, frame %u, "
                         "expected %08x got %08x",
                        headless_count, v1, crc);
            }
            break;
        case WRITE_CRC:
        case WRITE_CRC_PCX:
        case WRITE_CRC_BIN:
            fprintf (crc_out, "%08x %u\n", crc, headless_count);
            fflush (crc_out);
            break;
        case TEST_BIN:
        case BENCHMARK:
            break;
    }

    if (headless_count >= test_end_frame) {
        printf ("reached final frame\n");
        exit (0);
    }
}

void I_ReadScreen (byte* scr)
{
    memcpy (scr, screens[0], SCREENWIDTH*SCREENHEIGHT);
}

void I_SetPalette (byte* palette)
{
    memcpy ( cur_palette, palette, 768 ) ;
    // palette is an array of 256 RGB triples.
    // i.e. 768 bytes
}

void I_InitGraphics (void)
{
}

void I_InitNetwork (void)
{
    doomcom = malloc (sizeof (*doomcom) );
    memset (doomcom, 0, sizeof(*doomcom) );
    doomcom->id = DOOMCOM_ID;
    doomcom->numnodes = 1;
    doomcom->numplayers = doomcom->numnodes;
    doomcom->ticdup = 1;
}

int I_GetTime (void)
{
    return fake_time++;
}

void M_CheckAddFile(const char* name, unsigned expect_crc)
{
    if (headless_mode != BENCHMARK) {
        /* Test the input files */
        char buf[1024];
        unsigned crc = 0;
        int size;
        FILE* fd;

        fd = fopen(name, "rb");
        if (!fd) {
            I_Error ("Required file '%s' is missing", name);
            return;
        }
        while ((size = (int) fread(buf, 1, sizeof(buf), fd)) > 0) {
            crc = crc32_8bytes (buf, size, crc);
        }
        if (crc != expect_crc) {
            I_Error ("Required file '%s' has an unexpected CRC; expected 0x%08x got 0x%08x",
                        name, expect_crc, crc);
            return;
        }
        fclose(fd);
    }
    D_AddFile((char*) name);
}

void IdentifyVersion (void)
{
    const char * mode = "";

    if (myargc > 1) {
        mode = myargv[1];
    }
    if (myargc > 3) {
        test_start_frame = (unsigned) atoi(myargv[2]);
        test_end_frame = (unsigned) atoi(myargv[3]);
    }

    if (strcmp(mode, "test") == 0) {
        headless_mode = TEST;
    } else if (strcmp(mode, "test_pcx") == 0) {
        headless_mode = TEST_PCX;
    } else if (strcmp(mode, "test_bin") == 0) {
        headless_mode = TEST_BIN;
    } else if (strcmp(mode, "write_crc") == 0) {
        headless_mode = WRITE_CRC;
    } else if (strcmp(mode, "write_pcx") == 0) {
        headless_mode = WRITE_CRC_PCX;
    } else if (strcmp(mode, "write_bin") == 0) {
        headless_mode = WRITE_CRC_BIN;
    } else if ((strcmp(mode, "") == 0)
    || (strcmp(mode, "benchmark") == 0)) {
        headless_mode = BENCHMARK;
    } else {
        I_Error ("Unknown mode '%s' - use 'test' or 'benchmark'", mode);
    }

    switch (headless_mode) {
        case TEST:
        case TEST_PCX:
        case TEST_BIN:
            printf ("Headless Doom running in Test mode\n");
            crc_out = fopen ("crc.dat", "rt");
            if (!crc_out) {
                I_Error ("Unable to read 'crc.dat'");
            }
            break;
        case WRITE_CRC:
        case WRITE_CRC_PCX:
        case WRITE_CRC_BIN:
            printf ("Headless Doom running in Test (write) mode\n");
            crc_out = fopen ("crc.dat", "wt");
            if (!crc_out) {
                I_Error ("Unable to create 'crc.dat'");
            }
            break;
        case BENCHMARK:
            printf ("Headless Doom running in Benchmark mode\n");
            break;
    }

    switch (headless_mode) {
        case TEST:
        case TEST_PCX:
        case WRITE_CRC:
        case WRITE_CRC_PCX:
        case BENCHMARK:
            break;
        case WRITE_CRC_BIN:
            bin_out = fopen ("reference.dat", "wb");
            if (!bin_out) {
                I_Error ("Unable to create 'reference.dat'");
            }
            break;
        case TEST_BIN:
            bin_in = fopen ("reference.dat", "rb");
            if (!bin_in) {
                I_Error ("Unable to read 'reference.dat'");
            }
            break;
    }
    fflush (stdout);
    start_time = M_GetTimeMicroseconds();
    gamemode = retail;
    M_CheckAddFile ("doom.wad",    0xbf0eaac0U);
    M_CheckAddFile ("DDQ-EP1.LMP", 0x29df95fcU);
    M_CheckAddFile ("DDQ-EP2.LMP", 0xf8538520U);
    M_CheckAddFile ("DDQ-EP3.LMP", 0x29a5a958U);
    M_CheckAddFile ("DDQ-EP4.LMP", 0x9b9f2a3eU);
    singletics = true;
}

void D_DoAdvanceDemo (void)
{
    uint64_t stop_time;

    players[consoleplayer].playerstate = PST_LIVE;  // not reborn
    advancedemo = false;
    usergame = false;               // no save / end game here
    paused = false;
    gameaction = ga_nothing;

    demosequence++;
    printf ("demo sequence %d\n", demosequence);
    
    switch (demosequence)
    {
        case 0:
            gamestate = GS_DEMOSCREEN;
            pagetic = 5;
            pagename = "TITLEPIC";
            S_StartMusic (mus_intro);
            break;
        case 1:
            G_DeferedPlayDemo ("DDQ-EP1");
            break;
        case 2:
            gamestate = GS_DEMOSCREEN;
            pagename = "CREDIT";
            pagetic = 5;
            break;
        case 3:
            G_DeferedPlayDemo ("DDQ-EP2");
            break;
        case 4:
            gamestate = GS_DEMOSCREEN;
            pagename = "CREDIT";
            pagetic = 5;
            break;
        case 5:
            G_DeferedPlayDemo ("DDQ-EP3");
            break;
        case 6:
            G_DeferedPlayDemo ("DDQ-EP4");
            break;
        default:
            switch (headless_mode) {
                case TEST:
                case TEST_PCX:
                    printf ("Test complete - %u frames tested ok\n", headless_count);
                    break;
                case WRITE_CRC:
                case WRITE_CRC_PCX:
                case WRITE_CRC_BIN:
                    printf ("Test (write) complete - CRC calculated for %u frames\n", headless_count);
                    break;
                case BENCHMARK:
                    stop_time = M_GetTimeMicroseconds();
                    printf ("Benchmark complete - %u frames\n", headless_count);
                    printf ("Total time %7.3f seconds\n",
                            (double) (stop_time - start_time) / 1.0e6);
                    break;
            }
            exit (0);
    }
}

void M_LoadDefaults(void)
{
    screenblocks = 9;
    detailLevel = 0;
    showMessages = 1;
    usegamma = 0;
    snd_SfxVolume = 8; // affects the status bar (first seen - frame 244)
}

// Stubs for sound/video/network/IO functions which are not used
int I_GetSfxLumpNum(void* sfx) { return 0; }
void I_InitSound(void) {}
void I_NetCmd (void) {}
void I_PauseSong (int handle) {}
void I_ResumeSong (int handle) {}
void I_StopSong(int handle) {}
void I_UnRegisterSong(int handle) {}
int I_RegisterSong(void* data) { return 1; }
int I_QrySongPlaying(int handle) { return 0; }
void I_PlaySong(int handle, int looping) {}
void I_SetChannels(void) {}
void I_SetMusicVolume(int volume) {}
void I_InitMusic(void) {}
void I_ShutdownMusic(void) {}
void I_ShutdownSound(void) {}
int I_SoundIsPlaying(int handle) { return 0; }
int I_StartSound(int id, int vol, int sep, int pitch, int priority) { return 0; }
void I_StopSound(int handle) {}
void I_SubmitSound(void) {}
void I_UpdateSoundParams(int handle, int vol, int sep, int pitch) {}
void I_StartFrame(void) {}
void I_GetEvent(void) {}
void I_StartTic(void) {}
void I_UpdateNoBlit(void) {}
void I_WaitVBL(int count) {}
void M_SaveDefaults(void) {}
void P_ArchivePlayers () {}
void P_ArchiveWorld () {}
void P_ArchiveThinkers () {}
void P_ArchiveSpecials () {}
void P_UnArchivePlayers () {}
void P_UnArchiveWorld () {}
void P_UnArchiveThinkers () {}
void P_UnArchiveSpecials () {}

