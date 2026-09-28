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
// Rust translation of ModelingData/TKG3d/GeomEval/GeomEval_RepUtils.pxx (TryEvalCurve*) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Curve evaluation through a descriptor, with the parameter map applied to the
//! derivatives.
use crate::geom::errors;
use crate::geom::{GeomCurve, GeomCurveD1, GeomCurveD2, GeomCurveD3};
use crate::geom_eval::{Map1d, RepCurveDesc};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{Point3, Vector3};

// Shared body of the TryEvalCurve* functions: order limit, domain, parameter map.
fn try_eval<T>(
    desc: &Handle<dyn RepCurveDesc>,
    order: usize,
    u: f64,
    evaluate: impl FnOnce(&dyn GeomCurve, f64) -> Result<T, Failure>,
    scale: impl FnOnce(T, &Map1d) -> T,
) -> Result<Option<T>, Failure> {
    let Some(desc) = desc.get() else {
        return Ok(None);
    };
    if desc.max_deriv_order().is_some_and(|limit| limit < order) {
        return Ok(None);
    }
    let representation: Handle<dyn GeomCurve> = desc.representation();
    let Some(curve) = representation.get() else {
        return Err(errors::program("GeomEval_RepUtils: null Representation"));
    };
    let Some((domain, map)) = desc.mapping() else {
        return evaluate(curve, u).map(Some);
    };
    let u_rep: f64 = map.map(u);
    if domain.is_some_and(|domain| !domain.contains(u_rep)) {
        return Ok(None);
    }
    Ok(Some(scale(evaluate(curve, u_rep)?, &map)))
}

/// Point through the descriptor (`TryEvalCurveD0`); `try_eval_curve_d0(&desc, 0.5)?`.
pub fn try_eval_curve_d0(
    desc: &Handle<dyn RepCurveDesc>,
    u: f64,
) -> Result<Option<Point3>, Failure> {
    try_eval(desc, 0, u, |curve, u| curve.eval_d0(u), |point, _| point)
}

/// Point and D1 through the descriptor, D1 scaled by the map (`TryEvalCurveD1`);
/// `try_eval_curve_d1(&desc, 0.5)?`.
pub fn try_eval_curve_d1(
    desc: &Handle<dyn RepCurveDesc>,
    u: f64,
) -> Result<Option<GeomCurveD1>, Failure> {
    try_eval(
        desc,
        1,
        u,
        |curve, u| curve.eval_d1(u),
        |result, map| GeomCurveD1 {
            point: result.point,
            d1: result.d1 * map.scale,
        },
    )
}

/// Point and two derivatives through the descriptor (`TryEvalCurveD2`);
/// `try_eval_curve_d2(&desc, 0.5)?`.
pub fn try_eval_curve_d2(
    desc: &Handle<dyn RepCurveDesc>,
    u: f64,
) -> Result<Option<GeomCurveD2>, Failure> {
    try_eval(
        desc,
        2,
        u,
        |curve, u| curve.eval_d2(u),
        |result, map| {
            let scale: f64 = map.scale;
            let scale2: f64 = scale * scale;
            GeomCurveD2 {
                point: result.point,
                d1: result.d1 * scale,
                d2: result.d2 * scale2,
            }
        },
    )
}

/// Point and three derivatives through the descriptor (`TryEvalCurveD3`);
/// `try_eval_curve_d3(&desc, 0.5)?`.
pub fn try_eval_curve_d3(
    desc: &Handle<dyn RepCurveDesc>,
    u: f64,
) -> Result<Option<GeomCurveD3>, Failure> {
    try_eval(
        desc,
        3,
        u,
        |curve, u| curve.eval_d3(u),
        |result, map| {
            let scale: f64 = map.scale;
            let scale2: f64 = scale * scale;
            let scale3: f64 = scale2 * scale;
            GeomCurveD3 {
                point: result.point,
                d1: result.d1 * scale,
                d2: result.d2 * scale2,
                d3: result.d3 * scale3,
            }
        },
    )
}

/// Derivative of order `n` through the descriptor, scaled by `Scale^n` computed with
/// `std::pow` as natively; `None` for `n < 1` (`TryEvalCurveDN`);
/// `try_eval_curve_dn(&desc, 0.5, 2)?`.
pub fn try_eval_curve_dn(
    desc: &Handle<dyn RepCurveDesc>,
    u: f64,
    n: i32,
) -> Result<Option<Vector3>, Failure> {
    let Ok(order) = usize::try_from(n) else {
        return Ok(None);
    };
    if order < 1 {
        return Ok(None);
    }
    try_eval(
        desc,
        order,
        u,
        |curve, u| curve.eval_dn(u, n),
        |vector, map| vector * map.scale.powf(f64::from(n)),
    )
}
