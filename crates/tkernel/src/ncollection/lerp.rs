// Created by: Kirill GAVRILOV
// Copyright (c) 2016 OPEN CASCADE SAS
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

// Rust translation of FoundationClasses/TKernel/NCollection/NCollection_Lerp.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! NCollection_Lerp: linear interpolation between two values of any type that supports
//! `scalar * value` and `value + value`.
//!
//! The native template evaluates `(1.0 - t) * start + t * end`; keeping that expression (not
//! `start + (end - start) * t`) keeps the rounding of both end points exact at t = 0 and 1.
//! Specializations such as gp_TrsfNLerp are separate types, because they interpolate parts
//! of a value rather than the value itself.
use std::ops::{Add, Mul};
/// Start and end values of a linear interpolation; `Lerp::new(1.0, 3.0).interpolate(0.5)`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Lerp<T> {
    start: T,
    end: T,
}
impl<T> Lerp<T> {
    /// Stores both end values, as `NCollection_Lerp(theStart, theEnd)`; `Lerp::new(1.0, 3.0)`.
    pub const fn new(start: T, end: T) -> Self {
        Self { start, end }
    }
    /// Replaces both end values, as `Init`; `lerp.init(1.0, 3.0)`.
    pub fn init(&mut self, start: T, end: T) {
        self.start = start;
        self.end = end;
    }
}
impl<T> Lerp<T>
where
    T: Clone + Add<Output = T>,
    f64: Mul<T, Output = T>,
{
    /// `(1 - t) * start + t * end`, as `Interpolate(theT, theResult)`; `lerp.interpolate(0.25)`.
    pub fn interpolate(&self, parameter: f64) -> T {
        (1.0 - parameter) * self.start.clone() + parameter * self.end.clone()
    }
    /// The static `Interpolate(theStart, theEnd, theT)`; `Lerp::interpolated(1.0, 3.0, 0.5)`.
    pub fn interpolated(start: T, end: T, parameter: f64) -> T {
        Self::new(start, end).interpolate(parameter)
    }
}
