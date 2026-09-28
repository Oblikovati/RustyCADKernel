// Copyright (c) 1995-1999 Matra Datavision
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

//! The Precision.hxx constants CSLib reads.
//!
//! TKernel's Precision package has no Rust port yet; these mirror its constexpr
//! definitions (same expressions, so the same bits) and should move to that port.

/// Precision::Confusion().
pub(crate) const CONFUSION: f64 = 1.0e-7;
/// Precision::PConfusion(), computed as `Confusion() * 0.01` like the native constexpr.
pub(crate) const P_CONFUSION: f64 = CONFUSION * 0.01;
/// Precision::Infinite().
pub(crate) const INFINITE: f64 = 2.0e100;

/// Precision::IsInfinite(R); `is_infinite(1e100)`.
pub(crate) fn is_infinite(value: f64) -> bool {
    value.abs() >= 0.5 * INFINITE
}

#[cfg(test)]
mod tests {
    use super::{P_CONFUSION, is_infinite};

    #[test]
    fn precision_constants_match_native_expressions() {
        assert_eq!(P_CONFUSION.to_bits(), (1.0e-7_f64 * 0.01).to_bits());
        assert!(is_infinite(1.0e100) && is_infinite(-1.0e100));
        assert!(!is_infinite(9.9e99) && !is_infinite(f64::NAN));
    }
}
