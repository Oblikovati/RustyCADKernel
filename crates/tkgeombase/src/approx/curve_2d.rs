// Copyright (c) 1997-1999 Matra Datavision
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
// Rust translation of ModelingData/TKGeomBase/Approx/Approx_Curve2d.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Approx_Curve2d`: the polynomial B-spline approximation of an adapted 2D curve as
//! two 1D functions (u and v) with their own tolerances.
use super::curve_3d::null_table;
use crate::curve_approx_eval::{Curve2dEvaluator, cut_points_2d};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::Geom2dBSplineCurve;
use rustycad_tkg2d::geom2d::arrays::one_based;
use rustycad_tkg3d::adv_approx::{AdvApproxApproxAFunction, AdvApproxPrefAndRec};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::Point2;

/// `Approx_Curve2d`: the B-spline and the greatest errors along u and v.
///
/// ```ignore
/// let approx = Curve2d::new(&adaptor, 0.0, 1.0, 1e-6, 1e-6, Shape::C2, 14, 16)?;
/// let bspline = approx.curve();
/// ```
#[derive(Clone)]
pub struct Curve2d {
    curve: Handle<Geom2dBSplineCurve>,
    is_done: bool,
    has_result: bool,
    max_error_2d_u: f64,
    max_error_2d_v: f64,
}

impl Curve2d {
    /// `Approx_Curve2d(C2D, First, Last, TolU, TolV, Continuity, MaxDegree,
    /// MaxSegments)`. The native constructor trims `C2D` to `[First, Last]` and
    /// discards the result (only its exceptions remain), then approximates the
    /// untrimmed adaptor on `[First, Last]`, cut at the C2 and C3 breaks of its whole
    /// range; `Curve2d::new(&c, 0.0, 1.0, 1e-6, 1e-6, Shape::C2, 14, 16)?`.
    #[allow(clippy::too_many_arguments)] // Mirrors the native eight-argument constructor.
    pub fn new(
        c2d: &Handle<dyn Adaptor2dCurve2d>,
        first: f64,
        last: f64,
        tol_u: f64,
        tol_v: f64,
        continuity: Shape,
        max_degree: i32,
        max_segments: i32,
    ) -> Result<Self, Failure> {
        let adaptor: &dyn Adaptor2dCurve2d =
            c2d.get().ok_or_else(|| null_table("Approx_Curve2d: C2D"))?;
        adaptor.trim(first, last, precision::p_confusion())?;
        let mut one_d_tol: Array1<'static, f64> = one_based(2)?;
        (one_d_tol[1], one_d_tol[2]) = (tol_u, tol_v);
        let cut_tool = AdvApproxPrefAndRec::new(
            &cut_points_2d(adaptor, Shape::C2)?,
            &cut_points_2d(adaptor, Shape::C3)?,
            5.0,
        )?;
        let mut evaluator =
            Curve2dEvaluator::new(c2d.clone(), [first, last], true, "Approx_Curve2d_Eval");
        let approx = AdvApproxApproxAFunction::with_cutting(
            2,
            0,
            0,
            Some(&one_d_tol),
            None,
            None,
            first,
            last,
            continuity,
            max_degree,
            max_segments,
            &mut evaluator,
            &cut_tool,
        );
        evaluator.take_failure()?;
        Self::from_approximation(&approx?)
    }

    fn from_approximation(approx: &AdvApproxApproxAFunction) -> Result<Self, Failure> {
        let mut result = Self {
            curve: Handle::default(),
            is_done: approx.is_done(),
            has_result: approx.has_result(),
            max_error_2d_u: 0.0,
            max_error_2d_v: 0.0,
        };
        if !result.has_result {
            return Ok(result);
        }
        let poles: Array1<'static, Point2> = poles_from_two_1d(approx, [1, 2])?;
        let knots = approx.knots().ok_or_else(|| null_table("Knots"))?;
        let mults = approx
            .multiplicities()
            .ok_or_else(|| null_table("Multiplicities"))?;
        result.curve = Handle::new(Geom2dBSplineCurve::new(
            &poles,
            knots,
            mults,
            approx.degree(),
            false,
        )?);
        result.max_error_2d_u = approx.max_error(1, 1)?;
        result.max_error_2d_v = approx.max_error(1, 2)?;
        Ok(result)
    }

    /// `IsDone()`; `approx.is_done()`.
    pub fn is_done(&self) -> bool {
        self.is_done
    }

    /// `HasResult()`; `approx.has_result()`.
    pub fn has_result(&self) -> bool {
        self.has_result
    }

    /// `Curve()`: null without a result; `approx.curve()`.
    pub fn curve(&self) -> Handle<Geom2dBSplineCurve> {
        self.curve.clone()
    }

    /// `MaxError2dU()`; `approx.max_error_2d_u()`.
    pub fn max_error_2d_u(&self) -> f64 {
        self.max_error_2d_u
    }

    /// `MaxError2dV()`; `approx.max_error_2d_v()`.
    pub fn max_error_2d_v(&self) -> f64 {
        self.max_error_2d_v
    }
}

/// The 2D poles whose x and y are the 1D poles of the subspaces `indices`.
pub(crate) fn poles_from_two_1d(
    approx: &AdvApproxApproxAFunction,
    [index_u, index_v]: [i32; 2],
) -> Result<Array1<'static, Point2>, Failure> {
    let nb_poles: i32 = approx.nb_poles()?;
    let mut poles_u: Array1<'static, f64> = one_based(nb_poles)?;
    let mut poles_v: Array1<'static, f64> = one_based(nb_poles)?;
    approx.poles_1d_of(index_u, &mut poles_u)?;
    approx.poles_1d_of(index_v, &mut poles_v)?;
    let mut poles: Array1<'static, Point2> = one_based(nb_poles)?;
    for i in 1..=nb_poles {
        poles[i] = Point2::new(poles_u[i], poles_v[i]);
    }
    Ok(poles)
}
