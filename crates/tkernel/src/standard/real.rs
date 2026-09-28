// Copyright (c) 1998-1999 Matra Datavision
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
// Rust translation of FoundationClasses/TKernel/Standard/Standard_Real.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Standard_Real scalar helpers whose results feed tolerance decisions.

/// Returns the gap to the next representable value away from zero, as `Epsilon(x)` does.
///
/// The native expression moves toward `RealLast()` for non-negative inputs and toward
/// `RealFirst()` otherwise, so `f64::MAX` yields zero and infinities yield `-inf`.
/// Example: `assert_eq!(epsilon(1.0), f64::EPSILON)`.
pub fn epsilon(value: f64) -> f64 {
    let magnitude: f64 = value.abs();
    if magnitude == f64::MAX {
        return 0.0;
    }
    if magnitude == f64::INFINITY {
        // nextafter(inf, DBL_MAX) is DBL_MAX, and DBL_MAX - inf is -inf in both signs.
        return f64::NEG_INFINITY;
    }
    magnitude.next_up() - magnitude
}
