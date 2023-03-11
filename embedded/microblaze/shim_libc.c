
#include <sys/time.h>
#include <stdio.h>

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

int inbyte(void)
{
    return 0;
}

