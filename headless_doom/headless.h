
#include <stdint.h>
#include <stdlib.h>

// This location can be used to provide global workarounds
// for functions and macros that are not provided by your compiler
// or C library:

#ifdef __GNUC__
// Nothing required for GCC / Clang
#else
#ifdef _MSC_VER
// Workarounds for Microsoft Visual C
#define alloca _alloca
#define strcasecmp _stricmp
#define strncasecmp _strnicmp
#else
#ifdef __WATCOMC__
// Workarounds for Watcom C (MS-DOS)
#include <alloca.h>
#else
// add support for your compiler here
#endif
#endif
#endif

extern unsigned headless_count;
uint64_t M_GetTimeMicroseconds();
unsigned crc32_8bytes (const void *data, unsigned length, unsigned previousCrc32);
void IdentifyVersion (void);

#undef mkdir
#define mkdir(pathname, mode)
#undef access
#define access(pathname, mode) (-1)
