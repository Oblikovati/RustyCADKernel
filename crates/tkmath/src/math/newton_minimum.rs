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
use super::{Gauss, Jacobi, Matrix, MultipleVarFunctionWithHessian, Status, Vector};
use rustycad_tkernel::standard::Failure;
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
    status: Status,
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
        let mut result: Optimization = Optimization::new(size, tolerance, limit, 0.0)?;
        result.failure_message = "NewtonMinimum";
        Ok(Self {
            result,
            step: Vector::filled(1, size, 10.0 * tolerance)?,
            hessian: Matrix::with_bounds(1, size, 1, size).unwrap(),
            convexity,
            singularity,
            convex: true,
            min_eigenvalue: 0.0,
            status: Status::NotBracketed,
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
        let mut previous: Vector<'static> = Vector::with_bounds(1, function.variables())?;
        previous.initialized(start)?;
        let mut next: Vector<'static> = Vector::with_bounds(1, function.variables())?;
        self.result.done = true;
        self.status = Status::Ok;
        self.result.iterations = 0;
        self.iterate(function, &mut previous, &mut next);
        Ok(())
    }
    /// Applies the native step or relative-objective criterion; `solver.is_converged()`.
    pub fn is_converged(&self) -> bool {
        self.step.norm().unwrap() <= self.result.tolerance
            || (self.result.minimum - self.result.previous).abs()
                <= self.result.tolerance * self.result.previous.abs()
    }
    /// Reads status even when not done, as native GetStatus does; `solver.status()`.
    pub fn status(&self) -> Status {
        self.status
    }
    fn fail(&mut self, status: Status) -> bool {
        self.result.done = false;
        self.status = status;
        false
    }
}
gradient_results!(NewtonMinimum);
