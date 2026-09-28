// Copyright (c) 1995-1999 Matra Datavision
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
// Rust translation of FoundationClasses/TKMath/BSplCLib/BSplCLib_2.cxx (PrepareEval, D0..DN on reals) and BSplCLib_CurveComputation.pxx (PrepareEval_T, BSplCLib_D0..BSplCLib_DN) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Point and derivative evaluation of 1D, 2D and 3D B-spline curves.
//!
//! The span poles are copied to a local buffer (homogeneous when the weights of
//! the span truly differ), then evaluated with the Boor or Bohm scheme.
use super::MAX_DEGREE;
use super::bohm::bohm;
use super::build::{build_eval, build_eval_scalar, build_knots};
use super::forms::is_rational;
use super::knots::pole_index;
use super::locate::locate_parameter_for_eval;
use super::point::{CurveD3, CurvePoint};
use super::scheme::eval;
use crate::plib::rational_derivative;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;

/// Parameter, knots and multiplicities shared by the evaluation overloads.
#[derive(Clone, Copy)]
pub struct CurveSpan<'r, 'a> {
    /// Parameter to evaluate.
    pub u: f64,
    /// Knot index hint; out-of-range values are located again.
    pub index: i32,
    /// Degree of the curve.
    pub degree: i32,
    /// Periodicity of the curve.
    pub periodic: bool,
    /// Knots (flat knots when `mults` is None).
    pub knots: &'r Array1<'a, f64>,
    /// Multiplicities, or None for flat knots.
    pub mults: Option<&'r Array1<'a, i32>>,
}

// The local evaluation buffers (BSplCLib_DataContainer).
pub(crate) struct LocalSpan {
    pub(crate) u: f64,
    pub(crate) dim: usize,
    pub(crate) rational: bool,
    pub(crate) knots: Vec<f64>,
    pub(crate) poles: Vec<f64>,
}

/// Raises the native Standard_OutOfRange for degrees above MaxDegree
/// (`validateBSplineDegree`); `validate_degree(3)?`.
pub(crate) fn validate_degree(degree: i32) -> Result<(), Failure> {
    if degree > MAX_DEGREE {
        return Err(Failure::out_of_range(
            "BSplCLib: bspline degree is greater than maximum supported",
        ));
    }
    Ok(())
}

// PrepareEval: locates the span, builds its knots and (homogeneous) poles.
pub(crate) fn prepare_eval(
    span: &CurveSpan<'_, '_>,
    weights: Option<&Array1<'_, f64>>,
    dimension: usize,
    build: impl FnOnce(i32, Option<&Array1<'_, f64>>, &mut [f64]) -> Result<(), Failure>,
) -> Result<LocalSpan, Failure> {
    validate_degree(span.degree)?;
    let (mut u, mut index): (f64, i32) = (span.u, span.index);
    let (degree, knots): (i32, &Array1<'_, f64>) = (span.degree, span.knots);
    locate_parameter_for_eval(
        degree,
        knots,
        span.mults,
        u,
        span.periodic,
        &mut index,
        &mut u,
    )?;
    let max: usize = MAX_DEGREE as usize;
    let mut local = LocalSpan {
        u,
        dim: dimension,
        rational: false,
        knots: vec![0.0; 2 * max],
        poles: vec![0.0; (max + 1) * (dimension + 1)],
    };
    build_knots(
        degree,
        index,
        span.periodic,
        knots,
        span.mults,
        &mut local.knots,
    )?;
    index = match span.mults {
        None => index - (knots.lower() + degree),
        Some(mults) => pole_index(degree, index, span.periodic, mults)?,
    };
    if let Some(weights) = weights {
        let w_lower: i32 = weights.lower() + index;
        local.rational = is_rational(weights, w_lower, w_lower + degree, 0.0)?;
    }
    if local.rational {
        local.dim = dimension + 1;
        build(index, weights, &mut local.poles)?;
    } else {
        build(index, None, &mut local.poles)?;
    }
    Ok(local)
}

fn prepare_scalar(
    span: &CurveSpan<'_, '_>,
    poles: &Array1<'_, f64>,
    weights: Option<&Array1<'_, f64>>,
) -> Result<LocalSpan, Failure> {
    let degree: i32 = span.degree;
    prepare_eval(span, weights, 1, |index, weights, lp| {
        build_eval_scalar(degree, index, poles, weights, lp)
    })
}

fn prepare_points<P: CurvePoint>(
    span: &CurveSpan<'_, '_>,
    poles: &Array1<'_, P>,
    weights: Option<&Array1<'_, f64>>,
) -> Result<LocalSpan, Failure> {
    let degree: i32 = span.degree;
    prepare_eval(span, weights, P::DIMENSION, |index, weights, lp| {
        build_eval(degree, index, poles, weights, lp)
    })
}

// Runs Bohm to order `n` and returns the (rational) derivative rows.
fn derivatives(
    local: &mut LocalSpan,
    degree: i32,
    n: i32,
    dimension: usize,
) -> Result<Vec<f64>, Failure> {
    bohm(
        local.u,
        degree,
        n,
        &local.knots,
        local.dim as i32,
        &mut local.poles,
    )?;
    if !local.rational {
        return Ok(local.poles.clone());
    }
    let mut ders: Vec<f64> = vec![0.0; dimension * 4];
    rational_derivative(degree, n, dimension as i32, &local.poles, &mut ders, true)?;
    Ok(ders)
}

/// Value of a 1D B-spline (`BSplCLib::D0` on reals);
/// `bsplclib::d0_scalar(&span, &poles, None)?`.
pub fn d0_scalar(
    span: &CurveSpan<'_, '_>,
    poles: &Array1<'_, f64>,
    weights: Option<&Array1<'_, f64>>,
) -> Result<f64, Failure> {
    let mut local: LocalSpan = prepare_scalar(span, poles, weights)?;
    eval(
        local.u,
        span.degree,
        &local.knots,
        local.dim as i32,
        &mut local.poles,
    )?;
    if local.rational {
        return Ok(local.poles[0] / local.poles[1]);
    }
    Ok(local.poles[0])
}

/// Value and first derivative of a 1D B-spline (`BSplCLib::D1` on reals);
/// `bsplclib::d1_scalar(&span, &poles, None)?`.
pub fn d1_scalar(
    span: &CurveSpan<'_, '_>,
    poles: &Array1<'_, f64>,
    weights: Option<&Array1<'_, f64>>,
) -> Result<(f64, f64), Failure> {
    let mut local: LocalSpan = prepare_scalar(span, poles, weights)?;
    let result: Vec<f64> = derivatives(&mut local, span.degree, 1, 1)?;
    Ok((result[0], result[1]))
}

/// Value and two derivatives of a 1D B-spline (`BSplCLib::D2` on reals); a
/// non-rational curve of degree below 2 gives a zero second derivative;
/// `bsplclib::d2_scalar(&span, &poles, None)?`.
pub fn d2_scalar(
    span: &CurveSpan<'_, '_>,
    poles: &Array1<'_, f64>,
    weights: Option<&Array1<'_, f64>>,
) -> Result<(f64, f64, f64), Failure> {
    let mut local: LocalSpan = prepare_scalar(span, poles, weights)?;
    let result: Vec<f64> = derivatives(&mut local, span.degree, 2, 1)?;
    let v2: f64 = higher(&local, span.degree, 2, &result[2..3])[0];
    Ok((result[0], result[1], v2))
}

/// Value and three derivatives of a 1D B-spline (`BSplCLib::D3` on reals);
/// `bsplclib::d3_scalar(&span, &poles, None)?`.
pub fn d3_scalar(
    span: &CurveSpan<'_, '_>,
    poles: &Array1<'_, f64>,
    weights: Option<&Array1<'_, f64>>,
) -> Result<(f64, f64, f64, f64), Failure> {
    let mut local: LocalSpan = prepare_scalar(span, poles, weights)?;
    let result: Vec<f64> = derivatives(&mut local, span.degree, 3, 1)?;
    let v2: f64 = higher(&local, span.degree, 2, &result[2..3])[0];
    let v3: f64 = higher(&local, span.degree, 3, &result[3..4])[0];
    Ok((result[0], result[1], v2, v3))
}

// Derivative `order` or zeros when a non-rational curve has a lower degree.
fn higher(local: &LocalSpan, degree: i32, order: i32, row: &[f64]) -> Vec<f64> {
    if !local.rational && degree < order {
        return vec![0.0; row.len()];
    }
    row.to_vec()
}

/// N-th derivative of a 1D B-spline (`BSplCLib::DN` on reals);
/// `bsplclib::dn_scalar(&span, 2, &poles, None)?`.
pub fn dn_scalar(
    span: &CurveSpan<'_, '_>,
    n: i32,
    poles: &Array1<'_, f64>,
    weights: Option<&Array1<'_, f64>>,
) -> Result<f64, Failure> {
    let mut local: LocalSpan = prepare_scalar(span, poles, weights)?;
    Ok(nth_derivative(&mut local, span.degree, n, 1)?[0])
}

fn nth_derivative(
    local: &mut LocalSpan,
    degree: i32,
    n: i32,
    dimension: usize,
) -> Result<Vec<f64>, Failure> {
    bohm(
        local.u,
        degree,
        n,
        &local.knots,
        local.dim as i32,
        &mut local.poles,
    )?;
    if local.rational {
        let mut v: Vec<f64> = vec![0.0; dimension];
        rational_derivative(degree, n, dimension as i32, &local.poles, &mut v, false)?;
        return Ok(v);
    }
    if n > degree {
        return Ok(vec![0.0; dimension]);
    }
    let start: usize = usize::try_from(n)
        .map_err(|_| Failure::out_of_range(&format!("BSplCLib::DN: N = {n}; expected >= 0")))?
        * dimension;
    Ok(local.poles[start..start + dimension].to_vec())
}

/// Point of a 2D/3D B-spline (`BSplCLib::D0` on gp_Pnt/gp_Pnt2d);
/// `bsplclib::d0(&span, &poles, None)?`.
pub fn d0<P: CurvePoint>(
    span: &CurveSpan<'_, '_>,
    poles: &Array1<'_, P>,
    weights: Option<&Array1<'_, f64>>,
) -> Result<P, Failure> {
    let mut local: LocalSpan = prepare_points(span, poles, weights)?;
    eval(
        local.u,
        span.degree,
        &local.knots,
        local.dim as i32,
        &mut local.poles,
    )?;
    if !local.rational {
        return Ok(P::point_from(&local.poles));
    }
    let w: f64 = local.poles[P::DIMENSION];
    let scaled: Vec<f64> = local.poles[..P::DIMENSION]
        .iter()
        .map(|value| value / w)
        .collect();
    Ok(P::point_from(&scaled))
}

/// Point and first derivative of a 2D/3D B-spline (`BSplCLib::D1` on gp_Pnt/gp_Pnt2d);
/// `bsplclib::d1(&span, &poles, None)?`.
pub fn d1<P: CurvePoint>(
    span: &CurveSpan<'_, '_>,
    poles: &Array1<'_, P>,
    weights: Option<&Array1<'_, f64>>,
) -> Result<(P, P::Vector), Failure> {
    let mut local: LocalSpan = prepare_points(span, poles, weights)?;
    let d: usize = P::DIMENSION;
    let result: Vec<f64> = derivatives(&mut local, span.degree, 1, d)?;
    Ok((P::point_from(&result), P::vector_from(&result[d..])))
}

/// Point and two derivatives of a 2D/3D B-spline (`BSplCLib::D2` on gp_Pnt/gp_Pnt2d);
/// `bsplclib::d2(&span, &poles, None)?`.
pub fn d2<P: CurvePoint>(
    span: &CurveSpan<'_, '_>,
    poles: &Array1<'_, P>,
    weights: Option<&Array1<'_, f64>>,
) -> Result<(P, P::Vector, P::Vector), Failure> {
    let mut local: LocalSpan = prepare_points(span, poles, weights)?;
    let d: usize = P::DIMENSION;
    let result: Vec<f64> = derivatives(&mut local, span.degree, 2, d)?;
    let v2: Vec<f64> = higher(&local, span.degree, 2, &result[2 * d..3 * d]);
    Ok((
        P::point_from(&result),
        P::vector_from(&result[d..]),
        P::vector_from(&v2),
    ))
}

/// Point and three derivatives of a 2D/3D B-spline (`BSplCLib::D3` on gp_Pnt/gp_Pnt2d);
/// `bsplclib::d3(&span, &poles, None)?`.
pub fn d3<P: CurvePoint>(
    span: &CurveSpan<'_, '_>,
    poles: &Array1<'_, P>,
    weights: Option<&Array1<'_, f64>>,
) -> Result<CurveD3<P>, Failure> {
    let mut local: LocalSpan = prepare_points(span, poles, weights)?;
    let d: usize = P::DIMENSION;
    let result: Vec<f64> = derivatives(&mut local, span.degree, 3, d)?;
    let v2: Vec<f64> = higher(&local, span.degree, 2, &result[2 * d..3 * d]);
    let v3: Vec<f64> = higher(&local, span.degree, 3, &result[3 * d..4 * d]);
    Ok((
        P::point_from(&result),
        P::vector_from(&result[d..]),
        P::vector_from(&v2),
        P::vector_from(&v3),
    ))
}

/// N-th derivative of a 2D/3D B-spline (`BSplCLib::DN` on gp_Pnt/gp_Pnt2d);
/// `bsplclib::dn(&span, 2, &poles, None)?`.
pub fn dn<P: CurvePoint>(
    span: &CurveSpan<'_, '_>,
    n: i32,
    poles: &Array1<'_, P>,
    weights: Option<&Array1<'_, f64>>,
) -> Result<P::Vector, Failure> {
    let mut local: LocalSpan = prepare_points(span, poles, weights)?;
    let v: Vec<f64> = nth_derivative(&mut local, span.degree, n, P::DIMENSION)?;
    Ok(P::vector_from(&v))
}
