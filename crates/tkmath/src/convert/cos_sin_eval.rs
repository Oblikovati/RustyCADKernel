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
// Rust translation of Convert_ConicToBSplineCurve.cxx (CosAndSinRationalC1, CosAndSinQuasiAngular, AlgorithmicCosAndSin and the QuasiAngular/RationalC1 branch of BuildCosAndSin) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The QuasiAngular and RationalC1 arcs: `(U² - V², 2UV) / (U² + V²)` with
//! polynomial `U`, `V`, interpolated at the Schoenberg points of the target knots.
use super::CosAndSinEvalFunction;
use super::arrays::one_based;
use super::cos_sin::{CosAndSin, SpanLayout, construction};
use crate::bsplclib::{self, CurveSpan};
use crate::gp::{Point2, Point3};
use crate::plib;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use std::f64::consts::PI;

// The auxiliary curve (U, V) the evaluator reads.
struct EvalCurve {
    degree: i32,
    poles: Array1<'static, Point2>,
    knots: Array1<'static, f64>,
    mults: Array1<'static, i32>,
    evaluator: CosAndSinEvalFunction,
}

/// Fills the QuasiAngular (`quasi_angular`) or RationalC1 arc on `[u_first, u_last]`;
/// `build_algorithmic(&mut cs, true, layout, 0.0, PI)?`.
pub(super) fn build_algorithmic(
    result: &mut CosAndSin,
    quasi_angular: bool,
    layout: SpanLayout,
    u_first: f64,
    u_last: f64,
) -> Result<(), Failure> {
    let alpha: f64 = (u_last - u_first) * 0.5e0;
    let beta: f64 = (u_last + u_first) * 0.5e0;
    let mut flat_knots: Array1<'static, f64> = symmetric_flat_knots(layout, alpha)?;
    set_end_knots(result, layout, u_first, u_last);
    let curve: EvalCurve = if quasi_angular {
        quasi_angular_curve(alpha)?
    } else {
        rational_c1_curve(result, &mut flat_knots, layout, u_first, alpha)?
    };
    algorithmic_cos_and_sin(result, &flat_knots, &curve)?;
    rotate(result, layout.num_poles, beta.cos(), beta.sin());
    Ok(())
}

// Clamped flat knots on [-alpha, alpha]; the interior entries are set per branch.
fn symmetric_flat_knots(layout: SpanLayout, alpha: f64) -> Result<Array1<'static, f64>, Failure> {
    let mut flat_knots: Array1<'static, f64> = one_based(layout.num_poles + layout.order)?;
    for ii in 1..=layout.order {
        flat_knots[ii] = -alpha;
        flat_knots[ii + layout.num_poles] = alpha;
    }
    Ok(flat_knots)
}

fn set_end_knots(result: &mut CosAndSin, layout: SpanLayout, u_first: f64, u_last: f64) {
    result.knots[1] = u_first;
    result.knots[layout.num_knots] = u_last;
    result.mults[1] = layout.order;
    result.mults[layout.num_knots] = layout.order;
}

fn eval_curve(degree: i32, evaluator: CosAndSinEvalFunction) -> Result<EvalCurve, Failure> {
    Ok(EvalCurve {
        degree,
        poles: one_based(4)?,
        knots: one_based(3)?,
        mults: one_based(3)?,
        evaluator,
    })
}

// V(t) = t + c t^3 in X and U(t) = 1 + b t^2 in Y, with b from the half-angle
// gamma = alpha / 2 and c = 1/3 + b.
fn quasi_angular_curve(alpha: f64) -> Result<EvalCurve, Failure> {
    let alpha_2: f64 = alpha * 0.5e0;
    let mut p_param: f64 = -1.0e0 / (alpha_2 * alpha_2);
    if alpha_2 < PI * 0.5e0 {
        if alpha_2 < 1.0e-7 {
            // Fixed degenerate case, when obtain 0 / 0 uncertainty.
            // According to Taylor approximation: b (gamma) = -6.0 / 15.0 + o(gamma^2)
            p_param = -6.0 / 15.0;
        } else {
            let tan_alpha_2: f64 = alpha_2.tan();
            let value1: f64 = 3.0e0 * (tan_alpha_2 - alpha_2);
            p_param += alpha_2 / value1;
        }
    }
    let q_param: f64 = (1.0e0 / 3.0e0) + p_param;
    let mut curve: EvalCurve = eval_curve(3, cos_and_sin_quasi_angular)?;
    curve.poles[1] = Point2::new(0.0e0, 1.0e0);
    curve.poles[2] = Point2::new(1.0e0, 0.0e0);
    curve.poles[3] = Point2::new(0.0e0, p_param);
    curve.poles[4] = Point2::new(q_param, 0.0e0);
    Ok(curve)
}

// A quadratic (U, V) with one interior knot, whose U² + V² has zero slope at both ends.
fn rational_c1_curve(
    result: &mut CosAndSin,
    flat_knots: &mut Array1<'static, f64>,
    layout: SpanLayout,
    u_first: f64,
    alpha: f64,
) -> Result<EvalCurve, Failure> {
    for ii in layout.order + 1..=layout.num_poles {
        flat_knots[ii] = 0.0e0;
    }
    result.knots[2] = u_first + alpha;
    result.mults[2] = result.degree - 1;
    let alpha_4: f64 = alpha * 0.25e0;
    let tan_alpha_2: f64 = (alpha * 0.5e0).tan();
    let middle: f64 = 1.0e0 + alpha_4 * tan_alpha_2;
    let mut curve: EvalCurve = eval_curve(2, cos_and_sin_rational_c1)?;
    curve.poles[1] = Point2::new(-tan_alpha_2, 1.0e0);
    curve.poles[2] = Point2::new(alpha_4 - tan_alpha_2, middle);
    curve.poles[3] = Point2::new(-alpha_4 + tan_alpha_2, middle);
    curve.poles[4] = Point2::new(tan_alpha_2, 1.0e0);
    let (knots, mults) = (&mut curve.knots, &mut curve.mults);
    (knots[1], knots[2], knots[3]) = (-alpha, 0.0e0, alpha);
    (mults[1], mults[2], mults[3]) = (3, 1, 3);
    Ok(curve)
}

// CosAndSinRationalC1: the B-spline (U, V) at the parameter.
fn cos_and_sin_rational_c1(
    parameter: f64,
    eval_degree: i32,
    eval_poles: &Array1<'_, Point2>,
    eval_knots: &Array1<'_, f64>,
    eval_mults: Option<&Array1<'_, i32>>,
) -> Result<[f64; 2], Failure> {
    let span = CurveSpan {
        u: parameter,
        index: 0,
        degree: eval_degree,
        periodic: false,
        knots: eval_knots,
        mults: eval_mults,
    };
    let point: Point2 = bsplclib::d0(&span, eval_poles, None)?;
    Ok([point.x(), point.y()])
}

// CosAndSinQuasiAngular: the polynomial (V, U) with canonical coefficients at t / 2.
fn cos_and_sin_quasi_angular(
    parameter: f64,
    eval_degree: i32,
    eval_poles: &Array1<'_, Point2>,
    _eval_knots: &Array1<'_, f64>,
    _eval_mults: Option<&Array1<'_, i32>>,
) -> Result<[f64; 2], Failure> {
    if eval_poles.len() as i64 != i64::from(eval_degree) + 1 {
        return Err(Failure::out_of_range(&format!(
            "CosAndSinQuasiAngular: EvalPoles size mismatch: {} poles for degree {eval_degree}",
            eval_poles.len()
        )));
    }
    let coefficients: Vec<f64> = eval_poles.iter().flat_map(|p| [p.x(), p.y()]).collect();
    let mut values: [f64; 2] = [0.0; 2];
    let param: f64 = parameter * 0.5e0;
    plib::no_derivative_eval_polynomial(
        param,
        eval_degree,
        2,
        eval_degree << 1,
        &coefficients,
        &mut values,
    )?;
    Ok(values)
}

// AlgorithmicCosAndSin: interpolates the homogeneous (cos, sin, 1) * (U² + V²).
fn algorithmic_cos_and_sin(
    result: &mut CosAndSin,
    flat_knots: &Array1<'_, f64>,
    curve: &EvalCurve,
) -> Result<(), Failure> {
    let order: i32 = result.degree + 1;
    let num_poles: i32 = flat_knots.len() as i32 - order;
    let lengths = [
        &result.cos_numerator,
        &result.sin_numerator,
        &result.denominator,
    ];
    if lengths.iter().any(|array| array.len() as i32 != num_poles) {
        return Err(construction(&format!(
            "AlgorithmicCosAndSin: {num_poles} poles expected, arrays of {} values",
            result.cos_numerator.len()
        )));
    }
    let mut parameters: Array1<'static, f64> = one_based(num_poles)?;
    bsplclib::build_schoenberg_points(result.degree, flat_knots, &mut parameters)?;
    let mut poles: Array1<'static, Point3> = homogeneous_samples(&parameters, curve)?;
    let mut contacts: Array1<'static, i32> = one_based(num_poles)?;
    contacts.init(0);
    // The native call ignores the returned pivot index.
    let mut pivot_index_problem: i32 = 0;
    bsplclib::interpolate(
        result.degree,
        flat_knots,
        &parameters,
        &contacts,
        &mut poles,
        &mut pivot_index_problem,
    )?;
    store_homogeneous(result, &poles);
    Ok(())
}

fn homogeneous_samples(
    parameters: &Array1<'_, f64>,
    curve: &EvalCurve,
) -> Result<Array1<'static, Point3>, Failure> {
    let mut poles: Array1<'static, Point3> = one_based(parameters.len() as i32)?;
    for ii in parameters.lower()..=parameters.upper() {
        // result[0] is the X coordinate of the auxiliary curve, result[1] the Y one.
        let [first, second] = (curve.evaluator)(
            parameters[ii],
            curve.degree,
            &curve.poles,
            &curve.knots,
            Some(&curve.mults),
        )?;
        poles[ii] = Point3::new(
            second * second - first * first,
            2.0e0 * second * first,
            second * second + first * first,
        );
    }
    Ok(poles)
}

fn store_homogeneous(result: &mut CosAndSin, poles: &Array1<'_, Point3>) {
    for ii in 1..=result.cos_numerator.len() as i32 {
        let inverse: f64 = 1.0e0 / poles[ii].z();
        result.cos_numerator[ii] = poles[ii].x() * inverse;
        result.sin_numerator[ii] = poles[ii].y() * inverse;
        result.denominator[ii] = poles[ii].z();
    }
}

// Rotates the arc from [-alpha, alpha] to [u_first, u_last] by the bisector angle.
fn rotate(result: &mut CosAndSin, num_poles: i32, cos_beta: f64, sin_beta: f64) {
    for ii in 1..=num_poles {
        let (cos, sin) = (result.cos_numerator[ii], result.sin_numerator[ii]);
        result.cos_numerator[ii] = cos_beta * cos - sin_beta * sin;
        result.sin_numerator[ii] = sin_beta * cos + cos_beta * sin;
    }
}
