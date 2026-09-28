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
mod types;
pub use types::{NewtonBoundsN, NewtonOptions, NewtonResultN};
mod engine;
mod linear;
mod merit;
mod symmetric;
use crate::math_utils::Status;
pub use symmetric::{SymmetricFunction2D, solve_2d_symmetric};
/// Evaluates a fixed square residual and Jacobian; closures can implement this directly.
/// Example: `|x: &[f64; 2], f: &mut [f64; 2], j: &mut [[f64; 2]; 2]| { *f = *x; *j = [[1.0,0.0],[0.0,1.0]]; true }`.
pub trait SmallSystem<const N: usize> {
    /// Supplies every residual and Jacobian entry; `function.evaluate(&x, &mut f, &mut j)`.
    fn evaluate(&mut self, x: &[f64; N], f: &mut [f64; N], j: &mut [[f64; N]; N]) -> bool;
}
impl<const N: usize, F: FnMut(&[f64; N], &mut [f64; N], &mut [[f64; N]; N]) -> bool> SmallSystem<N>
    for F
{
    fn evaluate(&mut self, x: &[f64; N], f: &mut [f64; N], j: &mut [[f64; N]; N]) -> bool {
        self(x, f, j)
    }
}
struct General<'a, F>(&'a mut F);
impl<const N: usize, F: SmallSystem<N>> engine::Policy<N> for General<'_, F> {
    fn system(&mut self, x: &[f64; N]) -> Result<([f64; N], [[f64; N]; N]), Status> {
        let mut f: [f64; N] = [0.0; N];
        let mut j: [[f64; N]; N] = [[0.0; N]; N];
        if self.0.evaluate(x, &mut f, &mut j) && merit::finite(&f, &j) {
            Ok((f, j))
        } else {
            Err(Status::NumericalError)
        }
    }
    fn value_norm(&mut self, x: &[f64; N]) -> Result<f64, Status> {
        self.system(x).map(|(f, _)| merit::norm(&f))
    }
    fn direction(&self, j: &[[f64; N]; N], f: &[f64; N], residual: f64) -> Option<[f64; N]> {
        linear::direction(j, f, residual)
    }
}
/// Native general 2D Newton method; `solve_2d(&mut function, [0.0; 2], &bounds, &options)`.
pub fn solve_2d<F: SmallSystem<2>>(
    function: &mut F,
    start: [f64; 2],
    bounds: &NewtonBoundsN<2>,
    options: &NewtonOptions,
) -> NewtonResultN<2> {
    engine::solve(&mut General(function), start, bounds, options)
}
/// Native general 3D Newton method; `solve_3d(&mut function, [0.0; 3], &bounds, &options)`.
pub fn solve_3d<F: SmallSystem<3>>(
    function: &mut F,
    start: [f64; 3],
    bounds: &NewtonBoundsN<3>,
    options: &NewtonOptions,
) -> NewtonResultN<3> {
    engine::solve(&mut General(function), start, bounds, options)
}
/// Native general 4D Newton method; `solve_4d(&mut function, [0.0; 4], &bounds, &options)`.
pub fn solve_4d<F: SmallSystem<4>>(
    function: &mut F,
    start: [f64; 4],
    bounds: &NewtonBoundsN<4>,
    options: &NewtonOptions,
) -> NewtonResultN<4> {
    engine::solve(&mut General(function), start, bounds, options)
}

mod geometry;
pub use geometry::*;
