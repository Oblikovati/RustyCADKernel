// Copyright (c) 1997-1999 Matra Datavision
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

//! Native sampling arithmetic; the source explicitly disables range checks.
/// Custom sampling protocols retain virtual parameter evaluation; implement `parameter(index)`.
pub trait FunctionSampling {
    /// Returns the declared bounds; `sample.bounds()`.
    fn bounds(&self) -> [f64; 2];
    /// Returns the declared number of points; `sample.number_of_points()`.
    fn number_of_points(&self) -> i32;
    /// Evaluates a one-based sample parameter; `sample.parameter(1)`.
    fn parameter(&self, index: i32) -> f64;
}
/// Uniform samples with native endpoint arithmetic; `FunctionSample::new(0.0,1.0,11)`.
pub struct FunctionSample {
    first: f64,
    last: f64,
    count: i32,
}
impl FunctionSample {
    /// Keeps bounds and count verbatim; `FunctionSample::new(0.0,1.0,11)`.
    pub fn new(first: f64, last: f64, count: i32) -> Self {
        Self { first, last, count }
    }
    /// Reads the original endpoint parameters; `sample.bounds()`.
    pub fn bounds(&self) -> [f64; 2] {
        [self.first, self.last]
    }
    /// Reads the supplied point count; `sample.number_of_points()`.
    pub fn number_of_points(&self) -> i32 {
        self.count
    }
    /// Retains the unchecked native formula, including extrapolation; `sample.parameter(1)`.
    pub fn parameter(&self, index: i32) -> f64 {
        (f64::from(self.count - index) * self.first + f64::from(index - 1) * self.last)
            / f64::from(self.count - 1)
    }
}
impl FunctionSampling for FunctionSample {
    fn bounds(&self) -> [f64; 2] {
        self.bounds()
    }
    fn number_of_points(&self) -> i32 {
        self.number_of_points()
    }
    fn parameter(&self, index: i32) -> f64 {
        self.parameter(index)
    }
}
