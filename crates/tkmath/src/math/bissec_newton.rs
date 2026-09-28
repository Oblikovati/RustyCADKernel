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

//! Combined bisection/Newton solver, preserving native stopping and cached outputs.
use super::FunctionWithDerivative;
use rustycad_tkernel::standard::Failure;
mod iterate;
/// Reusable bracketed Newton solver; `BissecNewton::new(1e-10)`.
pub struct BissecNewton {
    tolerance: f64,
    x: f64,
    dx: f64,
    value: f64,
    derivative: f64,
    done: bool,
}
impl BissecNewton {
    /// Creates an unfinished solver with absolute step tolerance; `BissecNewton::new(1e-10)`.
    pub fn new(tolerance: f64) -> Self {
        Self {
            tolerance,
            x: 0.0,
            dx: 0.0,
            value: 0.0,
            derivative: 0.0,
            done: false,
        }
    }
    /// Performs a fresh bracket search; `solver.perform(&mut function,0.0,2.0,100)`.
    pub fn perform(
        &mut self,
        function: &mut impl FunctionWithDerivative,
        first: f64,
        second: f64,
        limit: i32,
    ) {
        self.done = false;
        let Some((low, high)) = self.bracket(function, first, second) else {
            return;
        };
        self.x = 0.5 * (first + second);
        self.dx = (second - first).abs();
        if !function.values(self.x, &mut self.value, &mut self.derivative) {
            return;
        }
        self.iterate(function, low, high, limit);
    }
    fn bracket(
        &mut self,
        function: &mut impl FunctionWithDerivative,
        first: f64,
        second: f64,
    ) -> Option<(f64, f64)> {
        let mut fl: f64 = 0.0;
        let mut fh: f64 = 0.0;
        if !function.values(first, &mut fl, &mut self.derivative) {
            return None;
        }
        if !function.values(second, &mut fh, &mut self.derivative) {
            return None;
        }
        let tolerance: f64 = f64::EPSILON;
        if fl * fh > tolerance * tolerance {
            return None;
        }
        if fl < -tolerance || (fl < tolerance && fh < -tolerance) {
            Some((first, second))
        } else {
            Some((second, first))
        }
    }
    /// Reports successful convergence; `solver.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }
    /// Reads the latest root after convergence; `solver.root()?`.
    pub fn root(&self) -> Result<f64, Failure> {
        self.check_done()?;
        Ok(self.x)
    }
    /// Reads the cached function value, which may precede the final step; `solver.value()?`.
    pub fn value(&self) -> Result<f64, Failure> {
        self.check_done()?;
        Ok(self.value)
    }
    /// Reads the cached derivative, preserving native early convergence; `solver.derivative()?`.
    pub fn derivative(&self) -> Result<f64, Failure> {
        self.check_done()?;
        Ok(self.derivative)
    }
    /// Applies the default native step criterion; `solver.is_solution_reached()`.
    pub fn is_solution_reached(&self) -> bool {
        self.dx.abs() <= self.tolerance
    }
    fn check_done(&self) -> Result<(), Failure> {
        if !self.done {
            return Err(Failure::with_type("StdFail_NotDone", " ", ""));
        }
        Ok(())
    }
}
