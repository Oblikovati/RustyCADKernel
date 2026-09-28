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

//! Newton iteration for square function systems with native LU and clipping.
use super::{FunctionSetWithDerivatives, Gauss, Matrix, Vector};
use rustycad_tkernel::standard::Failure;
mod iterate;
mod results;
/// Newton system solver; `NewtonFunctionSetRoot::new(&function,&tolerances,1e-6,100)?`.
pub struct NewtonFunctionSetRoot {
    tolerance: Vector<'static>,
    function_tolerance: f64,
    solution: Vector<'static>,
    delta: Vector<'static>,
    residual: Vector<'static>,
    jacobian: Matrix<'static>,
    done: bool,
    _state: i32,
    iteration: i32,
    limit: i32,
}
impl NewtonFunctionSetRoot {
    /// Configures both step and residual tolerances; `NewtonFunctionSetRoot::new(&function,&tolerances,1e-6,100)?`.
    pub fn new(
        function: &impl FunctionSetWithDerivatives,
        tolerance: &Vector<'_>,
        function_tolerance: f64,
        limit: i32,
    ) -> Result<Self, Failure> {
        let mut solver: Self = Self::without_step_tolerance(function, function_tolerance, limit)?;
        solver.set_tolerance(tolerance)?;
        Ok(solver)
    }
    /// Allocates state before step tolerances are supplied; `NewtonFunctionSetRoot::without_step_tolerance(&function,1e-6,100)?`.
    pub fn without_step_tolerance(
        function: &impl FunctionSetWithDerivatives,
        function_tolerance: f64,
        limit: i32,
    ) -> Result<Self, Failure> {
        let n: i32 = function.variables();
        Ok(Self {
            tolerance: Vector::with_bounds(1, n)?,
            function_tolerance,
            solution: Vector::with_bounds(1, n)?,
            delta: Vector::with_bounds(1, n)?,
            residual: Vector::with_bounds(1, n)?,
            jacobian: Matrix::with_bounds(1, n, 1, n).unwrap(),
            done: false,
            _state: 0,
            iteration: 0,
            limit,
        })
    }
    /// Copies tolerance indices 1..n exactly as native code does; `solver.set_tolerance(&tolerances)?`.
    pub fn set_tolerance(&mut self, tolerance: &Vector<'_>) -> Result<(), Failure> {
        for index in 1..=self.tolerance.length() {
            self.tolerance[index] = *tolerance.value(index)?;
        }
        Ok(())
    }
    /// Searches with finite native RealFirst/RealLast bounds; `solver.perform(&mut function,&start)?`.
    pub fn perform(
        &mut self,
        function: &mut impl FunctionSetWithDerivatives,
        start: &Vector<'_>,
    ) -> Result<(), Failure> {
        let lower: Vector<'static> = Vector::filled(1, function.variables(), -f64::MAX)?;
        let upper: Vector<'static> = Vector::filled(1, function.variables(), f64::MAX)?;
        self.perform_bounded(function, start, &lower, &upper)
    }
    /// Clips each Newton trial to the supplied declared bounds; `solver.perform_bounded(&mut function,&start,&lower,&upper)?`.
    pub fn perform_bounded(
        &mut self,
        function: &mut impl FunctionSetWithDerivatives,
        start: &Vector<'_>,
        lower: &Vector<'_>,
        upper: &Vector<'_>,
    ) -> Result<(), Failure> {
        self.done = false;
        self.solution.initialized(start)?;
        if !function.values(&self.solution, &mut self.residual, &mut self.jacobian) {
            return Ok(());
        }
        self.iterate(function, lower, upper)
    }
    /// Applies native un-clipped step and residual tests; `solver.is_solution_reached()`.
    pub fn is_solution_reached(&self) -> bool {
        for index in 1..=self.delta.length() {
            if self.delta[index].abs() > self.tolerance[index]
                || self.residual[index].abs() > self.function_tolerance
            {
                return false;
            }
        }
        true
    }
}
