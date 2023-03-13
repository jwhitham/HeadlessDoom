#include <stdio.h>
#include <unistd.h>
#include <stdlib.h>
#include <string.h>
#include <assert.h>
#include <limits.h>
#include <stdint.h>

#include <sys/types.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <signal.h>
#include <ctype.h>

#include "mb_mem.h"
#include "mb_core.h"
#include "mb_elf.h"

#define CLOCK_FREQUENCY_MHZ     100

typedef struct user_data_t {
    uint64_t microseconds;
    MB_Context *mb;
} user_data_t;

static void Trace (void * t_user, MB_Context * mb, 
                    MB_Trace_Name trace_name, const void * param)
{
    switch (trace_name) {
    case MB_INTERRUPT:
        fprintf(stderr, "Interrupt?\n");
        exit(1);
    case MB_SIM_CMD:
    case MB_ILLEGAL_INST:
        fprintf(stderr, "Illegal instruction 0x%x at 0x%x\n",
                    mb->cur_iword, mb->cur_pc);
        exit(1);
    case MB_ALIGN:
        fprintf(stderr, "Alignment error with EA 0x%x at 0x%x\n",
                    (uint32_t) ((intptr_t) param), mb->cur_pc);
        exit(1);
    case MB_EXECUTE:
        break;
    default:
        return;
    }
}


static void Put(void * m_user, uint32_t flags, uint32_t fsl, uint32_t data)
{
    // see shim_asm.S for definitions of these functions
    switch (fsl) {
        case 0:
            // outbyte function
            if (isprint(data) || data == '\n' || data == '\x08') {
                fputc(data, stdout);
                fflush(stdout);
            }
            break;
        case 1:
            // exit function
            exit(data);
        default:
            fprintf(stderr, "Cannot execute put instruction for FSL %d\n", fsl);
            exit(1);
    }
}

static uint32_t Get(void * m_user, uint32_t flags, uint32_t fsl)
{
    // see shim_asm.S
    struct MB_System_Context_struct *sc = (struct MB_System_Context_struct *) m_user;
    MB_Context * mb = MB_System_Get_MB_Context(sc);
    user_data_t *user_data = (user_data_t *) mb->t_user;
    switch (fsl) {
        case 0:
            // gettimeofday (seconds - and capture)
            user_data->microseconds = (user_data->mb->clock_cycle /
                                (uint64_t) CLOCK_FREQUENCY_MHZ);
            return user_data->microseconds / 1000000;
        case 1:
            // gettimeofday (microseconds when the seconds were captured)
            return user_data->microseconds % 1000000;
        default:
            fprintf(stderr, "Cannot execute get instruction for FSL %d\n", fsl);
            exit(1);
    }
}

int main ( int argc , char ** argv )
{
    struct MB_System_Context_struct * sys ;
    MB_Context * mb;
    const char * out;
    user_data_t user_data;
    uint32_t i, j;
    uint32_t args_base = 0xffff0000U;
    uint32_t args_ptr;

    if (argc < 2) {
        printf("Usage: %s <elf binary> [more args...]\n", argv[0]);
        return 1;
    }
    memset(&user_data, 0, sizeof(user_data));
    sys = MB_System_Init(Trace, 1, &user_data);
    mb = MB_System_Get_MB_Context(sys);
    mb->put_fn = Put;
    mb->get_fn = Get;
    user_data.mb = mb;

    // copy args
    args_ptr = args_base + (argc * 4);
    for (i = 1; i < (uint32_t) argc; i++) {
        mb->store_fn(sys, args_base + ((i - 1) * 4), args_ptr, 4);
        for (j = 0; argv[i][j] != '\0'; j++) {
            mb->store_fn(sys, args_ptr, argv[i][j], 1);
            args_ptr++;
        }
        mb->store_fn(sys, args_ptr, 0, 1);
        args_ptr++;
    }
    mb->store_fn(sys, args_base + ((argc - 1) * 4), 0, 4);

    // load program
    out = MB_Read_Elf(sys, argv[1]);
    if (out != NULL) {
        printf("ELF read error: %s\n", out);
        return 1;
    }

    while (1) {
        MB_Step(mb, 0);
    }
}

