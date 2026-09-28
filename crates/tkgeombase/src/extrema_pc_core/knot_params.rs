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
// Rust translation of buildKnotAwareParams of ExtremaPC2d_BSplineCurve.cxx and
// ExtremaPC_BSplineCurve.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The knot-aware partition of the BSpline evaluators: the domain ends, every knot
//! strictly inside, and `2 (degree + 1) - 1` uniform samples inside each knot span
//! intersected with the domain, over every period that meets a periodic domain.
use super::validity::THE_BSPLINE_SPAN_MULTIPLIER;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::math::Vector;
use rustycad_tkmath::math_utils::Domain1D;

/// The BSpline data the partition reads.
pub(crate) struct KnotGrid {
    /// The distinct knots, in order (`Knots()`).
    pub knots: Vec<f64>,
    /// `Degree()`.
    pub degree: i32,
    /// `Period()` of a periodic curve.
    pub period: Option<f64>,
    /// The native class, for the RangeError message.
    pub class_name: &'static str,
}

/// `buildKnotAwareParams`: Standard_RangeError when the shift range or the parameter
/// count does not fit an `int`.
pub(crate) fn knot_aware_params(
    grid: &KnotGrid,
    domain: &Domain1D,
) -> Result<Vector<'static>, Failure> {
    let (u_min, u_max) = (domain.min, domain.max);
    let samples_per_span: usize = THE_BSPLINE_SPAN_MULTIPLIER * (grid.degree + 1) as usize;
    let period: f64 = grid.period.unwrap_or(0.0);
    let (first_knot, last_knot) = (grid.knots[0], grid.knots[grid.knots.len() - 1]);
    let (first_shift, last_shift) = match grid.period {
        Some(period) => (
            ((u_min - last_knot) / period).floor(),
            ((u_max - first_knot) / period).ceil(),
        ),
        None => (0.0, 0.0),
    };
    let nb_shifts: f64 = last_shift - first_shift + 1.0;
    let max_params: f64 = 2.0 + nb_shifts * (grid.knots.len() - 1) as f64 * samples_per_span as f64;
    let (int_first, int_last) = (f64::from(i32::MIN), f64::from(i32::MAX));
    if first_shift < int_first
        || first_shift > int_last
        || last_shift < int_first
        || last_shift > int_last
        || nb_shifts < 1.0
        || max_params > int_last
    {
        let message = format!(
            "{}: parameter grid is too large: shifts [{first_shift}, {last_shift}], {max_params} parameters; expected int ranges",
            grid.class_name
        );
        return Err(Failure::with_type("Standard_RangeError", &message, ""));
    }
    let mut params: Vec<f64> = Vec::with_capacity(max_params as usize);
    params.push(u_min);
    let mut shift_index: i32 = first_shift as i32;
    loop {
        let shift: f64 = f64::from(shift_index) * period;
        add_span_samples(
            &grid.knots,
            shift,
            (u_min, u_max),
            samples_per_span,
            &mut params,
        );
        if shift_index == last_shift as i32 {
            break;
        }
        shift_index += 1;
    }
    params.push(u_max);
    let mut result: Vector<'static> = Vector::with_size(params.len());
    for (index, parameter) in params.iter().enumerate() {
        result[index as i32] = *parameter;
    }
    Ok(result)
}

// The interior samples and inner knots of every span of one period shift.
fn add_span_samples(
    knots: &[f64],
    shift: f64,
    (u_min, u_max): (f64, f64),
    samples_per_span: usize,
    params: &mut Vec<f64>,
) {
    for pair in knots.windows(2) {
        let (knot_lo, knot_hi) = (pair[0] + shift, pair[1] + shift);
        let span_lo: f64 = rustycad_tkg2d::geom2d::native::std_max(knot_lo, u_min);
        let span_hi: f64 = rustycad_tkg2d::geom2d::native::std_min(knot_hi, u_max);
        if span_hi <= span_lo {
            continue;
        }
        let step: f64 = (span_hi - span_lo) / samples_per_span as f64;
        for sample_index in 1..samples_per_span {
            let u: f64 = span_lo + sample_index as f64 * step;
            if u > u_min && u < u_max {
                params.push(u);
            }
        }
        if knot_hi > u_min && knot_hi < u_max {
            params.push(knot_hi);
        }
    }
}
