/*
 * Scratchpad MMU - Microblaze(tm)-compatible simulator
 * Copyright (C) Jack Whitham 2009
 * http://www.jwhitham.org.uk/c/smmu.html
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Lesser General Public
 * License as published by the Free Software Foundation
 * (version 2.1 of the License only).
 * 
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Lesser General Public License for more details.
 * 
 * You should have received a copy of the GNU Lesser General Public
 * License along with this library; if not, write to the Free Software
 * Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA
 */

#ifndef MB_CORE_H
#define MB_CORE_H

#define NS 255

typedef enum {
        MB_RESET , MB_INTERRUPT , MB_SPR_READ , MB_SPR_WRITE , 
        MB_CALL , MB_RETURN , MB_ILLEGAL_INST ,
        MB_DIV_BY_ZERO , MB_EXECUTE , MB_SIM_CMD ,
        MB_ALIGN , MB_MISS_LOAD , MB_MISS_STORE , MB_OPEN , MB_CLOSE ,
        MB_STORE , MB_LOAD , MB_SET_D , MB_HIT_LOAD , MB_HIT_STORE ,
        MB_CACHE_MISS 
} MB_Trace_Name ;

struct MB_Context_struct ;

typedef unsigned (* MB_Load_Fn) 
        ( void * m_user , unsigned address , unsigned size ) ;
typedef void (* MB_Store_Fn) 
        ( void * m_user , unsigned address , unsigned data , unsigned size ) ;
typedef void (* MB_Trace_Fn) 
        ( void * t_user , struct MB_Context_struct * mc ,
          MB_Trace_Name trace_name , const void * param ) ;
typedef void (* MB_Put_Fn) 
        ( void * m_user , unsigned flags, unsigned fsl, unsigned data );
typedef unsigned (* MB_Get_Fn) 
        ( void * m_user , unsigned flags, unsigned fsl );
typedef void (* MB_WIC_Fn) 
        ( void * m_user , unsigned address );

typedef struct MB_Context_struct
{
    unsigned        gpr [ 32 ] ;
    unsigned        pc , msr , next_iword , cur_iword;
    unsigned        immediate , cur_pc , next_pc ;
    unsigned        bubble_reg_1, bubble_reg_2, bubble_time;
    int             atomic , immediate_available , delay_enable_ints ;
    void *          m_user ;
    void *          t_user ;
    unsigned        clock_cycle ;
    unsigned        instruction_count ;

    MB_Load_Fn      ifetch_fn ;
    MB_Load_Fn      load_fn ;
    MB_Store_Fn     store_fn ;
    MB_Trace_Fn     trace_fn ;
    MB_Put_Fn       put_fn;
    MB_Get_Fn       get_fn;
    MB_WIC_Fn       wic_fn;
    MB_WIC_Fn       wdc_fn;
} MB_Context ;

#define MSR_BIP         0x8 
#define MSR_EIP         0x200 
#define MSR_EE          0x100 
#define MSR_IE          0x2 
#define MSR_C           0x80000004 
#define MB_NOP          0x80000000 
#define MB_FLUSH_NOP    0x800007ff 
#define MSR_DCACHE_ENABLE 0x80
#define MSR_ICACHE_ENABLE 0x20

#define MB_FSL_C        0x1
#define MB_FSL_N        0x2

void MB_Reset ( MB_Context * c ) ;
void MB_Step ( MB_Context * c , int interrupt_flag ) ;
void MB_Jump ( MB_Context * c , unsigned target_pc ) ;

static inline unsigned Get_rD ( MB_Context * c ) 
{
    return ( c -> cur_iword >> 21 ) & 0x1f ;
}

static inline unsigned Get_D ( MB_Context * c ) 
{ 
    return c -> gpr [ Get_rD ( c ) ] ;
}

static inline unsigned Get_rA ( MB_Context * c ) 
{
    return ( c -> cur_iword >> 16 ) & 0x1f ;
}

static inline unsigned Get_rB ( MB_Context * c ) 
{
    unsigned iword = c -> cur_iword ;
    unsigned rB = ( iword >> 11 ) & 0x1f ;

    if (( iword & ( 1 << 29 ) ) != 0 )
    {
        rB = 0 ; /* immediate form */
    }
    return rB ;
}

#define IP(x) ( (void *) ( (uintptr_t) (x) ))

#endif

