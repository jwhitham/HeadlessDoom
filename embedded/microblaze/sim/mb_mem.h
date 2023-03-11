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
#ifndef MB_MEM_H
#define MB_MEM_H

#define PAGE_SIZE       0x10000
#define PAGE_SHIFT      16
#define NUM_PAGES       0x10000

#include <arpa/inet.h>
#define endian_swap(x) (htonl((x)))

#include "mb_core.h"

struct MB_System_Context_struct ;

struct MB_System_Context_struct * MB_System_Init ( MB_Trace_Fn trace_fn , 
                            unsigned latency , void * t_user ) ;
void MB_System_Delete ( struct MB_System_Context_struct * sc ) ;

MB_Context * MB_System_Get_MB_Context ( struct MB_System_Context_struct * sc ) ;

void MB_Map_IO ( struct MB_System_Context_struct * sc , unsigned page , 
                            MB_Load_Fn load_fn , MB_Store_Fn store_fn  ,
                            void * io_user ) ;
void MB_Unmap_IO ( struct MB_System_Context_struct * sc , unsigned page ) ;
void MB_Write ( struct MB_System_Context_struct * sc , unsigned address , 
                            const char * data , unsigned size ) ;
void MB_Read ( struct MB_System_Context_struct * sc , unsigned address , 
                            char * data , unsigned size ) ;
unsigned MB_Write_From_File ( struct MB_System_Context_struct * sc , 
                            unsigned address , int fd , unsigned size ) ;
unsigned MB_Read_To_File ( struct MB_System_Context_struct * sc , 
                            unsigned address , int fd , unsigned size ) ;


void Page_Store ( unsigned * page_data , unsigned address , 
                            unsigned data , unsigned size ) ;
unsigned Page_Load ( unsigned * page_data , 
                            unsigned address , unsigned size ) ;

#endif

