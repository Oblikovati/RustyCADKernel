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

//! gp_Dir's inline normalized products in their checked and `No_Exception` builds.
//!
//! The same inline gp code raises Standard_ConstructionError on a zero result in most
//! translation units but divides by zero silently where `No_Exception` is defined (ElSLib).
//! Frame operations are written once, generic over this choice.
use super::super::{Direction3, Failure, cross};
use std::convert::Infallible;

/// Normalized cross products with a build-specific zero-norm policy.
pub(crate) trait DirectionProducts {
    /// Failure type: [`Failure`] when checked, [`Infallible`] when compiled out.
    type Error;
    /// `left ^ right` normalized; `P::crossed(x,y)?`.
    fn crossed(left: Direction3, right: Direction3) -> Result<Direction3, Self::Error>;
    /// `base ^ (first ^ second)` normalized; `P::cross_crossed(n,x,n)?`.
    fn cross_crossed(
        base: Direction3,
        first: Direction3,
        second: Direction3,
    ) -> Result<Direction3, Self::Error>;
}
/// gp_Dir products that raise on a zero result.
pub(crate) struct CheckedProducts;
/// gp_Dir products compiled with `No_Exception`: a zero result becomes NaN components.
pub(crate) struct UncheckedProducts;
impl DirectionProducts for CheckedProducts {
    type Error = Failure;
    fn crossed(left: Direction3, right: Direction3) -> Result<Direction3, Failure> {
        left.crossed(right)
    }
    fn cross_crossed(
        base: Direction3,
        first: Direction3,
        second: Direction3,
    ) -> Result<Direction3, Failure> {
        base.cross_crossed(first, second)
    }
}
impl DirectionProducts for UncheckedProducts {
    type Error = Infallible;
    fn crossed(left: Direction3, right: Direction3) -> Result<Direction3, Infallible> {
        Ok(Direction3::normalize_unchecked(cross(
            left.coordinates,
            right.coordinates,
        )))
    }
    fn cross_crossed(
        base: Direction3,
        first: Direction3,
        second: Direction3,
    ) -> Result<Direction3, Infallible> {
        let inner = cross(first.coordinates, second.coordinates);
        Ok(Direction3::normalize_unchecked(cross(
            base.coordinates,
            inner,
        )))
    }
}
