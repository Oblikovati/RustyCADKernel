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

//! One-variable adapter around the native general function-set root algorithm.
use super::{
    FunctionSet, FunctionSetRoot, FunctionSetWithDerivatives, FunctionWithDerivative, Matrix,
    Vector,
};
use rustycad_tkernel::standard::Failure;
mod adapter;
mod results;
/// Scalar root result; `FunctionRoot::new(&mut function,3.0,1e-6)?`.
pub struct FunctionRoot {
    done: bool,
    root: f64,
    derivative: f64,
    value: f64,
    iterations: i32,
}
impl FunctionRoot {
    /// Uses the default 100-iteration limit; `FunctionRoot::new(&mut function,3.0,1e-6)?`.
    pub fn new(
        function: &mut impl FunctionWithDerivative,
        guess: f64,
        tolerance: f64,
    ) -> Result<Self, Failure> {
        Self::with_iterations(function, guess, tolerance, 100)
    }
    /// Configures the iteration limit without finite domain bounds; `FunctionRoot::with_iterations(&mut function,3.0,1e-6,50)?`.
    pub fn with_iterations(
        function: &mut impl FunctionWithDerivative,
        guess: f64,
        tolerance: f64,
        limit: i32,
    ) -> Result<Self, Failure> {
        Self::solve(function, guess, tolerance, None, limit)
    }
    /// Searches within explicit bounds; `FunctionRoot::bounded(&mut function,1.5,1e-6,1.0,3.0,100)?`.
    pub fn bounded(
        function: &mut impl FunctionWithDerivative,
        guess: f64,
        tolerance: f64,
        lower: f64,
        upper: f64,
        limit: i32,
    ) -> Result<Self, Failure> {
        Self::solve(function, guess, tolerance, Some([lower, upper]), limit)
    }
    fn solve(
        function: &mut impl FunctionWithDerivative,
        guess: f64,
        tolerance: f64,
        bounds: Option<[f64; 2]>,
        limit: i32,
    ) -> Result<Self, Failure> {
        let solver: FunctionSetRoot = adapter::search(function, guess, tolerance, bounds, limit)?;
        let mut result: Self = Self {
            done: solver.is_done(),
            root: 0.0,
            derivative: 0.0,
            value: 0.0,
            iterations: 0,
        };
        if !result.done {
            return Ok(result);
        }
        result.iterations = solver.iterations()?;
        function.state_number();
        result.root = solver.root()?[1];
        result.derivative = solver.derivative()?[(1, 1)];
        function.value(result.root, &mut result.value);
        Ok(result)
    }
}
