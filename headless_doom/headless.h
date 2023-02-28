
#include <stdint.h>
#include <stdlib.h>

#ifdef __GNUC__
#else
#ifdef _MSC_VER
#define alloca _alloca
#define strcasecmp _stricmp
#define strncasecmp _strnicmp
#else
#error "Only GCC and MSVC are currently supported; please add support to headless.h"
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
