// Copyright (c) 1993-1999 Matra Datavision
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
// Rust translation of ModelingData/TKG2d/Geom2d (C++ standard library semantics used by the Geom2d sources) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `std::min`/`std::max` on doubles with their exact C++ semantics.
//!
//! `f64::min`/`f64::max` may return either zero for `(-0.0, 0.0)` and skip NaN;
//! the C++ templates return the first argument unless the comparison says
//! otherwise, which decides the bits of signed zeros and NaN results.

/// `std::max(a, b)`: `a < b ? b : a`; `std_max(-0.0, 0.0).is_sign_negative()`.
pub fn std_max(a: f64, b: f64) -> f64 {
    if a < b { b } else { a }
}

/// `std::min(a, b)`: `b < a ? b : a`; `std_min(0.0, -0.0).is_sign_positive()`.
pub fn std_min(a: f64, b: f64) -> f64 {
    if b < a { b } else { a }
}
