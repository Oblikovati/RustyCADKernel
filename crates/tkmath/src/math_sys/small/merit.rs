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
use crate::math_poly::extended::LongDouble as E;
pub(super) fn norm<const N: usize>(values: &[f64; N]) -> f64 {
    values.iter().fold(0.0_f64, |a, v| a.hypot(*v))
}
pub(super) fn finite<const N: usize>(f: &[f64; N], j: &[[f64; N]; N]) -> bool {
    f.iter().chain(j.iter().flatten()).all(|v| v.is_finite())
}
pub(super) fn derivative<const N: usize>(f: &[f64; N], j: &[[f64; N]; N], step: &[f64; N]) -> E {
    (0..N).fold(E::ZERO, |total, row| {
        let value: E = (0..N).fold(E::ZERO, |a, col| {
            a + E::new(j[row][col]) * E::new(step[col])
        });
        total + E::new(f[row]) * value
    })
}
pub(super) fn armijo(current: f64, trial: f64, derivative: E) -> bool {
    if current.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater)
        || derivative.partial_cmp(&E::ZERO) != Some(std::cmp::Ordering::Less)
    {
        return false;
    }
    let current: E = E::new(current);
    let ratio: E = E::new(trial) / current;
    let normalized: E = derivative / current / current;
    let bound: E = E::new(1.0) + E::new(2.0) * E::new(1e-4) * normalized;
    bound >= E::ZERO && ratio <= bound.sqrt()
}
