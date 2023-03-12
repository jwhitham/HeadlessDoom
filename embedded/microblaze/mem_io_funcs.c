
#include <fcntl.h>
#include <errno.h>
#include <sys/types.h>
#include <unistd.h>
#include <stdint.h>
#include <string.h>
#include <stdlib.h>


typedef struct file_table_entry_t {
    const char *name;
    const char *start;
    const char *end;
} file_table_entry_t;

extern char _binary_______headless_doom_DDQ_EP1_LMP_start;
extern char _binary_______headless_doom_DDQ_EP2_LMP_start;
extern char _binary_______headless_doom_DDQ_EP3_LMP_start;
extern char _binary_______headless_doom_DDQ_EP4_LMP_start;
extern char _binary_______headless_doom_doom_wad_start;
extern char _binary_______headless_doom_DDQ_EP1_LMP_end;
extern char _binary_______headless_doom_DDQ_EP2_LMP_end;
extern char _binary_______headless_doom_DDQ_EP3_LMP_end;
extern char _binary_______headless_doom_DDQ_EP4_LMP_end;
extern char _binary_______headless_doom_doom_wad_end;

static const file_table_entry_t file_table[] = {
    {
        "DDQ-EP1.LMP",
        &_binary_______headless_doom_DDQ_EP1_LMP_start,
        &_binary_______headless_doom_DDQ_EP1_LMP_end,
    },
    {
        "DDQ-EP2.LMP",
        &_binary_______headless_doom_DDQ_EP2_LMP_start,
        &_binary_______headless_doom_DDQ_EP2_LMP_end,
    },
    {
        "DDQ-EP3.LMP",
        &_binary_______headless_doom_DDQ_EP3_LMP_start,
        &_binary_______headless_doom_DDQ_EP3_LMP_end,
    },
    {
        "DDQ-EP4.LMP",
        &_binary_______headless_doom_DDQ_EP4_LMP_start,
        &_binary_______headless_doom_DDQ_EP4_LMP_end,
    },
    {
        "doom.wad",
        &_binary_______headless_doom_doom_wad_start,
        &_binary_______headless_doom_doom_wad_end,
    },
    {
        NULL,
        NULL,
        NULL,
    },
};

typedef struct file_handle_entry_t {
    const file_table_entry_t *file;
    size_t position;
    size_t size;
} file_handle_entry_t;

#define NUM_HANDLES 8
#define FIRST_USABLE_HANDLE 3
static file_handle_entry_t handle_table[NUM_HANDLES];

int open(const char *pathname, int flags, ...)
{
    size_t i;
    const file_table_entry_t *file = NULL;

    (void) flags;
    for (i = 0; file_table[i].name; i++) {
        if (strcmp(file_table[i].name, pathname) == 0) {
            file = &file_table[i];
            break;
        }
    }
    if (file == NULL) {
        // File not found
        errno = ENOENT;
        return -1;
    }

    for (i = FIRST_USABLE_HANDLE; i < NUM_HANDLES; i++) {
        if (handle_table[i].file == NULL) {
            file_handle_entry_t *handle = &handle_table[i];
            handle->file = file;
            handle->position = 0;
            handle->size = (size_t)((intptr_t) file->end - (intptr_t) file->start);
            return i;
        }
    }
    errno = EMFILE;
    return -1;
}

static file_handle_entry_t *get_handle(int fd)
{
    file_handle_entry_t *handle;

    if (((unsigned) fd) >= NUM_HANDLES) {
        return NULL;
    }

    handle = &handle_table[fd];
    if (handle->file == NULL) {
        return NULL;
    }
    return handle;
}

ssize_t read(int fd, void *buf, size_t count)
{
    file_handle_entry_t *handle = get_handle(fd);
    size_t bound;

    if (handle == NULL) {
        errno = EBADF;
        return -1;
    }

    if (handle->position > handle->size) {
        handle->position = handle->size;
    }
    bound = handle->position + count;
    if ((bound > handle->size) || (bound < handle->position)) {
        bound = handle->size;
    }
    count = bound - handle->position;
    if (count > 0) {
        memcpy(buf, &handle->file->start[handle->position], count);
    }
    handle->position = bound;
    return (ssize_t) count;
}

extern void outbyte(int x);

ssize_t write(int fd, const void *buf, size_t count)
{
    if ((fd == 1) || (fd == 2)) {
        size_t i;
        for (i = 0; i < count; i++) {
            outbyte(*((const char*) buf));
            buf++;
        }
        return count;
    } else {
        errno = EBADF;
        return -1;
    }
}

int close(int fd)
{
    file_handle_entry_t *handle = get_handle(fd);

    if (handle == NULL) {
        errno = EBADF;
        return -1;
    }

    handle->file = NULL;
    return 0;
}

off_t lseek(int fd, off_t offset, int whence)
{
    file_handle_entry_t *handle = get_handle(fd);

    if (handle == NULL) {
        errno = EBADF;
        return -1;
    }

    switch (whence) {
        case SEEK_SET:
            handle->position = offset;
            break;
        case SEEK_CUR:
            handle->position += offset;
            break;
        case SEEK_END:
            handle->position = (size_t) ((off_t) handle->size + (off_t) offset);
            break;
        default:
            errno = EINVAL;
            return -1;
    }
    return handle->position;
}


int inbyte(void)
{
    return 0;
}
