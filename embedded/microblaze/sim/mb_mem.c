/*
 * Scratchpad MMU - Simulated memory and IO devices
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
#include <stdlib.h>
#include <string.h>
#include <assert.h>
#include <unistd.h>

#include "mb_mem.h"
#include "mb_core.h"

#define EMPTY_MEMORY_IS     0
#define CHECK               0xc8f3680a


typedef struct MB_Memory_Page_struct {
    unsigned *          data ;
    MB_Load_Fn          io_load_fn ;
    MB_Store_Fn         io_store_fn ;
    void *              io_user ;
} MB_Memory_Page ;

typedef struct MB_System_Context_struct {
    unsigned            check ;
    MB_Memory_Page      memory_page [ NUM_PAGES ] ;
    MB_Context *        mb_context ;
    unsigned            latency ;
} MB_System_Context ;



static unsigned Load ( void * m_user , unsigned address , unsigned size ) ;
static unsigned IFetch ( void * m_user , unsigned address , unsigned size ) ;
static void Store ( void * m_user , unsigned address , 
                            unsigned data , unsigned size ) ;

MB_System_Context * MB_System_Init ( MB_Trace_Fn trace_fn , 
                             unsigned latency , void * t_user )
{
    MB_System_Context * sc = calloc ( 1 , sizeof ( MB_System_Context ) ) ;
    MB_Context * mc = calloc ( 1 , sizeof ( MB_Context ) ) ;
    
    assert ( mc != NULL ) ;
    assert ( sc != NULL ) ;

    sc -> check = CHECK ;
    sc -> mb_context = mc ;
    sc -> latency = latency ;

    mc -> m_user = sc ;
    mc -> ifetch_fn = IFetch ;
    mc -> load_fn = Load ;
    mc -> store_fn = Store ;
    mc -> trace_fn = trace_fn ;
    mc -> t_user = t_user ;
    MB_Reset ( mc ) ;
    return sc ;
}

MB_Context * MB_System_Get_MB_Context ( MB_System_Context * sc )
{
    return sc -> mb_context ;
}

void MB_System_Delete ( MB_System_Context * sc )
{
    unsigned i ;

    assert ( sc -> check == CHECK ) ;
    for ( i = 0 ; i < NUM_PAGES ; i ++ )
    {
        unsigned * d = sc -> memory_page [ i ] . data ;

        if ( d != NULL )
        {
            free ( d ) ;
        }
    }
    free ( sc -> mb_context ) ;
    free ( sc ) ;
}

static int Is_IO_Page ( MB_Memory_Page * p )
{
    return ( p -> io_load_fn != NULL ) ;
}

static MB_Memory_Page * Lookup ( MB_System_Context * sc , 
                unsigned page , int allocate )
{
    MB_Memory_Page * p ;

    assert ( page < NUM_PAGES ) ;
    p = & sc -> memory_page [ page ] ;
    if (( p -> data == NULL )
    && ( allocate )
    && ( ! Is_IO_Page ( p )))
    {
        p -> data = malloc ( PAGE_SIZE ) ;
        assert ( p -> data != NULL ) ;
        memset ( p -> data , (unsigned char) EMPTY_MEMORY_IS , PAGE_SIZE ) ;
    }
    return p ;
}

void Page_Store ( unsigned * page_data , unsigned address , 
                            unsigned data , unsigned size )
{
    unsigned within_page = address >> 2 ;
    unsigned within_word = ( address & 3 ) ;
    unsigned mask , modify , shift ; 

    if ( size == 4 )
    {
        page_data [ within_page ] = endian_swap ( data ) ;
        return ;
    } else if ( size == 2 )
    {
        mask = 0xffff ;
        shift = (( 2 - within_word ) * 8 ) ;
    } else {
        mask = 0xff ;
        shift = (( 3 - within_word ) * 8 ) ;
    }
    modify = endian_swap ( page_data [ within_page ] ) ;
    modify &= ~ ( mask << shift ) ;
    modify |= ( data & mask ) << shift ;
    page_data [ within_page ] = endian_swap ( modify ) ;
}

unsigned Page_Load ( unsigned * page_data , unsigned address , unsigned size )
{
    unsigned within_page = address >> 2 ;
    unsigned within_word = ( address & 3 ) ;
    unsigned data = endian_swap ( page_data [ within_page ] ) ;

    if ( size == 4 )
    {
        return data ;
    } else if ( size == 2 )
    {
        return ( data >> (( 2 - within_word ) * 8 )) & 0xffff ;
    } else {
        return ( data >> (( 3 - within_word ) * 8 )) & 0xff ;
    }
}

static unsigned IFetch ( void * m_user , unsigned address , unsigned size )
{
    MB_System_Context * sc = (MB_System_Context *) m_user ;

    sc -> mb_context -> clock_cycle -= sc -> latency ;
    return Load ( m_user , address , size ) ;
}

static unsigned Load ( void * m_user , unsigned address , unsigned size )
{
    MB_System_Context * sc = (MB_System_Context *) m_user ;
    unsigned page = address >> PAGE_SHIFT ;
    MB_Memory_Page * p ;

    assert ( sc -> check == CHECK ) ;
    p = Lookup ( sc , page , 0 ) ;
    if ( p -> data != NULL )
    {
        sc -> mb_context -> clock_cycle += sc -> latency ;
        return Page_Load ( p -> data , address & ( PAGE_SIZE - 1 ) , size ) ;
    } else if ( Is_IO_Page ( p ) )
    {
        return p -> io_load_fn ( p -> io_user , address , size ) ;
    } else {
        /* Return 0xff, 0xffff, etc. for invalid page */
        sc -> mb_context -> clock_cycle += sc -> latency ;
        return EMPTY_MEMORY_IS >> (( 4UL - size ) * 8UL ) ;
    }
}

static void Store ( void * m_user , unsigned address , 
                            unsigned data , unsigned size )
{
    MB_System_Context * sc = (MB_System_Context *) m_user ;
    unsigned page = address >> PAGE_SHIFT ;
    MB_Memory_Page * p ;

    assert ( sc -> check == CHECK ) ;
    p = Lookup ( sc , page , 1 ) ;
    if ( Is_IO_Page ( p ) )
    {
        p -> io_store_fn ( p -> io_user , address , data , size ) ;
    } else {
        assert ( p -> data != NULL ) ;
        sc -> mb_context -> clock_cycle += sc -> latency ;
        Page_Store ( p -> data , address & ( PAGE_SIZE - 1 ) , data , size ) ;
    }
}

void MB_Map_IO ( MB_System_Context * sc , unsigned page , 
                            MB_Load_Fn load_fn , MB_Store_Fn store_fn  ,
                            void * io_user )
{
    MB_Memory_Page * p ;

    assert ( sc -> check == CHECK ) ;
    p = Lookup ( sc , page , 0 ) ;
    if ( p -> data != NULL )
    {
        free ( p -> data ) ;
        p -> data = NULL ;
    }
    p -> io_load_fn = load_fn ;
    p -> io_store_fn = store_fn ;
    p -> io_user = io_user ;
}

void MB_Unmap_IO ( MB_System_Context * sc , unsigned page )
{
    MB_Memory_Page * p ;

    assert ( sc -> check == CHECK ) ;
    p = Lookup ( sc , page , 0 ) ;
    p -> io_load_fn = NULL ;
    p -> io_store_fn = NULL ;
    p -> io_user = NULL ;
}

static unsigned MB_Access ( MB_System_Context * sc , unsigned address , 
                char * data , unsigned size , int write_flag , int fd )
{
    unsigned total_bytes = 0 ;

    assert ( sc -> check == CHECK ) ;
    while ( size != 0 )
    {
        unsigned page = address >> PAGE_SHIFT ;
        unsigned within_page = ( address & ( PAGE_SIZE - 1 )) ;
        unsigned space = PAGE_SIZE - within_page ;
        MB_Memory_Page * p = Lookup ( sc , page , write_flag ) ;
        char * ptr = NULL ;

        
        if ( space > size ) space = size ;

        assert ( ! Is_IO_Page ( p ) ) ; /* No access to IO pages */

        if ( p -> data != NULL )
        {
            ptr = & (( (char *) p -> data ) [ within_page ] ) ;
        }

        if ( write_flag )
        {
            assert ( ptr != NULL ) ;
            if ( fd >= 0 )
            {
                ssize_t s = read ( fd , ptr , space ) ;
                if ( s != space )
                {
                    if ( s > 0 )
                    {
                        total_bytes += space ;
                    }
                    return total_bytes ;
                }
            } else {
                memcpy ( ptr , data , space ) ;
            }
        } else if ( ptr == NULL )
        {
            if ( fd >= 0 )
            {
                char blank [ BUFSIZ ] ;
                ssize_t s ;

                if ( space >= BUFSIZ ) space = BUFSIZ ;
                memset ( blank , (unsigned char) EMPTY_MEMORY_IS , space ) ;
                s = write ( fd , blank , space ) ;
                assert ( s == space ) ;
            } else {
                memset ( data , (unsigned char) EMPTY_MEMORY_IS , space ) ;
            }
        } else {
            if ( fd >= 0 )
            {
                ssize_t s = write ( fd , ptr , space ) ;
                assert ( s == space ) ;
            } else {
                memcpy ( data , ptr , space ) ;
            }
        }
        size -= space ;
        data += space ;
        address += space ;
        total_bytes += space ;
    }
    return total_bytes ;
}

void MB_Write ( MB_System_Context * sc , unsigned address , 
                            const char * data , unsigned size )
{
    MB_Access ( sc , address , (char *) data , size , 1 , -1 ) ;
}


void MB_Read ( MB_System_Context * sc , unsigned address , 
                            char * data , unsigned size ) 
{
    MB_Access ( sc , address , data , size , 0 , -1 ) ;
}

unsigned MB_Write_From_File ( MB_System_Context * sc , unsigned address ,
                            int fd , unsigned size )
{
    return MB_Access ( sc , address , NULL , size , 1 , fd ) ;
}

unsigned MB_Read_To_File ( MB_System_Context * sc , unsigned address ,
                            int fd , unsigned size )
{
    return MB_Access ( sc , address , NULL , size , 0 , fd ) ;
}






