
#include <sys/time.h>
#include <stdio.h>

int gettimeofday(struct timeval *tv, struct timezone *tz)
{
    tv->tv_sec = 0;
    tv->tv_usec = 0;
    return 0;
}

FILE *fopen(const char *pathname, const char *mode)
{
    return NULL;
}

int fclose(FILE *stream)
{
    return 0;
}

int fseek(FILE *stream, long offset, int whence)
{
    return 0;
}

long ftell(FILE *stream)
{
    return 0;
}

size_t fread(void *ptr, size_t size, size_t nmemb, FILE *stream)
{
    return 0;
}

size_t fwrite(const void *ptr, size_t size, size_t nmemb, FILE *stream)
{
    return 0;
}

int printf(const char *format, ...)
{
    return 0;
}

int fprintf(FILE *stream, const char *format, ...)
{
    return 0;
}

void outbyte(int x)
{
}

int inbyte(void)
{
    return 0;
}

