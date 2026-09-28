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

//! MathLin_Utils.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use crate::math::{Matrix, Vector};
/// Checks every entry, accepting empty matrices; `is_finite_matrix(&a)`.
pub fn is_finite_matrix(a: &Matrix<'_>) -> bool {
    super::jacobi::finite_matrix(a)
}
/// Checks every entry, accepting empty vectors; `is_finite_vector(&x)`.
pub fn is_finite_vector(x: &Vector<'_>) -> bool {
    x.values().iter().all(|v| v.is_finite())
}
/// Accumulates a scaled norm with native failure writes; `accumulate_norm(x,&mut scale,&mut sum)`.
pub fn accumulate_norm(value: f64, scale: &mut f64, sum: &mut f64) -> bool {
    if !value.is_finite() || !scale.is_finite() || !sum.is_finite() {
        return false;
    }
    let magnitude: f64 = value.abs();
    if magnitude == 0.0 {
        return true;
    }
    if *scale < magnitude {
        let ratio: f64 = *scale / magnitude;
        *sum = 1.0 + *sum * ratio * ratio;
        *scale = magnitude;
    } else {
        let ratio: f64 = magnitude / *scale;
        *sum += ratio * ratio;
    }
    scale.is_finite() && sum.is_finite()
}
/// Rejects invalid accumulation state or overflow; `norm(scale,sum)`.
pub fn norm(scale: f64, sum: f64) -> Option<f64> {
    if !scale.is_finite() || !sum.is_finite() || scale < 0.0 || sum < 0.0 {
        return None;
    }
    let norm: f64 = if scale == 0.0 {
        0.0
    } else {
        scale * sum.sqrt()
    };
    norm.is_finite().then_some(norm)
}
/// Applies the native machine-precision floor; `relative_tolerance(0.0,3)`.
pub fn relative_tolerance(tolerance: f64, dimension: usize) -> f64 {
    if !tolerance.is_finite() || tolerance < 0.0 {
        return -1.0;
    }
    tolerance.max(f64::EPSILON * dimension.max(1) as f64)
}
