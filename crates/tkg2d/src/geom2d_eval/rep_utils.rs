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
// Rust translation of ModelingData/TKG2d/Geom2dEval/Geom2dEval_RepUtils.pxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2dEval_RepUtils`: validation, cloning and evaluation through a descriptor.
//!
//! The `try_eval_*` functions return `Ok(None)` where the native ones return false
//! (no descriptor, derivative order above the limit, parameter outside the mapped
//! domain), so the owner falls back to its own evaluation.
use super::rep_curve_desc::{
    Map1d, RepCurveDesc, RepCurveDescDerivBounded, RepCurveDescFull, RepCurveDescMapped,
    RepCurveKind,
};
use crate::geom2d::errors;
use crate::geom2d::{Geom2dCurve, Geom2dCurveD1, Geom2dCurveD2, Geom2dCurveD3};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{Point2, Vector2};

/// Reject a descriptor with a null representation, an invalid map, or a
/// representation that is the owner itself (`ValidateCurveDesc`);
/// `validate_curve_desc(&desc, std::ptr::from_ref(self).cast())?`.
pub fn validate_curve_desc(
    desc: &Handle<dyn RepCurveDesc>,
    owner: *const (),
) -> Result<(), Failure> {
    let Some(desc) = desc.get() else {
        return Ok(());
    };
    let representation: Handle<dyn Geom2dCurve> = desc.representation();
    if representation.is_null() {
        return Err(errors::program(
            "SetEvalRepresentation: null Representation",
        ));
    }
    let mapping = desc
        .mapping()
        .filter(|_| desc.kind() == RepCurveKind::Mapped);
    if let Some((_, map)) = mapping.filter(|(_, map)| !map.is_valid()) {
        return Err(errors::program(&format!(
            "SetEvalRepresentation: invalid parameter map, Scale = {}; expected |Scale| > Precision::Confusion()",
            map.scale
        )));
    }
    if representation.object_address() == owner {
        return Err(errors::program(
            "SetEvalRepresentation: invalid descriptor chain; the representation is the curve itself",
        ));
    }
    Ok(())
}

/// Derivative of order `n` through the descriptor; `None` for `n < 1` (`TryEvalCurveDN`);
/// `try_eval_curve_dn(&desc, 0.5, 2)?`.
// The scale is multiplied N times here, while GeomEval_RepUtils uses pow(Scale, N),
// so this member is not shared with GeomEval.
pub fn try_eval_curve_dn(
    desc: &Handle<dyn RepCurveDesc>,
    u: f64,
    n: i32,
) -> Result<Option<Vector2>, Failure> {
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
        |vector, map| {
            let mut scale_n: f64 = 1.0;
            for _ in 0..n {
                scale_n *= map.scale;
            }
            vector * scale_n
        },
    )
}

/// The curve half of a package's RepUtils (`CloneCurveDesc` and `TryEvalCurveD0`..
/// `TryEvalCurveD3`), which Geom2dEval and GeomEval write alike over their own curve
/// and descriptor types; `errors`, `Map1d`, `RepCurveDesc`, `RepCurveKind`, the three
/// descriptor classes, `Handle` and `Failure` must be in scope;
/// `rustycad_tkg2d::define_rep_curve_evaluation!(dyn GeomCurve, Point3, GeomCurveD1, GeomCurveD2, GeomCurveD3, "GeomEval_RepUtils");`.
#[macro_export]
macro_rules! define_rep_curve_evaluation {
    ($curve:ty, $point:ty, $d1:ident, $d2:ident, $d3:ident, $utils:literal) => {
        /// Deep copy with a copied representation; null when there is nothing to copy
        /// (`CloneCurveDesc`); `let copy = clone_curve_desc(&desc)?;`.
        pub fn clone_curve_desc(
            desc: &Handle<dyn RepCurveDesc>,
        ) -> Result<Handle<dyn RepCurveDesc>, Failure> {
            let Some(source) = desc.get() else {
                return Ok(Handle::default());
            };
            let representation: Handle<$curve> = source.representation();
            let Some(curve) = representation.get() else {
                return Ok(Handle::default());
            };
            let copy: Handle<$curve> = curve.copy()?.cast();
            if copy.is_null() {
                return Ok(Handle::default());
            }
            Ok(
                match (source.kind(), source.max_deriv_order(), source.mapping()) {
                    (RepCurveKind::DerivBounded, Some(order), _) => {
                        Handle::new(RepCurveDescDerivBounded::new(copy, order)).cast()
                    }
                    (RepCurveKind::Mapped, Some(order), Some((domain, map))) => {
                        Handle::new(RepCurveDescMapped::new(copy, order, domain, map)).cast()
                    }
                    _ => Handle::new(RepCurveDescFull::new(copy)).cast(),
                },
            )
        }

        // Shared body of the TryEvalCurve* functions: order limit, domain, parameter map.
        fn try_eval<T>(
            desc: &Handle<dyn RepCurveDesc>,
            order: usize,
            u: f64,
            evaluate: impl FnOnce(&$curve, f64) -> Result<T, Failure>,
            scale: impl FnOnce(T, &Map1d) -> T,
        ) -> Result<Option<T>, Failure> {
            let Some(desc) = desc.get() else {
                return Ok(None);
            };
            if desc.max_deriv_order().is_some_and(|limit| limit < order) {
                return Ok(None);
            }
            let representation: Handle<$curve> = desc.representation();
            let Some(curve) = representation.get() else {
                return Err(errors::program(concat!($utils, ": null Representation")));
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
        ) -> Result<Option<$point>, Failure> {
            try_eval(desc, 0, u, |curve, u| curve.eval_d0(u), |point, _| point)
        }

        /// Point and D1 through the descriptor, D1 scaled by the map (`TryEvalCurveD1`);
        /// `try_eval_curve_d1(&desc, 0.5)?`.
        pub fn try_eval_curve_d1(
            desc: &Handle<dyn RepCurveDesc>,
            u: f64,
        ) -> Result<Option<$d1>, Failure> {
            try_eval(
                desc,
                1,
                u,
                |curve, u| curve.eval_d1(u),
                |result, map| {
                    let scale: f64 = map.scale;
                    $d1 {
                        point: result.point,
                        d1: result.d1 * scale,
                    }
                },
            )
        }

        /// Point and two derivatives through the descriptor (`TryEvalCurveD2`);
        /// `try_eval_curve_d2(&desc, 0.5)?`.
        pub fn try_eval_curve_d2(
            desc: &Handle<dyn RepCurveDesc>,
            u: f64,
        ) -> Result<Option<$d2>, Failure> {
            try_eval(
                desc,
                2,
                u,
                |curve, u| curve.eval_d2(u),
                |result, map| {
                    let scale: f64 = map.scale;
                    let scale2: f64 = scale * scale;
                    $d2 {
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
        ) -> Result<Option<$d3>, Failure> {
            try_eval(
                desc,
                3,
                u,
                |curve, u| curve.eval_d3(u),
                |result, map| {
                    let scale: f64 = map.scale;
                    let scale2: f64 = scale * scale;
                    let scale3: f64 = scale2 * scale;
                    $d3 {
                        point: result.point,
                        d1: result.d1 * scale,
                        d2: result.d2 * scale2,
                        d3: result.d3 * scale3,
                    }
                },
            )
        }
    };
}

define_rep_curve_evaluation!(
    dyn Geom2dCurve,
    Point2,
    Geom2dCurveD1,
    Geom2dCurveD2,
    Geom2dCurveD3,
    "Geom2dEval_RepUtils"
);
