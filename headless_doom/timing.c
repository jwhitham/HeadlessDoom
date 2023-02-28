
#include "headless.h"

#ifdef _MSC_VER
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

