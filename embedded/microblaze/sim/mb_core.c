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

#include <stdio.h>
#include <string.h>
#include <assert.h>
#include <stdint.h>
#include "mb_core.h"



/* Microblaze(tm) compatible virtual machine.
 * NOT OFFICIAL XILINX SOFTWARE - USE ENTIRELY AT YOUR OWN RISK
 * Supports barrel shift, multiply, divide, interrupts.
 * Does not support floating point, fast simplex link, 
 * pattern instructions or internal exceptions.
 */

static const char * fsl_type[] = { "", "c", "n", "nc" };


void MB_Jump ( MB_Context * c , uint32_t target_pc )
{
    c -> pc = target_pc ;
    c -> next_iword = MB_FLUSH_NOP ;
    c -> cur_iword = MB_FLUSH_NOP ;
    c -> immediate = 0 ;
    c -> atomic = 0 ;
    c -> immediate_available = 0 ;
}

void MB_Reset ( MB_Context * c )
{
    uint32_t i ;

    for ( i = 0 ; i < 32 ; i ++ )
    {
        c -> gpr [ i ] = 0 ;
    }
    c -> cur_pc = c -> next_pc = 0 ;
    c -> pc = 0 ;
    c -> msr = 0 ;
    c -> trace_fn ( c -> t_user , c , MB_RESET , 0 ) ;
    c -> clock_cycle = 0 ;
    c -> instruction_count = 0 ;
    MB_Jump ( c , 0 ) ;
}

static void Set_D ( MB_Context * c , uint32_t x ) 
{ 
    uint32_t r = ( c -> cur_iword >> 21 ) & 0x1f ;
    if ( r != 0 )
    {
        c -> gpr [ r ] = x ;
        c -> trace_fn ( c -> t_user , c , MB_SET_D , IP(x) ) ;
    }
}
    
/* Opcodes beginning with command 0x24.
 * These are not very common unless the code has been built
 * without barrel shifter support. */
static const char * Annex_Sign_Extend_Short_Shift (
            MB_Context * c , uint32_t a )
{
    uint32_t    x , out = 0 ;
    const char * name = "?" ;

    switch ( c -> cur_iword & 0x7f )
    {
        case 0x60 : /* sext8 (pg 136) */
            if (( a & 0x80 ) != 0 )
            {
                a |= 0xffffff00 ;
            } else {
                a &= 0xff ;
            }
            out = a ;
            name = "sext8" ;
            break ;
        case 0x61 : /* sext16 (pg 135) */
            if (( a & 0x8000 ) != 0 )
            {
                a |= 0xffff0000 ;
            } else {
                a &= 0xffff ;
            }
            out = a ;
            name = "sext16" ;
            break ;
        case 0x01 : /* sra (pg 139) */
            if (( a & 1 ) != 0 )    /* get carry flag */
            {
                c -> msr |= MSR_C ;
            } else {
                c -> msr &= ~MSR_C ;
            }
            out = (uint32_t) ( (int32_t) a >> 1 ) ;
            name = "sra" ;
            break ;
        case 0x21 : /* src (pg 140) */
            /* use old carry flag */
            x = ( c -> msr & MSR_C ) ;
            if (( a & 1 ) != 0 )   /* get new carry flag */
            {
                c -> msr |= MSR_C ;
            } else {
                c -> msr &= ~MSR_C ;
            }
            a = a >> 1 ;
            a = a & 0x7fffffff ;
            if ( x != 0 ) a |= 1 << 31 ;
            out = a ;
            name = "src" ;
            break ;
        case 0x41 : /* srl (pg 141) */
            if (( a & 1 ) != 0 )    /* get carry flag */
            {
                c -> msr |= MSR_C ;
            } else {
                c -> msr &= ~MSR_C ;
            }
            out = ( a >> 1 ) & 0x7fffffff ;
            name = "srl" ;
            break ;
        case 0x68 : /* wic */
            if (c->wic_fn) {
                c->wic_fn(c, a);
            }
            return "wic" ;
        case 0x64 : /* wdc */
            if (c->wdc_fn) {
                c->wdc_fn(c, a);
            }
            return "wdc" ;
        default :
            return "?" ; /* unknown op */
    }

    Set_D ( c , out ) ;
    return name ;
}

    /* Opcodes for accessing the SPRs */
static const char * Annex_SPR_Ops ( MB_Context * c ,
            uint32_t a )
{
    uint32_t out = c -> msr ;
    const char * name = "?" ;

    switch ( c -> cur_iword & 0xc000 )
    {
        case 0 : /* msrset or msrclr */
            if ( 0 != ( c -> cur_iword & 0x10000 )) /* msrclr */
            {
                c -> msr &= ~ ( c -> cur_iword & 0x3fff ) ;
                name = "msrclr" ;
            } else {
                c -> msr |= ( c -> cur_iword & 0x3fff ) ;
                name = "msrset" ;
            }
            break ;
        case 0x8000 : /* mfs */
            switch ( c -> cur_iword & 0x3fff )
            {
                case 1 :    /* MSR selected */
                    out = c -> msr ;
                    break ;
                case 0x2000:    /* PVR0 */
                    out = 0x73000b00; /* version 7.10.d with caches */
                    break;
                default :
                    /* Unknown SPR read */
                    out = 0 ;
                    c -> trace_fn ( c -> t_user , c , MB_SPR_READ , 0 ) ;
                    break ;
            }
            name = "mfs" ;
            break ;
        case 0xc000 : /* mts */
            switch ( c -> cur_iword & 0x3fff )
            {
                case 1 :    /* MSR selected */
                    c -> msr = a ;
                    break ;
                default :
                    /* Unknown SPR write */
                    c -> trace_fn ( c -> t_user , c , 
                                    MB_SPR_WRITE , IP ( a ) ) ;
                    break ;
            }
            name = "mts" ;
            return name ; /* not a read operation */
        default :
            return name ; /* not a known operation */
    }
    Set_D ( c , out ) ;
    return name ;
}


void MB_Step ( MB_Context * c , int interrupt_flag )
{
    /* Instruction pipeline */
    uint32_t    iword , topcode , ea ;
    int         imm_inst = 0 ;
    int         div_zero = 0 ;
    int         nodelay , valid = 1 ;
    uint32_t    a , b , temp , carry_in , rA , rB , rD ;
    int         condition , sub_call = 0 ;
    const char * temp_name = "?" ;
    char        name [ NS + 1 ] ;
    const int   pcoffset = -8 ;
    uint32_t    latency = 1 ;
    int         bubble = 0;

    do {
        iword = c -> cur_iword = c -> next_iword ;
        c -> cur_pc = c -> next_pc ;

        if (c->next_iword == MB_FLUSH_NOP) {
            c -> next_iword = c -> ifetch_fn ( c -> m_user , c -> pc , 4 ) ;
            (void) c->ifetch_fn(c->m_user, c->pc + 4, 4);
            //printf("xfetch %x -> %x\n", c->pc, c->next_iword);
            c->bubble_time = 0;
        } else {
            c->next_iword = ~0;
        }

        c -> next_pc = c -> pc ;
        c -> pc += 4 ;

        name [ NS ] = '\0' ;

        /* Interrupt handler */
        if ( c -> atomic )
        {
            c -> atomic = 0 ;
        } else if (( interrupt_flag ) /* interrupt! */
        && (( c -> msr & ( MSR_IE | MSR_BIP | MSR_EIP )) == MSR_IE ))
        {
            /* IE high, BIP low, EIP low */
            /* Prevent further interrupts until reenabled by code */
            c -> msr &= ~ MSR_IE ;
            /* PC copied to "return PC" GPR */
            c -> pc += pcoffset ;
            c -> gpr [ 14 ] = c -> pc ; 
            /* Pipeline flush */
            iword = c -> cur_iword = c -> next_iword = MB_FLUSH_NOP ;
            /* Go to vector code */
            c -> pc = 0x10 ;
            c -> trace_fn ( c -> t_user , c , MB_INTERRUPT , 0 ) ;
        } else if ( c -> delay_enable_ints > 0 )
        {
            c -> delay_enable_ints -- ;
            if ( c -> delay_enable_ints == 0 )
            {
                c -> msr |= MSR_IE ; 
            }
        }
    } while ( iword == MB_FLUSH_NOP ) ;

    c -> next_iword = ~0;

    /* Decoding of command */
    topcode = (( iword >> 26 ) & 0x3f ) ;
    rD = ( iword >> 21 ) & 0x1f ;
    rA = ( iword >> 16 ) & 0x1f ;
    rB = ( iword >> 11 ) & 0x1f ;

    a = c -> gpr [ rA ] ;
    if (( iword & ( 1 << 29 ) ) != 0 ) {
        /* Get B value from immediate */
        if ( c -> immediate_available ) {
            b = (( c -> immediate << 16 ) | 
                    ( iword & 0xffff )) ;
        } else if (( iword & 0x8000 ) != 0 ) {
            b = iword | (int32_t) ( 0xffff << 16 ) ;
        } else {
            b = iword & 0xffff ;
        }
        rB = 0;
    } else {
        /* Get B value from register */
        b = c -> gpr [ rB ] ;
    }
    switch ( topcode ) {
        case 0x27 :     
        case 0x2f :
            /* beq etc.: Conditional branch 
             * (page 79 of ISA ref, onwards) */
            switch (( iword >> 21 ) & 7 )
            {
                case 0 :    condition = ( (int32_t) a == 0 ) ; break ;
                case 1 :    condition = ( (int32_t) a != 0 ) ; break ;
                case 2 :    condition = ( (int32_t) a < 0 ) ; break ;
                case 3 :    condition = ( (int32_t) a <= 0 ) ; break ;
                case 4 :    condition = ( (int32_t) a > 0 ) ; break ;
                case 5 :    condition = ( (int32_t) a >= 0 ) ; break ;
                default :   condition = 0 ; break ;
            }

            temp = c -> pc + b + pcoffset ;
            {
                static const char * b_names [ 8 ] = 
                        { "beq" , "bne" , "blt" , "ble" ,
                        "bgt" , "bge" , "?" , "?" } ;
                snprintf ( name , NS , "%s: r%u: to 0x%x (cond:%s taken)\n" ,
                        b_names [ ( iword >> 21 ) & 7 ] ,
                        rA , temp , condition ? "" : " not" ) ;
            }
                
            if ( condition ) /* conditional branch taken */
            {   
                c -> pc = temp ;

                if ( 0 == ( iword & ( 1 << 25 ) ))
                { /* delay bit not set */
                    c -> next_iword = MB_FLUSH_NOP ;
                    c -> next_pc = c -> pc ;
                    latency += 2;
                } else {
                    latency += 1;
                }
                c -> atomic = 1 ;
            }
            break ;
        case 0x26 :     
        case 0x2e :
            /* brai, etc.:
             * Unconditional branch (page 91 of ISA ref manual) */
            temp_name = "bra" ;

            nodelay = /* delay bit not set */
                        ( 0 == ( iword & ( 1 << 20 ) )) ; 

            if ( 0 != ( iword & ( 1 << 18 ) )) /* link bit set */
            {
                temp_name = "bral" ;
                Set_D ( c , c -> pc + pcoffset ) ;
                /* *could* be a brk operation (page 95) */
                if (( iword & 0x1f0000 ) == 0x0c0000 ) /* brk */
                {
                    temp_name = "brk" ;
                    c -> msr |= MSR_BIP ;
                } else {
                    sub_call = 1 ;
                }
            }
            if ( 0 != ( iword & ( 1 << 19 ) )) /* absolute bit set */
            {
                c -> pc = b ;
            } else {
                c -> pc += b + pcoffset ;
            }
            if ( nodelay )
            {
                c -> next_iword = MB_FLUSH_NOP ;
                c -> next_pc = c -> pc ;
                latency += 2 ;
            } else {
                latency += 1;
            }
            if ( sub_call )
            {
                c -> trace_fn ( c -> t_user , c , MB_CALL , IP ( c -> pc )) ;
            }
            c -> atomic = 1 ; /* no interrupt in delay slot */
            snprintf ( name , NS , "%s: to 0x%x\n" ,
                    temp_name , c -> pc ) ;
            rA = 0;
            break ;
        case 0x2c :
            /* imm: immediate load instruction */
            snprintf ( name , NS , "imm: 0x%x\n" , iword & 0xffff ) ;
            imm_inst = 1 ;
            c -> immediate = iword ;
            rA = rB = 0;
            break ;
        case 0x12 :
        case 0x1a :
            /* idiv: division instruction (pg 107) */
            bubble = 2;
            if (0 != (iword & 0x2)) { /* unsigned division */
                temp_name = "idivu";
                if (a == 0) {
                    div_zero = 1;
                    a ++ ;
                } else {
                    latency += 32 + 1;
                }
                temp = b / a ;
            } else {
                temp_name = "idiv";
                if (a == 0) {
                    div_zero = 1;
                    a ++ ;
                } else {
                    latency += 32 + 1;
                }
                temp = (uint32_t) ( (int32_t) b / (int32_t) a ) ;
            }
            snprintf ( name , NS , "%s: r%u = r%u / r%u = 0x%x\n" , 
                        temp_name , rD , rA , rB , temp ) ;
            Set_D ( c , temp ) ;
            break ;
        case 0x1b :
            /* put and get (FSL interface) */
            if ((iword & 0x1ff8) != 0) {
                valid = 0;
            } else if ((rD == 0) && ((iword >> 15) & 1) && c->put_fn) {
                latency++;
                snprintf ( name , NS , "%sput r%d, rfsl%d\n" , 
                        fsl_type[(iword >> 13) & 3], rA, iword & 7) ;
                c->put_fn(c->m_user, (iword >> 13) & 3, iword & 7, a);
            } else if ((rA == 0) && !((iword >> 15) & 1) && c->get_fn) {
                latency++;
                snprintf ( name , NS , "%sget r%d, rfsl%d\n" ,
                        fsl_type[(iword >> 13) & 3], rD, iword & 7) ;
                Set_D(c, c->get_fn(c->m_user, (iword >> 13) & 3, iword & 7));
            } else {
                valid = 0;
            }
            rB = 0;
            break;
        case 0x24 : 
            /* Sign extend and 1-bit shifts, also 
             * cache writers wdc and wic. */
            temp_name = Annex_Sign_Extend_Short_Shift ( c , a ) ;
            snprintf ( name , NS , "%s: r%u = %s r%u = 0x%x\n" , 
                    temp_name , rD , temp_name , rA , Get_D ( c ) ) ;
            rB = 0;
            break ;
        case 0x2d :
            /* rtsd etc: Return operations */
            c -> pc = a + b ;
            latency += 1 ;
            switch (( iword >> 21 ) & 0x1f )
            {
                case 0x12 : 
                    c -> msr &= ~MSR_BIP ; 
                    temp_name = "rtbd" ;
                    break ;
                case 0x11 : 
                    c -> delay_enable_ints = 1;
                    temp_name = "rtid" ;
                    break ;
                case 0x14 : 
                    c -> msr &= ~ MSR_EIP ;
                    c -> msr |= MSR_EE ;
                    temp_name = "rted" ;
                    break ;
                default :
                    if (( rA == 15 )
                    && ( b == 8 ))
                    {
                        /* It's being used as a RETURN. */
                        c -> trace_fn ( c -> t_user , c , 
                                    MB_RETURN , IP ( c -> pc )) ;
                    }
                    temp_name = "rtsd" ;
                    break ;
            }
            c -> atomic = 1 ; /* no interrupt in delay slot */
            snprintf ( name , NS , "%s: to 0x%x\n" , 
                    temp_name , c -> pc ) ;
            break ;
        case 0x25 :
            /* mfs, mts, msrset, msrclr: Special purpose register
             * access commands. */
            temp_name = Annex_SPR_Ops ( c , a ) ;
            snprintf ( name , NS , "%s\n" , temp_name ) ;
            rB = 0;
            break ;


#include "mb_autogen.c"

        default :
            rA = rB = 0;
            c -> trace_fn ( c -> t_user , c , 
                                MB_ILLEGAL_INST , IP ( c -> pc )) ;
            valid = 0 ;
            break ;
    }
    if (div_zero) {
        c -> trace_fn ( c -> t_user , c ,
                                MB_DIV_BY_ZERO , IP ( c -> pc )) ;
        valid = 0 ;
    }

    if (c->bubble_time) {
        if ((c->bubble_reg_2 == rB) || (c->bubble_reg_2 == rA)) {
            latency += c->bubble_time;
            c->bubble_time = 0;
        } else {
            c->bubble_time--;
        }
    }

    if (imm_inst) {
        c -> immediate_available = c -> atomic = 1 ;
    } else {
        c -> immediate_available = 0 ;
    }
    c -> clock_cycle += latency ;
    c -> instruction_count ++ ;
    if (valid) {
        c -> trace_fn ( c -> t_user , c , MB_EXECUTE , name ) ;
    }

    if (bubble && rD) {
        /* Destination register creates a pipeline bubble */
        c->bubble_reg_2 = rD;
        c->bubble_time = bubble;
    }
    if (c->next_iword == (~0)) {
        c->next_iword = c->ifetch_fn(c->m_user, c->next_pc, 4);
        (void) c->ifetch_fn(c->m_user, c->next_pc + 4, 4);
    }
}

