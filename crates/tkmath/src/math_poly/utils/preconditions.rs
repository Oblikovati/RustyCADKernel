// Copyright (c) 2026 OPEN CASCADE SAS
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

//! Preconditions of the public `MathPoly::Utils` helpers. Native code takes raw pointers
//! and fixed `std::array` buffers and reads or writes past them when these do not hold
//! (undefined behaviour); the port raises Standard_OutOfRange naming the offending sizes
//! and the expected shape instead.
use crate::math_poly::MAX_POLY_DEGREE;
use rustycad_tkernel::standard::Failure;

/// Refuses a degree outside `minimum..=MAX_POLY_DEGREE`, the range the fixed
/// `THE_MAX_POLY_DEGREE + 1` arrays hold; returns it as an index.
/// `fixed_degree("MathPoly::Utils::Evaluate", 2, 0)? == 2`.
pub(super) fn fixed_degree(member: &str, degree: i32, minimum: i32) -> Result<usize, Failure> {
    if degree < minimum || degree > MAX_POLY_DEGREE as i32 {
        return Err(Failure::out_of_range(&format!(
            "{member}: degree {degree}; expected {minimum} <= degree <= {MAX_POLY_DEGREE}"
        )));
    }
    Ok(degree as usize)
}

/// Refuses coefficients that do not hold `c[0..=degree]` for a degree of at least
/// `minimum` (without upper limit: the caller's pointer is all native code reads).
/// `slice_degree("MathPoly::Utils::EvaluateComplex", 3, 2, 0)? == 2`.
pub(super) fn slice_degree(
    member: &str,
    len: usize,
    degree: i32,
    minimum: i32,
) -> Result<usize, Failure> {
    if degree < minimum || len <= degree as usize {
        return Err(Failure::out_of_range(&format!(
            "{member}: degree {degree} with {len} coefficients; expected degree >= {minimum} \
             and at least degree + 1 coefficients"
        )));
    }
    Ok(degree as usize)
}

/// Both limits: a caller slice copied into a fixed `THE_MAX_POLY_DEGREE + 1` array.
/// `bounded_degree("MathPoly::Utils::HasMultiplicity", 3, 2, 0)? == 2`.
pub(super) fn bounded_degree(
    member: &str,
    len: usize,
    degree: i32,
    minimum: i32,
) -> Result<usize, Failure> {
    fixed_degree(member, degree, minimum)?;
    slice_degree(member, len, degree, minimum)
}

/// Refuses a count that already fills a `capacity`-slot buffer about to take one more
/// item. `free_slot("MathPoly::Utils::AppendUnique", 3, 4, "roots")?`.
pub(super) fn free_slot(
    member: &str,
    count: usize,
    capacity: usize,
    items: &str,
) -> Result<(), Failure> {
    if count < capacity {
        return Ok(());
    }
    Err(Failure::out_of_range(&format!(
        "{member}: {items} {} of a {capacity}-slot buffer; expected at most {capacity} {items}",
        count + 1
    )))
}

/// Refuses a stored count above the buffer holding it.
/// `stored_count("MathPoly::Utils::ConsolidateRealRoots", 3, 20, "real roots")?`.
pub(super) fn stored_count(
    member: &str,
    count: usize,
    capacity: usize,
    items: &str,
) -> Result<(), Failure> {
    if count <= capacity {
        return Ok(());
    }
    Err(Failure::out_of_range(&format!(
        "{member}: {count} {items} in a {capacity}-slot buffer; expected at most {capacity}"
    )))
}
