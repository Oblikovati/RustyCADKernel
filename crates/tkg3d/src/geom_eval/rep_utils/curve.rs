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
use crate::geom_eval::rep_curve_desc::{
    RepCurveDescDerivBounded, RepCurveDescFull, RepCurveDescMapped,
};
use crate::geom_eval::{Map1d, RepCurveDesc};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d_eval::RepCurveKind;
use rustycad_tkmath::gp::{Point3, Vector3};

/// Derivative of order `n` through the descriptor, scaled by `Scale^n` computed with
/// `std::pow` as natively; `None` for `n < 1` (`TryEvalCurveDN`);
/// `try_eval_curve_dn(&desc, 0.5, 2)?`.
// Unlike Geom2dEval_RepUtils, which multiplies the scale N times, the 3D
// TryEvalCurveDN scales by pow(Scale, N), so this member is not shared.
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

rustycad_tkg2d::define_rep_curve_evaluation!(
    dyn GeomCurve,
    Point3,
    GeomCurveD1,
    GeomCurveD2,
    GeomCurveD3,
    "GeomEval_RepUtils"
);
