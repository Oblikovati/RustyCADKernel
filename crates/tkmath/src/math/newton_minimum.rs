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

//! Direct math_NewtonMinimum.cxx at OCCT 3d097a0; Done and Status remain independent.
use super::optimization::{Optimization, gradient_results};
use super::{Gauss, Jacobi, Matrix, MultipleVarFunctionWithHessian, Status, Vector, stream_real};
use rustycad_tkernel::standard::Failure;
use std::fmt::Write;
mod bounds;
mod curvature;
mod iterate;
mod trial;
/// Hessian-based minimization; `NewtonMinimum::new(&function)?`.
pub struct NewtonMinimum {
    result: Optimization,
    step: Vector<'static>,
    hessian: Matrix<'static>,
    convexity: f64,
    singularity: bool,
    convex: bool,
    min_eigenvalue: f64,
    bounds: Option<(Vector<'static>, Vector<'static>)>,
}
impl NewtonMinimum {
    /// Uses native confusion, 40 iterations and singularity treatment; `NewtonMinimum::new(&function)?`.
    pub fn new(function: &impl MultipleVarFunctionWithHessian) -> Result<Self, Failure> {
        Self::with_options(function, 1e-7, 40, 1e-6, true)
    }
    /// Configures the native Hessian solver; `NewtonMinimum::with_options(&function,1e-8,40,1e-6,true)?`.
    pub fn with_options(
        function: &impl MultipleVarFunctionWithHessian,
        tolerance: f64,
        limit: i32,
        convexity: f64,
        singularity: bool,
    ) -> Result<Self, Failure> {
        let size: i32 = function.variables();
        let mut result: Optimization =
            Optimization::new(size, [tolerance, 0.0], limit, Status::NotBracketed)?;
        result.failure_message = "NewtonMinimum";
        Ok(Self {
            result,
            step: Vector::filled(1, size, 10.0 * tolerance)?,
            hessian: Matrix::with_bounds(1, size, 1, size).unwrap(),
            convexity,
            singularity,
            convex: true,
            min_eigenvalue: 0.0,
            bounds: None,
        })
    }
    /// Defines the box used to project Newton steps; `solver.set_boundary(&left,&right)?`.
    pub fn set_boundary(&mut self, left: &Vector<'_>, right: &Vector<'_>) -> Result<(), Failure> {
        let size: i32 = self.result.location.length();
        let bounds: &mut (Vector<'static>, Vector<'static>) =
            self.bounds.get_or_insert_with(|| {
                (
                    Vector::filled(1, size, 0.0).unwrap(),
                    Vector::filled(1, size, 0.0).unwrap(),
                )
            });
        bounds.0.initialized(left)?;
        bounds.1.initialized(right)?;
        Ok(())
    }
    /// Runs until two consecutive convergence checks or a native exit; `solver.perform(&mut function,&start)?`.
    pub fn perform(
        &mut self,
        function: &mut impl MultipleVarFunctionWithHessian,
        start: &Vector<'_>,
    ) -> Result<(), Failure> {
        self.perform_with_criterion(function, start, &mut |solver: &Self| solver.is_converged())
    }
    /// Runs with a replacement for the virtual `IsConverged()` (FairCurve_Newton's hook);
    /// the criterion reads `current_step`, `current_minimum`, `previous_minimum` and
    /// `tolerance`. `solver.perform_with_criterion(&mut f,&start,&mut |s| s.is_converged())?`.
    pub fn perform_with_criterion(
        &mut self,
        function: &mut impl MultipleVarFunctionWithHessian,
        start: &Vector<'_>,
        criterion: &mut impl FnMut(&Self) -> bool,
    ) -> Result<(), Failure> {
        let mut previous: Vector<'static> = Vector::with_bounds(1, function.variables())?;
        previous.initialized(start)?;
        let mut next: Vector<'static> = Vector::with_bounds(1, function.variables())?;
        self.result.succeed();
        self.result.iterations = 0;
        self.iterate(function, [&mut previous, &mut next], criterion);
        Ok(())
    }
    /// Applies the native step or relative-objective criterion; `solver.is_converged()`.
    pub fn is_converged(&self) -> bool {
        self.step.norm().unwrap() <= self.result.tolerance
            || (self.result.minimum - self.result.previous).abs()
                <= self.result.tolerance * self.result.previous.abs()
    }
    /// Protected `TheStep`, the last Newton step; `solver.current_step()`.
    pub fn current_step(&self) -> &Vector<'static> {
        &self.step
    }
    /// Writes the native `Dump` text. Like OCCT, it reads `Location()` and friends, so a
    /// solver that is not done raises `StdFail_NotDone` after the status lines were
    /// written. `solver.dump(&mut text)?`.
    pub fn dump(&self, stream: &mut impl Write) -> Result<(), Failure> {
        let write = |_| Failure::new("math_NewtonMinimum::Dump: the stream refused a write");
        write!(stream, "math_Newton Optimisation: ").map_err(write)?;
        writeln!(stream, " Done   ={}", i32::from(self.result.done)).map_err(write)?;
        writeln!(stream, " Status = {}", self.result.status as i32).map_err(write)?;
        write!(stream, " Location Vector = ").map_err(write)?;
        self.location()?.dump(stream).map_err(write)?;
        writeln!(stream).map_err(write)?;
        writeln!(stream, " Minimum value = {}", stream_real(self.minimum()?)).map_err(write)?;
        let previous = stream_real(self.result.previous);
        writeln!(stream, " Previous value = {previous}").map_err(write)?;
        writeln!(stream, " Number of iterations = {}", self.iterations()?).map_err(write)?;
        writeln!(stream, " Convexity = {}", i32::from(self.convex)).map_err(write)?;
        let eigen = stream_real(self.min_eigenvalue);
        writeln!(stream, " Eigen Value = {eigen}").map_err(write)
    }
    fn fail(&mut self, status: Status) -> bool {
        self.result.fail(status);
        false
    }
}
gradient_results!(NewtonMinimum);
