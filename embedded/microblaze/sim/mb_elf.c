/*
 * Scratchpad MMU - ELF reader
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
#include <sys/types.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <unistd.h>
#include <stdint.h>


#ifndef off64_t
#define off64_t int64_t
#endif

#include <libelf.h>
#include <gelf.h>

#include "mb_core.h"
#include "mb_mem.h"


static const char * __Read_Elf ( 
            struct MB_System_Context_struct * sc , Elf * e , int fd ) ;

const char * MB_Read_Elf ( 
            struct MB_System_Context_struct * sc , const char * fname )
{
    Elf *           e ;
    const char *    out = NULL ;
    int             fd ;

    if ( elf_version ( EV_CURRENT ) == EV_NONE )
    {
        return "libelf version check failed." ;
    }
    fd = open ( fname , O_RDONLY ) ;
    if ( fd < 0 )
    {
        return "MB_Read_Elf open() failed." ;
    }

    e = elf_begin ( fd , ELF_C_READ , NULL ) ;
    if ( e == NULL )
    {
        close ( fd ) ;
        return "elf_begin() failed." ;
    }

    out = __Read_Elf ( sc , e , fd ) ;
    elf_end ( e ) ;
    close ( fd ) ;
    return out ;
}

static const char * __Read_Elf ( 
            struct MB_System_Context_struct * sc , Elf * e , int fd )
{
    Elf_Kind        ek ;
    GElf_Ehdr       ehdr ;
    GElf_Phdr       phdr ;
    size_t          i , n = 0 , loaded = 0 ;
    off_t           cur_pos ;

    ek = elf_kind ( e ) ;
    if ( ek != ELF_K_ELF )
    {
        return "unrecognised kind of ELF object." ;
    }
    if ( gelf_getehdr ( e , & ehdr ) == NULL )
    {
        return "unable to read ELF header." ;
    }
    if ( gelf_getclass ( e ) != ELFCLASS32 )
    {
        return "ELF object is not 32-bit." ;
    }
    if ( ehdr . e_type != ET_EXEC )
    {
        return "ELF type is not a plain executable." ;
    }
    if ( ehdr . e_machine != 0xbaab )
    {
        return "Machine for this ELF is not Microblaze." ;
    }
#ifdef elf_getphnum
    /* This function isn't in every libelf - it's in the Slackware
     * version and the one bundled with M5, but not the Red Hat one */
    if ( elf_getphnum ( e , & n ) == 0 )
    {
        return "elf_getphnum() failed: no program header?" ;
    }
#else
    n = ehdr . e_phnum ;
#endif
    for ( i = 0 ; i < n ; i ++ )
    {
        if ( gelf_getphdr ( e , i , & phdr ) != & phdr )
        {
            return "gelf_getphdr() failed." ;
        }
        if ( phdr . p_type != PT_LOAD )
        {
            continue ;
        }
        cur_pos = lseek ( fd , 0 , SEEK_CUR ) ;
        if ( lseek ( fd , phdr . p_offset , SEEK_SET ) !=
                        phdr . p_offset )
        {
            return "lseek() failed." ;
        }

        MB_Write_From_File ( sc , (uint32_t) phdr . p_paddr , 
                        fd , (uint32_t) phdr . p_filesz ) ;
        lseek ( fd , cur_pos , SEEK_SET ) ;
        loaded ++ ;
    }
    if ( loaded == 0 )
    {
        return "ELF contains no loadable sections?" ;
    }
    MB_Jump ( MB_System_Get_MB_Context ( sc ) , 
                    (uint32_t) ehdr . e_entry ) ;

    /* success */
    return NULL ;
}



