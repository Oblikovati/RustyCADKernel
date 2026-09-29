// Copyright (c) 1993-1999 Matra Datavision
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
// Rust translation of ModelingData/TKG2d/Geom2d/Geom2d_BezierCurve.cxx, Geom2d_BSplineCurve.cxx, Geom2d_BSplineCurve_1.cxx (PeriodicNormalization, RemoveKnot index check, closure test, evaluation representation) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The Geom2d members of the shared spline curve states: the evaluation
//! representation of Geom2dEval, the loops of `PeriodicNormalization`, the
//! `RemoveKnot` index range and the gp::Resolution closure test; B-spline edits
//! never clear the evaluation representation.
use super::arrays::at;
use super::bezier_curve_core::{BezierCurveCore, BezierCurveKind};
use super::bspline_curve_core::{BSplineCurveCore, BSplineCurveKind};
use super::errors;
use super::spline_curve_kind::{KindD1, KindD2, KindD3, SplineCurveKind};
use crate::geom2d_eval::{RepCurveDesc, rep_utils};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{Point2, Vector2, resolution};

/// The Geom2d variant of the spline curve kinds.
#[derive(Clone, Copy, Debug)]
pub(super) struct Geom2dSplineKind;

/// The fields of Geom2d_BSplineCurve.
pub(super) type BSplineState = BSplineCurveCore<Geom2dSplineKind>;
/// The fields of Geom2d_BezierCurve.
pub(super) type BezierState = BezierCurveCore<Geom2dSplineKind>;

type RepHandle = Handle<dyn RepCurveDesc>;

impl SplineCurveKind for Geom2dSplineKind {
    type Point = Point2;
    type EvalRep = RepHandle;
    fn undefined_derivative(message: &str) -> Failure {
        errors::undefined_derivative(message)
    }
    fn rep_d0(eval_rep: &RepHandle, u: f64) -> Result<Option<Point2>, Failure> {
        rep_utils::try_eval_curve_d0(eval_rep, u)
    }
    fn rep_d1(eval_rep: &RepHandle, u: f64) -> Result<Option<KindD1<Self>>, Failure> {
        Ok(rep_utils::try_eval_curve_d1(eval_rep, u)?.map(|r| (r.point, r.d1)))
    }
    fn rep_d2(eval_rep: &RepHandle, u: f64) -> Result<Option<KindD2<Self>>, Failure> {
        Ok(rep_utils::try_eval_curve_d2(eval_rep, u)?.map(|r| (r.point, r.d1, r.d2)))
    }
    fn rep_d3(eval_rep: &RepHandle, u: f64) -> Result<Option<KindD3<Self>>, Failure> {
        Ok(rep_utils::try_eval_curve_d3(eval_rep, u)?.map(|r| (r.point, r.d1, r.d2, r.d3)))
    }
    fn rep_dn(eval_rep: &RepHandle, u: f64, n: i32) -> Result<Option<Vector2>, Failure> {
        rep_utils::try_eval_curve_dn(eval_rep, u, n)
    }
}

impl BezierCurveKind for Geom2dSplineKind {
    const BEZIER_CLASS: &'static str = "Geom2d_BezierCurve";
    fn point_distance(first: Point2, last: Point2) -> f64 {
        first.distance(last)
    }
    fn closure_tolerance() -> f64 {
        resolution()
    }
}

impl BSplineCurveKind for Geom2dSplineKind {
    const BSPLINE_CLASS: &'static str = "Geom2d_BSplineCurve";
    // Geom2d edits keep the evaluation representation.
    fn clear_eval_rep_on_edit(_eval_rep: &mut RepHandle) {}
    // Whole periods are subtracted or added one at a time.
    fn periodic_normalization(state: &BSplineState, parameter: f64) -> Result<f64, Failure> {
        if !state.periodic {
            return Ok(parameter);
        }
        let upper: f64 = at(&state.flat_knots, state.flat_knots.upper() - state.deg)?;
        let lower: f64 = at(&state.flat_knots, state.deg + 1)?;
        let period: f64 = upper - lower;
        let mut value: f64 = parameter;
        while value > upper {
            value -= period;
        }
        while value < lower {
            value += period;
        }
        Ok(value)
    }
    // Any used knot, the first and last included.
    fn check_removable_index(state: &BSplineState, index: i32) -> Result<(), Failure> {
        let (i1, i2) = (state.first_u_knot_index()?, state.last_u_knot_index()?);
        if index < i1 || index > i2 {
            return Err(errors::out_of_range(&format!(
                "BSpline curve: RemoveKnot: index {index} out of range; expected in [{i1}, {i2}]"
            )));
        }
        Ok(())
    }
    fn tangents_agree(v1: Vector2, v2: Vector2, ang_tol: f64) -> Result<bool, Failure> {
        if v1.square_magnitude() <= resolution() || v2.square_magnitude() <= resolution() {
            return Ok(false);
        }
        Ok(v1.angle(v2)?.abs() <= ang_tol)
    }
}
