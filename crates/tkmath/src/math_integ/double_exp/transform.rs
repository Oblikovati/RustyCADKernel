// Copyright (c) 2025 OPEN CASCADE SAS
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

//! MathInteg_DoubleExp.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use std::f64::consts::FRAC_PI_2;
#[derive(Clone, Copy)]
pub(super) enum Transform {
    Finite {
        lower: f64,
        upper: f64,
        mid: f64,
        half: f64,
    },
    Semi(f64),
    Infinite,
}
impl Transform {
    pub(super) fn signs(self) -> [f64; 2] {
        if matches!(self, Self::Finite { .. }) {
            [1.0, -1.0]
        } else {
            [-1.0, 1.0]
        }
    }
    pub(super) fn sample(self, t: f64) -> Option<(f64, f64)> {
        let u: f64 = FRAC_PI_2 * t.sinh();
        let cosh_t: f64 = t.cosh();
        match self {
            Self::Finite {
                lower,
                upper,
                mid,
                half,
            } => finite_sample(u, cosh_t, [lower, upper, mid, half]),
            Self::Semi(lower) => semi_sample(u, cosh_t, lower),
            Self::Infinite => infinite_sample(u, cosh_t),
        }
    }
}
fn finite_sample(u: f64, cosh_t: f64, bounds: [f64; 4]) -> Option<(f64, f64)> {
    if u.abs() > 700.0 {
        return None;
    }
    let cosh_u: f64 = u.cosh();
    let sech_squared: f64 = 1.0 / (cosh_u * cosh_u);
    let x: f64 = bounds[2] + bounds[3] * u.tanh();
    if x <= bounds[0] || x >= bounds[1] {
        return None;
    }
    let weight: f64 = bounds[3] * FRAC_PI_2 * cosh_t * sech_squared;
    if weight <= 0.0 || !weight.is_finite() {
        return None;
    }
    Some((x, weight))
}
fn semi_sample(u: f64, cosh_t: f64, lower: f64) -> Option<(f64, f64)> {
    if u > 700.0 {
        return None;
    }
    let exponential: f64 = u.exp();
    let weight: f64 = FRAC_PI_2 * cosh_t * exponential;
    if weight < crate::math_utils::ZERO_TOL || !weight.is_finite() || u < -30.0 {
        return None;
    }
    Some((lower + exponential, weight))
}
fn infinite_sample(u: f64, cosh_t: f64) -> Option<(f64, f64)> {
    if u.abs() > 700.0 {
        return None;
    }
    let x: f64 = u.sinh();
    let weight: f64 = FRAC_PI_2 * cosh_t * u.cosh();
    if weight < crate::math_utils::ZERO_TOL || !weight.is_finite() {
        return None;
    }
    Some((x, weight))
}
