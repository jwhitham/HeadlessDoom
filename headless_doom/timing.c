
#include "headless.h"

// timing.c must provide M_GetTimeMicroseconds() which returns
// a microsecond value as a uint64_t. This is called at the beginning
// and end of the benchmark and the difference is used to calculate the
// running time. The suggested implementations use the real time clock,
// but you may prefer to use some other counter.

#ifdef _MSC_VER
// For Microsoft Visual C
#include <Windows.h>
uint64_t M_GetTimeMicroseconds() {
    SYSTEMTIME st;
    FILETIME ft;
    uint64_t time;
    // get time in hour:minute:second form
    GetSystemTime(&st);
    // convert to a count of 100 nanosecond intervals since some epoch
    SystemTimeToFileTime(&st, &ft);
    // store in a single 64-bit value
    time = ((uint64_t) ft.dwLowDateTime);
    time += ((uint64_t) ft.dwHighDateTime) << 32;
    // convert to microseconds
    return time / 10;
}
#else
#ifdef __WATCOMC__
// For OpenWatcom (MS-DOS)
#include <dos.h>
uint64_t M_GetTimeMicroseconds() {
    struct dostime_t t;
    uint64_t time;
    static uint64_t previous_call;

    _dos_gettime(&t);
    time = (uint64_t) t.hour;
    time = (time * 60) + (uint64_t) t.minute;
    time = (time * 60) + (uint64_t) t.second;
    time = (time * 100) + (uint64_t) t.hsecond;
    time = time * 10000;    // microseconds
    while (time < previous_call) {
        // wrap over midnight
        time += (uint64_t) 24 * 60 * 60 * 1000000;
    }
    previous_call = time;
    return time;
}
#else
// For everything else - GCC / Clang fallback (Linux, MinGW, etc.)
#include <sys/time.h>
#include <stddef.h>
uint64_t M_GetTimeMicroseconds() {
    struct timeval st;
    uint64_t time;
    // get time in microseconds
    gettimeofday(&st, NULL);
    // store in a single 64-bit value
    time = ((uint64_t)st.tv_sec) * 1000000;
    time += (uint64_t)st.tv_usec;
    // convert to microseconds
    return time;
}
#endif
#endif

