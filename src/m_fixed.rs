// Emacs style mode select   -*- C++ -*- 
//-----------------------------------------------------------------------------
//
// $Id:$
//
// Copyright (C) 1993-1996 by id Software, Inc.
//
// This source is available for distribution and/or modification
// only under the terms of the DOOM Source Code License as
// published by id Software. All rights reserved.
//
// The source is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// FITNESS FOR A PARTICULAR PURPOSE. See the DOOM Source Code License
// for more details.
//
// $Log:$
//
// DESCRIPTION:
//	Fixed point implementation.
//
//-----------------------------------------------------------------------------


use crate::defs::*;

#[no_mangle]
pub extern "C" fn FixedMul(a: fixed_t, b: fixed_t) -> fixed_t {
    return (((a as i64) * (b as i64)) >> FRACBITS) as fixed_t;
}


//
// FixedDiv, C version.
//
#[no_mangle]
pub extern "C" fn FixedDiv(a: fixed_t, b: fixed_t) -> fixed_t {
    if (fixed_t::abs(a) >> 14) >= fixed_t::abs(b) {
        return if (a ^ b) < 0 { MININT } else { MAXINT };
    }
    return FixedDiv2 (a,b);
}


fn FixedDiv2(a: fixed_t, b: fixed_t) -> fixed_t {
//#if 0
    //long long c;
    //c = ((long long)a<<16) / ((long long)b);
    //return (fixed_t) c;
//#endif

    if b == 0 {
        panic!("FixedDiv: divide by zero (exactly)");
    }

    let c: i64 = ((a as i64) << FRACBITS) / (b as i64);

    if (c >= 2147483648) || (c < -2147483648) {
        panic!("FixedDiv: divide by zero (approximately)");
    }
    return c as fixed_t;
}
