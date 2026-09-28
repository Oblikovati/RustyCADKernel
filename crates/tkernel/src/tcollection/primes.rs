// Created on: 1992-10-13
// Created by: Ramin BARRETO
// Copyright (c) 1992-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.

//! The deprecated `TCollection` class: `NextPrimeForMap` over `int`.
use crate::ncollection::next_map_prime;

/// `TCollection::NextPrimeForMap`: `NCollection_Primes::NextPrimeForMap` called with the
/// `int` converted to `size_t` and the result narrowed back to `int`; `next_prime_for_map(100) == 101`.
///
/// The conversions are part of the result: -1 becomes `SIZE_MAX`, whose `+ 1` wraps to 0,
/// so it maps to 101; other negative values and values above the last table prime come
/// back as `value + 1`, which for `i32::MAX` wraps to `i32::MIN`.
pub fn next_prime_for_map(value: i32) -> i32 {
    let size = value as isize as usize;
    // `NCollection_Primes` looks for the first table prime >= size + 1 in wrapping size_t
    // arithmetic; `next_map_prime(n)` answers "first prime > n", i.e. ">= n + 1".
    let wanted = size.wrapping_add(1);
    next_map_prime(wanted.saturating_sub(1)) as i32
}
