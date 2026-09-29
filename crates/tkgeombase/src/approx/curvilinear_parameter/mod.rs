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
// Rust translation of ModelingData/TKGeomBase/Approx/Approx_CurvilinearParameter.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Approx_CurvilinearParameter`: the B-spline approximation of a curve (or of a
//! curve on one or two surfaces) reparametrized by its curvilinear abscissa.
mod evaluator;

use super::curve_2d::poles_from_two_1d;
use super::curve_3d::null_table;
use super::curvlin_func::CurvlinFunc;
use crate::native_text::stream_real;
use evaluator::CurvlinEvaluator;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::Geom2dBSplineCurve;
use rustycad_tkg2d::geom2d::arrays::one_based;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::adv_approx::{AdvApproxApproxAFunction, AdvApproxPrefAndRec};
use rustycad_tkg3d::geom::GeomBSplineCurve;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::Point3;
use std::fmt;

/// `std::max(a, b)`: `b` only when `a < b`.
fn std_max(a: f64, b: f64) -> f64 {
    if a < b { b } else { a }
}

/// The approximation options: `(Order, MaxDegree, MaxSegments)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CurvilinearOptions {
    /// `Order`: the continuity.
    pub order: Shape,
    /// `MaxDegree`.
    pub max_degree: i32,
    /// `MaxSegments`.
    pub max_segments: i32,
}

/// `Approx_CurvilinearParameter`: the reparametrized 3D curve and 2D curves with
/// their errors. The case 2 constructor leaves `myMaxError2d2` uninitialized
/// natively; the port reads 0.
///
/// ```ignore
/// let options = CurvilinearOptions { order: Shape::C2, max_degree: 14, max_segments: 16 };
/// let approx = CurvilinearParameter::from_curve(&adaptor, 1e-6, options)?;
/// ```
#[derive(Clone)]
pub struct CurvilinearParameter {
    case: i32,
    done: bool,
    has_result: bool,
    curve_3d: Handle<GeomBSplineCurve>,
    max_error_3d: f64,
    curve_2d1: Handle<Geom2dBSplineCurve>,
    max_error_2d1: f64,
    curve_2d2: Handle<Geom2dBSplineCurve>,
    max_error_2d2: f64,
}

impl CurvilinearParameter {
    fn empty(case: i32) -> Self {
        Self {
            case,
            done: false,
            has_result: false,
            curve_3d: Handle::default(),
            max_error_3d: 0.0,
            curve_2d1: Handle::default(),
            max_error_2d1: 0.0,
            curve_2d2: Handle::default(),
            max_error_2d2: 0.0,
        }
    }

    /// `Approx_CurvilinearParameter(C3D, Tol, Order, MaxDegree, MaxSegments)`: the 3D
    /// curve within `tol` (lengths within `tol / 10`);
    /// `CurvilinearParameter::from_curve(&adaptor, 1e-6, options)?`.
    pub fn from_curve(
        c3d: &Handle<dyn Adaptor3dCurve>,
        tol: f64,
        options: CurvilinearOptions,
    ) -> Result<Self, Failure> {
        let mut three_d_tol: Array1<'static, f64> = one_based(1)?;
        three_d_tol.init(tol);
        let fonct = CurvlinFunc::from_curve(c3d, tol / 10.)?;
        let approx = approximate(fonct, 1, [None, Some(&three_d_tol)], options)?;
        let mut result: Self = Self::empty(1);
        result.store(&approx)?;
        result.max_error_3d = approx.max_error(3, 1)?;
        Ok(result)
    }

    /// `Approx_CurvilinearParameter(C2D, Surf, Tol, Order, MaxDegree, MaxSegments)`:
    /// the curve on surface; its u and v tolerances are `tol` (the computed ones are
    /// overwritten natively), the 3D one `tol / 2`;
    /// `CurvilinearParameter::from_curve_on_surface(&c2d, &s, 1e-6, options)?`.
    pub fn from_curve_on_surface(
        c2d: &Handle<dyn Adaptor2dCurve2d>,
        surf: &Handle<dyn Adaptor3dSurface>,
        tol: f64,
        options: CurvilinearOptions,
    ) -> Result<Self, Failure> {
        let mut one_d_tol: Array1<'static, f64> = one_based(2)?;
        (one_d_tol[1], one_d_tol[2]) = tolerance_computation(c2d, surf, 10, tol)?;
        (one_d_tol[1], one_d_tol[2]) = (tol, tol);
        let mut three_d_tol: Array1<'static, f64> = one_based(1)?;
        three_d_tol.init(tol / 2.);
        let fonct = CurvlinFunc::from_curve_on_surface(c2d, surf, tol / 20.)?;
        let approx = approximate(fonct, 2, [Some(&one_d_tol), Some(&three_d_tol)], options)?;
        let mut result: Self = Self::empty(2);
        result.store(&approx)?;
        result.max_error_2d1 = std_max(approx.max_error(1, 1)?, approx.max_error(1, 2)?);
        result.max_error_3d = approx.max_error(3, 1)?;
        Ok(result)
    }

    /// `Approx_CurvilinearParameter(C2D1, Surf1, C2D2, Surf2, Tol, Order, MaxDegree,
    /// MaxSegments)`: the two curves on surfaces, with their computed u and v
    /// tolerances; `CurvilinearParameter::from_two_curves_on_surfaces(&c1, &s1, &c2, &s2, 1e-6, options)?`.
    pub fn from_two_curves_on_surfaces(
        [c2d1, c2d2]: [&Handle<dyn Adaptor2dCurve2d>; 2],
        [surf1, surf2]: [&Handle<dyn Adaptor3dSurface>; 2],
        tol: f64,
        options: CurvilinearOptions,
    ) -> Result<Self, Failure> {
        let mut one_d_tol: Array1<'static, f64> = one_based(4)?;
        (one_d_tol[1], one_d_tol[2]) = tolerance_computation(c2d1, surf1, 10, tol)?;
        (one_d_tol[3], one_d_tol[4]) = tolerance_computation(c2d2, surf2, 10, tol)?;
        let mut three_d_tol: Array1<'static, f64> = one_based(1)?;
        three_d_tol.init(tol / 2.);
        let fonct = CurvlinFunc::from_two_curves_on_surfaces(c2d1, c2d2, surf1, surf2, tol / 20.)?;
        let approx = approximate(fonct, 3, [Some(&one_d_tol), Some(&three_d_tol)], options)?;
        let mut result: Self = Self::empty(3);
        result.store(&approx)?;
        result.max_error_2d1 = std_max(approx.max_error(1, 1)?, approx.max_error(1, 2)?);
        result.max_error_2d2 = std_max(approx.max_error(1, 3)?, approx.max_error(1, 4)?);
        result.max_error_3d = approx.max_error(3, 1)?;
        Ok(result)
    }

    // The curves of the approximation, when there is a result.
    fn store(&mut self, approx: &AdvApproxApproxAFunction) -> Result<(), Failure> {
        self.done = approx.is_done();
        self.has_result = approx.has_result();
        if !self.has_result {
            return Ok(());
        }
        let knots = approx.knots().ok_or_else(|| null_table("Knots"))?;
        let mults = approx
            .multiplicities()
            .ok_or_else(|| null_table("Multiplicities"))?;
        let degree: i32 = approx.degree();
        let mut poles: Array1<'static, Point3> = one_based(approx.nb_poles()?)?;
        approx.poles_of(1, &mut poles)?;
        self.curve_3d = Handle::new(GeomBSplineCurve::new(&poles, knots, mults, degree, false)?);
        if self.case == 1 {
            return Ok(());
        }
        let poles2d = poles_from_two_1d(approx, [1, 2])?;
        self.curve_2d1 = Handle::new(Geom2dBSplineCurve::new(
            &poles2d, knots, mults, degree, false,
        )?);
        if self.case == 3 {
            let poles2d = poles_from_two_1d(approx, [3, 4])?;
            self.curve_2d2 = Handle::new(Geom2dBSplineCurve::new(
                &poles2d, knots, mults, degree, false,
            )?);
        }
        Ok(())
    }

    /// `IsDone()`; `approx.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }

    /// `HasResult()`; `approx.has_result()`.
    pub fn has_result(&self) -> bool {
        self.has_result
    }

    /// `Curve3d()`; `approx.curve_3d()`.
    pub fn curve_3d(&self) -> Handle<GeomBSplineCurve> {
        self.curve_3d.clone()
    }

    /// `MaxError3d()`; `approx.max_error_3d()`.
    pub fn max_error_3d(&self) -> f64 {
        self.max_error_3d
    }

    /// `Curve2d1()`: the 2D curve on the first surface; `approx.curve_2d1()`.
    pub fn curve_2d1(&self) -> Handle<Geom2dBSplineCurve> {
        self.curve_2d1.clone()
    }

    /// `MaxError2d1()`; `approx.max_error_2d1()`.
    pub fn max_error_2d1(&self) -> f64 {
        self.max_error_2d1
    }

    /// `Curve2d2()`: the 2D curve on the second surface; `approx.curve_2d2()`.
    pub fn curve_2d2(&self) -> Handle<Geom2dBSplineCurve> {
        self.curve_2d2.clone()
    }

    /// `MaxError2d2()`; `approx.max_error_2d2()`.
    pub fn max_error_2d2(&self) -> f64 {
        self.max_error_2d2
    }

    /// `Dump(o)`: the errors of the case, printed as `%g`; `approx.dump(&mut text)`.
    pub fn dump(&self, stream: &mut dyn fmt::Write) -> fmt::Result {
        writeln!(stream, "Dump of Approx_CurvilinearParameter")?;
        if self.case == 2 || self.case == 3 {
            writeln!(
                stream,
                "myMaxError2d1 = {}",
                stream_real(self.max_error_2d1)
            )?;
        }
        if self.case == 3 {
            writeln!(
                stream,
                "myMaxError2d2 = {}",
                stream_real(self.max_error_2d2)
            )?;
        }
        writeln!(stream, "myMaxError3d = {}", stream_real(self.max_error_3d))
    }
}

// AdvApprox over the abscissa range of `fonct`, cut at its C2 and C3 breaks.
fn approximate(
    fonct: CurvlinFunc,
    case: i32,
    [one_d, three_d]: [Option<&Array1<'static, f64>>; 2],
    options: CurvilinearOptions,
) -> Result<AdvApproxApproxAFunction, Failure> {
    let (first_s, last_s) = (fonct.first_parameter(), fonct.last_parameter());
    let cut_tool = AdvApproxPrefAndRec::new(
        &breaks(&fonct, Shape::C2)?,
        &breaks(&fonct, Shape::C3)?,
        5.0,
    )?;
    let num_1d: i32 = [0, 0, 2, 4][case as usize];
    let mut evaluator = CurvlinEvaluator::new(fonct, [first_s, last_s], case);
    let approx = AdvApproxApproxAFunction::with_cutting(
        num_1d,
        0,
        1,
        one_d,
        None,
        three_d,
        first_s,
        last_s,
        options.order,
        options.max_degree,
        options.max_segments,
        &mut evaluator,
        &cut_tool,
    );
    evaluator.take_failure()?;
    approx
}

fn breaks(fonct: &CurvlinFunc, shape: Shape) -> Result<Array1<'static, f64>, Failure> {
    let mut t: Array1<'static, f64> = one_based(fonct.nb_intervals(shape)? + 1)?;
    fonct.intervals(&mut t, shape)?;
    Ok(t)
}

/// `ToleranceComputation(C2D, S, MaxNumber, Tol, TolV, TolW)`: the u and v
/// tolerances `Tol / (4 max(1, |dS/du|))` (and `dS/dv`) over `max_number` points.
fn tolerance_computation(
    c2d: &Handle<dyn Adaptor2dCurve2d>,
    surf: &Handle<dyn Adaptor3dSurface>,
    max_number: i32,
    tol: f64,
) -> Result<(f64, f64), Failure> {
    let c2d = c2d.get().ok_or_else(|| null_table("C2D"))?;
    let surf = surf.get().ok_or_else(|| null_table("S"))?;
    let (first_u, last_u) = (c2d.first_parameter()?, c2d.last_parameter()?);
    let (mut max_ds_dv, mut max_ds_dw): (f64, f64) = (1., 1.);
    for i in 1..=max_number {
        let u: f64 = first_u + f64::from(i - 1) * (last_u - first_u) / f64::from(max_number - 1);
        let pnt_vw = c2d.value(u)?;
        let d = surf.d1(pnt_vw.x(), pnt_vw.y())?;
        max_ds_dv = std_max(max_ds_dv, d.d1u.magnitude());
        max_ds_dw = std_max(max_ds_dw, d.d1v.magnitude());
    }
    Ok((tol / (4. * max_ds_dv), tol / (4. * max_ds_dw)))
}
