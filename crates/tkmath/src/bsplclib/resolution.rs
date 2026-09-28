// Copyright (c) 1991-1999 Matra Datavision
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
// Rust translation of FoundationClasses/TKMath/BSplCLib/BSplCLib.cxx (Resolution) and BSplCLib_CurveComputation.pxx (BSplCLib_Resolution) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Parametric tolerance from a bound on the derivative of the curve.
//!
//! For C(t) = sum Ci Bi(t), |C'| <= d * max |Ci - Ci-1| / (ti+d - ti). In the rational
//! case the bound is taken over the differences Di (Ci - Cj) - Di-1 (Ci-1 - Cj) for
//! the poles j sharing a support, divided by the smallest weight. The native
//! dimension 2..4 specialisations perform the same operations as the general loop.
use super::flat_points;
use super::native::real_small;
use super::point::CurvePoint;
use crate::plib::access::{array_value, buffer_value};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;

const RESOLUTION: &str = "BSplCLib::Resolution";

/// Parametric tolerance matching `tolerance_3d` (`BSplCLib::Resolution` on reals);
/// `poles` holds `num_poles` rows of `array_dimension` values (read cyclically);
/// `bsplclib::resolution_flat(&poles, 3, 4, None, &flat, 3, 1e-7)?`.
pub fn resolution_flat(
    poles: &[f64],
    array_dimension: i32,
    num_poles: i32,
    weights: Option<&Array1<'_, f64>>,
    flat_knots: &Array1<'_, f64>,
    degree: i32,
    tolerance_3d: f64,
) -> Result<f64, Failure> {
    if num_poles <= 0 {
        return Err(Failure::out_of_range(&format!(
            "{RESOLUTION}: NumPoles = {num_poles}; expected > 0"
        )));
    }
    let bound = DerivativeBound {
        poles,
        dim: i64::from(array_dimension),
        num_poles,
        knots: flat_knots,
        degree,
    };
    let mut max_derivative: f64 = match weights {
        None => bound.polynomial()?,
        Some(weights) => bound.rational(weights)?,
    };
    max_derivative *= f64::from(degree);
    if max_derivative > real_small() {
        return Ok(tolerance_3d / max_derivative);
    }
    Ok(tolerance_3d / real_small())
}

struct DerivativeBound<'p, 'k, 'a> {
    poles: &'p [f64],
    dim: i64,
    num_poles: i32,
    knots: &'k Array1<'a, f64>,
    degree: i32,
}

impl DerivativeBound<'_, '_, '_> {
    fn flat_poles(&self) -> i32 {
        self.knots.len() as i32 - (self.degree + 1)
    }

    fn pole(&self, index: i32, k: i64) -> Result<f64, Failure> {
        buffer_value(RESOLUTION, self.poles, i64::from(index) * self.dim + k)
    }

    // 1 / (FK[ii+Degree] - FK[ii]) with FK starting at FlatKnots.Lower().
    fn inverse(&self, ii: i32) -> Result<f64, Failure> {
        let lower: i32 = self.knots.lower();
        let span: f64 = array_value(self.knots, lower + ii + self.degree)?
            - array_value(self.knots, lower + ii)?;
        Ok(1.0e0 / span)
    }

    fn polynomial(&self) -> Result<f64, Failure> {
        let mut max_derivative: f64 = 0.0e0;
        for ii in 1..self.flat_poles() {
            let (index, minus): (i32, i32) = (ii % self.num_poles, (ii - 1) % self.num_poles);
            let inverse: f64 = self.inverse(ii)?;
            let mut value: f64 = 0.0e0;
            for k in 0..self.dim {
                value += magnitude(self.pole(index, k)? - self.pole(minus, k)?);
            }
            value *= inverse;
            if max_derivative < value {
                max_derivative = value;
            }
        }
        Ok(max_derivative)
    }

    fn rational(&self, weights: &Array1<'_, f64>) -> Result<f64, Failure> {
        let wg = |i: i32| array_value(weights, weights.lower() + i);
        let mut min_weights: f64 = wg(0)?;
        for ii in 1..self.num_poles {
            let w: f64 = wg(ii)?;
            if w < min_weights {
                min_weights = w;
            }
        }
        let num_poles: i32 = self.flat_poles();
        let mut max_derivative: f64 = 0.0e0;
        for ii in 1..num_poles {
            let (index, minus): (i32, i32) = (ii % self.num_poles, (ii - 1) % self.num_poles);
            let (w_index, w_minus): (f64, f64) = (wg(index)?, wg(minus)?);
            let inverse: f64 = self.inverse(ii)?;
            let lower: i32 = (ii - (self.degree + 1)).max(0);
            let upper: i32 = (2 * self.degree + 1 + ii).min(num_poles);
            for jj in lower..upper {
                let other: i32 = jj % self.num_poles;
                let mut value: f64 = 0.0e0;
                for k in 0..self.dim {
                    let pj: f64 = self.pole(other, k)?;
                    let factor: f64 = ((pj - self.pole(index, k)?) * w_index)
                        - ((pj - self.pole(minus, k)?) * w_minus);
                    value += magnitude(factor);
                }
                value *= inverse;
                if max_derivative < value {
                    max_derivative = value;
                }
            }
        }
        Ok(max_derivative / min_weights)
    }
}

// `if (factor < 0) factor = -factor;`, which keeps -0.0 unlike f64::abs.
fn magnitude(factor: f64) -> f64 {
    if factor < 0.0 { -factor } else { factor }
}

/// Parametric tolerance of a 2D/3D curve (`BSplCLib::Resolution` on gp_Pnt/gp_Pnt2d);
/// `bsplclib::resolution(&poles, None, 4, &flat, 3, 1e-7)?`.
pub fn resolution<P: CurvePoint>(
    poles: &Array1<'_, P>,
    weights: Option<&Array1<'_, f64>>,
    num_poles: i32,
    flat_knots: &Array1<'_, f64>,
    degree: i32,
    tolerance_3d: f64,
) -> Result<f64, Failure> {
    let flat: Vec<f64> = flat_points(poles);
    resolution_flat(
        &flat,
        P::DIMENSION as i32,
        num_poles,
        weights,
        flat_knots,
        degree,
        tolerance_3d,
    )
}
