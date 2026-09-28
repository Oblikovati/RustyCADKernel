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
// Rust translation of ModelingData/TKG3d/AdvApprox/AdvApprox_EvaluatorFunction.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `AdvApprox_EvaluatorFunction`: the function an approximation samples.
//!
//! The native virtual `Evaluate(int* Dimension, double StartEnd[2], double* Parameter,
//! int* DerivativeRequest, double* Result, int* ErrorCode)` passes every argument
//! through a pointer, but no caller in AdvApprox reads an input back after the call.
//! The trait therefore takes the inputs by value, fills `result` (`Dimension`
//! values) and returns the native `ErrorCode` (0 on success; any other value makes
//! the approximation stop without a result). `operator()` is the same call.

/// A function of one parameter with values in a space of `dimension` reals.
///
/// Closures with the matching signature implement it;
/// `let mut f = |_: i32, _: [f64; 2], t: f64, _: i32, r: &mut [f64]| { r[0] = t; 0 };`.
pub trait AdvApproxEvaluatorFunction {
    /// Writes the derivative of order `derivative_request` at `parameter` into
    /// `result[..dimension]`; `start_end` is the interval being approximated.
    /// Returns the error code, 0 on success (`Evaluate`);
    /// `let code: i32 = f.evaluate(1, [0.0, 1.0], 0.5, 0, &mut values);`.
    fn evaluate(
        &mut self,
        dimension: i32,
        start_end: [f64; 2],
        parameter: f64,
        derivative_request: i32,
        result: &mut [f64],
    ) -> i32;
}

impl<F> AdvApproxEvaluatorFunction for F
where
    F: FnMut(i32, [f64; 2], f64, i32, &mut [f64]) -> i32,
{
    fn evaluate(
        &mut self,
        dimension: i32,
        start_end: [f64; 2],
        parameter: f64,
        derivative_request: i32,
        result: &mut [f64],
    ) -> i32 {
        self(dimension, start_end, parameter, derivative_request, result)
    }
}
