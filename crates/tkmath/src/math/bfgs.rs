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

//! Direct math_BFGS.cxx base algorithm at OCCT 3d097a0; retains OCC30492 convergence order.
use super::optimization::{Optimization, optimization_results};
use super::{Matrix, MultipleVarFunctionWithGradient, Vector};
use rustycad_tkernel::standard::Failure;
mod bounds;
mod iterate;
mod line;
mod update;
/// Broyden–Fletcher–Goldfarb–Shanno minimizer; `Bfgs::new(2)?`.
pub struct Bfgs {
    result: Optimization,
    bounds: Option<(Vector<'static>, Vector<'static>)>,
}
impl Bfgs {
    /// Uses native tolerance and iteration defaults; `Bfgs::new(2)?`.
    pub fn new(size: i32) -> Result<Self, Failure> {
        Self::with_options(size, 1e-8, 200, 1e-12)
    }
    /// Sets all native solver options; `Bfgs::with_options(2,1e-8,200,1e-12)?`.
    pub fn with_options(
        size: i32,
        tolerance: f64,
        limit: i32,
        epsilon: f64,
    ) -> Result<Self, Failure> {
        Ok(Self {
            result: Optimization::new(size, tolerance, limit, epsilon)?,
            bounds: None,
        })
    }
    /// Defines the box for subsequent searches; `solver.set_boundary(&lower,&upper)?`.
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
    /// Runs minimization, preserving native callback and stopping order; `solver.perform(&mut f,&start)?`.
    pub fn perform(
        &mut self,
        function: &mut impl MultipleVarFunctionWithGradient,
        start: &Vector<'_>,
    ) -> Result<(), Failure> {
        if !self.result.initialize(function, start)? {
            return Ok(());
        }
        let size: i32 = self.result.location.length();
        let mut hessian: Matrix<'static> = Matrix::filled(1, size, 1, size, 0.0).unwrap();
        let mut direction: Vector<'static> = Vector::with_bounds(1, size)?;
        for index in 1..=size {
            hessian[(index, index)] = 1.0;
            direction[index] = -self.result.gradient[index];
        }
        self.iterate(function, &mut direction, &mut hessian);
        Ok(())
    }
}
optimization_results!(Bfgs);
