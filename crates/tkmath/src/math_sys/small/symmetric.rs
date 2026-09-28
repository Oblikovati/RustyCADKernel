// Copyright (c) 2026 OPEN CASCADE SAS
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

//! Port of MathSys_NewtonTypes/2D/3D/4D at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::{NewtonBoundsN, NewtonOptions, NewtonResultN, checks, engine, linear};
use crate::math_utils::Status;
/// Native symmetric callback protocol; implement for `solve_2d_symmetric`.
pub trait SymmetricFunction2D {
    /// Evaluates two residuals; `function.value(&x, &mut f)`.
    fn value(&mut self, x: &[f64; 2], f: &mut [f64; 2]) -> bool;
    /// Supplies residuals and `[j11,j12,j22]`; `function.value_and_jacobian(&x, &mut f, &mut j)`.
    fn value_and_jacobian(&mut self, x: &[f64; 2], f: &mut [f64; 2], j: &mut [f64; 3]) -> bool;
}
struct Symmetric<'a, F>(&'a mut F);
impl<F: SymmetricFunction2D> engine::Policy<2> for Symmetric<'_, F> {
    const SYMMETRIC: bool = true;
    fn system(&mut self, x: &[f64; 2]) -> Result<([f64; 2], [[f64; 2]; 2]), Status> {
        let mut f: [f64; 2] = [0.0; 2];
        let mut j: [f64; 3] = [0.0; 3];
        let success: bool = self.0.value_and_jacobian(x, &mut f, &mut j);
        let matrix: [[f64; 2]; 2] = [[j[0], j[1]], [j[1], j[2]]];
        if success && checks::is_finite_system_n(&f, &matrix) {
            Ok((f, matrix))
        } else {
            Err(Status::NumericalError)
        }
    }
    fn value_norm(&mut self, x: &[f64; 2]) -> Result<f64, Status> {
        let mut f: [f64; 2] = [0.0; 2];
        if !self.0.value(x, &mut f) || f.iter().any(|v| !v.is_finite()) {
            return Err(Status::NumericalError);
        }
        Ok(checks::safe_norm_n(&f))
    }
    fn direction(&self, j: &[[f64; 2]; 2], f: &[f64; 2], residual: f64) -> Option<[f64; 2]> {
        linear::symmetric_direction(j, f, residual)
    }
}
/// Native symmetric 2D solver with separate value callbacks; `solve_2d_symmetric(&mut function, [0.0; 2], &bounds, &options)`.
pub fn solve_2d_symmetric<F: SymmetricFunction2D>(
    function: &mut F,
    start: [f64; 2],
    bounds: &NewtonBoundsN<2>,
    options: &NewtonOptions,
) -> NewtonResultN<2> {
    engine::solve(&mut Symmetric(function), start, bounds, options)
}
