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



static void Trace (void * t_user, MB_Context * mc , 
                    MB_Trace_Name trace_name, const void * param)
{
    switch (trace_name) {
    case MB_INTERRUPT:
        fprintf(stderr, "Interrupt?\n");
        exit(1);
    case MB_SIM_CMD:
    case MB_ILLEGAL_INST:
        fprintf(stderr, "Illegal instruction 0x%x at 0x%x\n",
                    mc->cur_iword, mc->cur_pc);
        exit(1);
    case MB_EXECUTE:
        break;
    default:
        return;
    }
}


static void Put(void * m_user, unsigned flags, unsigned fsl, unsigned data)
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

static unsigned Get(void * m_user, unsigned flags, unsigned fsl)
{
    // see shim_asm.S
    switch (fsl) {
        case 0:
            // gettimeofday (seconds - and capture)
            return 0;
        case 1:
            // gettimeofday (microseconds when the seconds were captured)
            return 0;
        default:
            fprintf(stderr, "Cannot execute get instruction for FSL %d\n", fsl);
            exit(1);
    }
}

int main ( int argc , char ** argv )
{
    struct MB_System_Context_struct * sys ;
    MB_Context * mb ;
    const char * out;

    if (argc != 2) {
        printf("Usage: %s <elf binary>\n", argv[0]);
        return 1;
    }
    sys = MB_System_Init(Trace, 1, NULL);
    mb = MB_System_Get_MB_Context(sys);
    out = MB_Read_Elf(sys, argv[1]);
    if (out != NULL) {
        printf("ELF read error: %s\n", out);
        return 1;
    }
    mb->pc = mb->cur_pc = 0; // start at address 0
    mb->gpr[1] = strtol(argv[4], NULL, 0);
    mb->gpr[15] = 0;    // return to location 8
    mb->put_fn = Put;
    mb->get_fn = Get;

    while (1) {
        MB_Step(mb, 0);
    }
}

