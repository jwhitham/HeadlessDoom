
"Headless Doom" is a benchmark/test program based on Doom. I have used
it for testing compilers, CPU simulators, FPGA hardware, timing analysis
software and a coverage testing tool. It is written in C
and should be portable to any 32/64-bit platform.

![Video of benchmark running](pic.gif)

Over more than ten years, I used versions of "Headless Doom" on 
x86 Linux, x64 Linux, x86 Windows, Microblaze (bare metal), Android and 
ARM Linux (RPi, RPi model 2, and Pandaboard). It uses the original 
source code release from id Software.

This source code is demo-compatible with
the original MS-DOS game and renders the game at the original 320x200
resolution. 56111 frames from the "Doom Done Quick" demo are rendered
in a way that is as similar to original Doom as possible given the
need to [fix some bugs](BUGS.md) in order to have repeatable behavior
and portable code.

# Requirements

Latest Version: 1.12

To run the benchmark or the test, you will need Ultimate Doom. You can
[buy a copy of the game from Steam](https://store.steampowered.com/app/2280/DOOM_1993/).

You will need the `doom.wad` data file, which must have the following MD5 sum:

    c4fe9fd920207691a9f493668e0a2083  doom.wad

In the Steam edition of the game, this file is found in the Steam directory
in a subdirectory such as `steamapps\common\Ultimate Doom\base`.

It may also be obtained by updating the registered MS-DOS version of Doom
to Ultimate Doom [using the official patches](https://www.doomworld.com/classicdoom/info/patches.php).

The benchmark and test both make use of "Doom Done Quick", a speedrun
that completes all 32 non-secret levels in under 20 minutes. The original
"DdQ-1941.zip" archive is included. For more details of DdQ, see:
http://quake.speeddemosarchive.com/quake/qdq/movies/ddq.html



# Instructions

Run `make` to compile with GCC/Clang, or build the project in Visual Studio.

Copy `doom.wad` into the `headless_doom` directory,
and unzip `DdQ-1941.zip` into the `headless_doom` directory.

Run `headless_doom.exe` to run the benchmark. This runs through 32 levels
of the game by playing the "Doom Done Quick" demo, then exits. As a final
step, the program prints the total time that elapsed. The program does not 
use your computer's real-time clock to limit the frame rate. All frames are
rendered regardless of CPU speed.

Run `headless_doom.exe test` to run the test. This does the same thing as the benchmark,
but also computes the CRC-32 of each frame rendered, and compares this 
against a "known good" list. As a result it's significantly slower. This
mode can be used to detect subtle software and hardware errors.

Other features such as `write_crc`, `write_pcx`, `write_bin`, `test_bin` exist
for maintenance and debugging (see `headless.c` for details).

# Typical benchmark timings

    Platform                     Compiler        Typical time   Version

    RPi 2 (ARMv7 1GHz)           GCC 4.6.3       77.3s          1.10
    RPi (ARMv6 700MHz)           GCC 4.6.3       217.1s         1.10
    RPi 3 (Cortex-A53 1.2GHz)    GCC 10.2.1      48.7s          1.12
    Windows x64 (Core i3 5005U)  GCC 4.7.2       10.9s          1.11
    Linux x64 (Core2 E8600)      GCC 4.7.2       9.4s           1.10
    Windows x86 (Core i5 2500)   GCC 4.7.4       6.8s           1.10
    Windows x64 (Core i3 8350)   GCC 8.3.1       4.9s           1.12
    Windows x64 (Core i3 8350)   MSVC 2019       4.4s           1.12
    Linux x64 (Core i3 3220)     GCC 4.7.2       6.9s           1.10
    Linux x86 (Core i3 3220)     GCC 4.1.2       7.5s           1.10
    Android 11.0 (SDMMAGPIE)     Clang 10.0.1    8.8s           1.12
    MS-DOS (Core i3 8350)        OpenWatcom 1.9  7.8s           1.12
    Linux PowerPC (E500)         GCC 4.9.2       81.4s          1.10
    Android 6.0 (Snapdragon 410) Clang 3.9.0     69.3s          1.10
    W10 Linux x64 (AMD A6-6310)  GCC 4.8.4       17.1s          1.10
    Microblaze (100MHz sim)      GCC 4.1.1       1189s          1.12

The CRC test typically requires 25% more time.

# Platform notes

I am interested in benchmark timings on unusual or vintage hardware - if you
would like to contribute these, please send them by email. Doom will
require at least 4Mb of RAM and a 32-bit CPU, and storing the data files
and executable will require about 16Mb of read-only memory of some sort
(e.g. Flash). To reduce the memory requirements, adjust `mb_used` in `i_system.c`.

Modern hardware (including some Android smartphones) can
run Headless Doom at around 10,000 frames per second.

The MS-DOS version was built with the Watcom C compiler 
and runs within DOS/4GW, just like Doom.

A C99-compatible compiler is required for definitions of `int64_t`
and others from `stdint.h`.  Older C compilers can be used if
suitable definitions in `stdint.h` are provided.

The Microblaze version ran on a simple simulator which assumed that each
instruction took exactly 1 clock cycle (no cache, pipeline or bus simulation)
and that the clock frequency was 100MHz. This was done primarily to ensure
that big-endian support is working and that all memory accesses are correctly
aligned, since most platforms are little-endian and tolerate unaligned accessses.

# Bugs

Headless Doom aims to be as similar to Doom as possible while
still meeting the requirements of a benchmark program, i.e. repeatable
behavior and portability.

Doom has various bugs which cause it to access undefined memory or
limit portability. Headless Doom fixes these, but does not make
other alterations. [I have classified all of the bugs which were
fixed in Headless Doom and assigned identifiers to them](BUGS.md).


# Videos

The Doom Done Quick demo may be watched here:
   https://www.youtube.com/watch?v=oZGRL8-bhhw

(This is not a recording of Headless Doom, which spends less time on the
title screens etc.)

Doom's rendering process is shown here:
   https://www.youtube.com/watch?v=ujXrQVyl610

This shows Headless Doom 1.10 running on a PowerPC E500 with execution slowed by
a factor of 16667, so that 60 microseconds of CPU time is one second of
video time. You can see how the game draws the walls, floor and sprites. See
also: https://www.jwhitham.org/2016/03/a-detailed-timing-trace-with-video.html )


