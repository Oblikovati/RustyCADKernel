// Copyright (c) 1993-1999 Matra Datavision
// Copyright (c) 1999-2024 OPEN CASCADE SAS
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
// Rust translation of IntCurveSurface_TheHCurveTool.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntCurveSurface_TheHCurveTool`: the static curve tool of IntCurveSurface_HInter
//! over `Adaptor3d_Curve` handles.
use super::sample_pars::bspline_sample_pars;
use super::tools::InterCurveTool;
use crate::int_imp::TheCurveTool;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dD3};
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomBezierCurve};
use rustycad_tkmath::geom_abs::{CurveType, Shape};
use rustycad_tkmath::gp::{Circle3, Ellipse3, Hyperbola3, Line3, Parabola3, Point3, Vector3};

/// `TheCurve` of IntCurveSurface_HInter: a handle to a curve adaptor.
pub type HCurve = Handle<dyn Adaptor3dCurve>;

/// `IntCurveSurface_TheHCurveTool`: every member forwards to the adaptor, except
/// the sample counts and parameters.
///
/// ```
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkg3d::geom::{GeomCurve, GeomLine};
/// use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
/// use rustycad_tkgeomalgo::int_curve_surface::{HCurve, TheHCurveTool};
/// use rustycad_tkmath::gp::{Direction3, Point3};
/// let line: Handle<dyn GeomCurve> =
///     Handle::new(GeomLine::from_point(Point3::new(0.0, 0.0, 0.0), Direction3::new(1.0, 0.0, 0.0).unwrap())).cast();
/// let curve: HCurve = Handle::new(GeomAdaptorCurve::with_range(&line, 0.0, 1.0).unwrap()).cast();
/// assert_eq!(TheHCurveTool::nb_samples(&curve, 0.0, 1.0).unwrap(), 2);
/// ```
pub struct TheHCurveTool;

impl TheHCurveTool {
    /// `FirstParameter(C)`; `TheHCurveTool::first_parameter(&c)?`.
    pub fn first_parameter(c: &HCurve) -> Result<f64, Failure> {
        c.first_parameter()
    }
    /// `LastParameter(C)`; `TheHCurveTool::last_parameter(&c)?`.
    pub fn last_parameter(c: &HCurve) -> Result<f64, Failure> {
        c.last_parameter()
    }
    /// `Continuity(C)`; `TheHCurveTool::continuity(&c)?`.
    pub fn continuity(c: &HCurve) -> Result<Shape, Failure> {
        c.continuity()
    }
    /// `NbIntervals(C, S)`; `TheHCurveTool::nb_intervals(&c, Shape::C2)?`.
    pub fn nb_intervals(c: &HCurve, s: Shape) -> Result<i32, Failure> {
        c.nb_intervals(s)
    }
    /// `Intervals(C, T, S)`; `TheHCurveTool::intervals(&c, &mut t, Shape::C2)?`.
    pub fn intervals(c: &HCurve, t: &mut Array1<'_, f64>, s: Shape) -> Result<(), Failure> {
        c.intervals(t, s)
    }
    /// `IsClosed(C)`; `TheHCurveTool::is_closed(&c)?`.
    pub fn is_closed(c: &HCurve) -> Result<bool, Failure> {
        c.is_closed()
    }
    /// `IsPeriodic(C)`; `TheHCurveTool::is_periodic(&c)?`.
    pub fn is_periodic(c: &HCurve) -> Result<bool, Failure> {
        c.is_periodic()
    }
    /// `Period(C)`; `TheHCurveTool::period(&c)?`.
    pub fn period(c: &HCurve) -> Result<f64, Failure> {
        c.period()
    }
    /// `Value(C, U)`; `TheHCurveTool::value(&c, 0.5)?`.
    pub fn value(c: &HCurve, u: f64) -> Result<Point3, Failure> {
        c.value(u)
    }
    /// `D0(C, U, P)`; `TheHCurveTool::d0(&c, 0.5)?`.
    pub fn d0(c: &HCurve, u: f64) -> Result<Point3, Failure> {
        c.d0(u)
    }
    /// `D1(C, U, P, V)`; `TheHCurveTool::d1(&c, 0.5)?`.
    pub fn d1(c: &HCurve, u: f64) -> Result<(Point3, Vector3), Failure> {
        c.d1(u)
    }
    /// `D2(C, U, P, V1, V2)`; `TheHCurveTool::d2(&c, 0.5)?`.
    pub fn d2(c: &HCurve, u: f64) -> Result<(Point3, Vector3, Vector3), Failure> {
        c.d2(u)
    }
    /// `D3(C, U, P, V1, V2, V3)`; `TheHCurveTool::d3(&c, 0.5)?`.
    pub fn d3(c: &HCurve, u: f64) -> Result<Adaptor3dD3, Failure> {
        c.d3(u)
    }
    /// `DN(C, U, N)`; `TheHCurveTool::dn(&c, 0.5, 2)?`.
    pub fn dn(c: &HCurve, u: f64, n: i32) -> Result<Vector3, Failure> {
        c.dn(u, n)
    }
    /// `Resolution(C, R3d)`; `TheHCurveTool::resolution(&c, 1e-7)?`.
    pub fn resolution(c: &HCurve, r3d: f64) -> Result<f64, Failure> {
        c.resolution(r3d)
    }
    /// `GetType(C)`; `TheHCurveTool::get_type(&c)?`.
    pub fn get_type(c: &HCurve) -> Result<CurveType, Failure> {
        c.get_type()
    }
    /// `Line(C)`; `TheHCurveTool::line(&c)?`.
    pub fn line(c: &HCurve) -> Result<Line3, Failure> {
        c.line()
    }
    /// `Circle(C)`; `TheHCurveTool::circle(&c)?`.
    pub fn circle(c: &HCurve) -> Result<Circle3, Failure> {
        c.circle()
    }
    /// `Ellipse(C)`; `TheHCurveTool::ellipse(&c)?`.
    pub fn ellipse(c: &HCurve) -> Result<Ellipse3, Failure> {
        c.ellipse()
    }
    /// `Hyperbola(C)`; `TheHCurveTool::hyperbola(&c)?`.
    pub fn hyperbola(c: &HCurve) -> Result<Hyperbola3, Failure> {
        c.hyperbola()
    }
    /// `Parabola(C)`; `TheHCurveTool::parabola(&c)?`.
    pub fn parabola(c: &HCurve) -> Result<Parabola3, Failure> {
        c.parabola()
    }
    /// `Bezier(C)`; `TheHCurveTool::bezier(&c)?`.
    pub fn bezier(c: &HCurve) -> Result<Handle<GeomBezierCurve>, Failure> {
        c.bezier()
    }
    /// `BSpline(C)`; `TheHCurveTool::bspline(&c)?`.
    pub fn bspline(c: &HCurve) -> Result<Handle<GeomBSplineCurve>, Failure> {
        c.bspline()
    }
    /// `NbSamples(C, U0, U1)`: 2 for a line, `3 + NbPoles` for a Bezier curve,
    /// `NbKnots * Degree * (Last - First) / (U1 - U0)` (at least 2) for a B-spline,
    /// 10 otherwise, at most 50 and truncated; `TheHCurveTool::nb_samples(&c, 0.0, 1.0)?`.
    pub fn nb_samples(c: &HCurve, u0: f64, u1: f64) -> Result<i32, Failure> {
        let mut nbs: f64 = match c.get_type()? {
            CurveType::Line => 2.0,
            CurveType::BezierCurve => f64::from(3 + c.nb_poles()?),
            CurveType::BSplineCurve => bspline_nb_samples(c, u0, u1)?,
            _ => 10.0,
        };
        if nbs > 50.0 {
            nbs = 50.0;
        }
        // (int) of a NaN count is undefined natively; `as` gives 0 here.
        Ok(nbs as i32)
    }
    /// `SamplePars(C, U0, U1, Defl, NbMin)`: uniform parameters (2 for a line,
    /// `3 + NbPoles` for a Bezier curve, 10 otherwise, at most 50); for a B-spline, the
    /// knots split `Degree` times, thinned while the chords stay within `Defl`, or
    /// `max(NbMin, 5)` uniform ones; `TheHCurveTool::sample_pars(&c, 0.0, 1.0, 0.1, 10)?`.
    /// The deprecated overload that writes a handle argument is this member too.
    pub fn sample_pars(
        c: &HCurve,
        u0: f64,
        u1: f64,
        defl: f64,
        nb_min: i32,
    ) -> Result<Array1<'static, f64>, Failure> {
        let nbs: f64 = match c.get_type()? {
            CurveType::Line => 2.0,
            CurveType::BezierCurve => f64::from(3 + c.nb_poles()?),
            CurveType::BSplineCurve => {
                return bspline_sample_pars(&c.bspline()?, u0, u1, defl, nb_min);
            }
            _ => 10.0,
        };
        let nnbs: i32 = if nbs > 50.0 { 50 } else { nbs as i32 };
        uniform_pars(nnbs, u0, u1)
    }
}

/// The B-spline sample count before the cap: knots times degree, scaled by the
/// curve range over `[u0, u1]` when that range is not null.
fn bspline_nb_samples(c: &HCurve, u0: f64, u1: f64) -> Result<f64, Failure> {
    let mut nbs: f64 = f64::from(c.nb_knots()?);
    nbs *= f64::from(c.degree()?);
    nbs *= c.last_parameter()? - c.first_parameter()?;
    let range: f64 = u1 - u0;
    if range.abs() > precision::p_confusion() {
        nbs /= range;
    }
    if nbs < 2.0 {
        nbs = 2.0;
    }
    Ok(nbs)
}

/// `nnbs` parameters from `u0` to `u1` by steps of `(u1 - u0) / (nnbs - 1)`, the
/// inner ones accumulated and the last one exactly `u1`.
pub(super) fn uniform_pars(nnbs: i32, u0: f64, u1: f64) -> Result<Array1<'static, f64>, Failure> {
    let mut pars: Array1<'static, f64> =
        Array1::with_bounds(1, nnbs).map_err(|e| e.into_failure("NCollection_HArray1"))?;
    let du: f64 = (u1 - u0) / f64::from(nnbs - 1);
    pars[1] = u0;
    pars[nnbs] = u1;
    let mut u: f64 = u0 + du;
    for i in 2..nnbs {
        pars[i] = u;
        u += du;
    }
    Ok(pars)
}

impl TheCurveTool for TheHCurveTool {
    type Curve = HCurve;
    fn first_parameter(curve: &HCurve) -> Result<f64, Failure> {
        TheHCurveTool::first_parameter(curve)
    }
    fn last_parameter(curve: &HCurve) -> Result<f64, Failure> {
        TheHCurveTool::last_parameter(curve)
    }
    fn resolution(curve: &HCurve, r3d: f64) -> Result<f64, Failure> {
        TheHCurveTool::resolution(curve, r3d)
    }
    fn value(curve: &HCurve, w: f64) -> Result<Point3, Failure> {
        TheHCurveTool::value(curve, w)
    }
    fn d1(curve: &HCurve, w: f64) -> Result<(Point3, Vector3), Failure> {
        TheHCurveTool::d1(curve, w)
    }
}

impl InterCurveTool for TheHCurveTool {
    fn get_type(curve: &HCurve) -> Result<CurveType, Failure> {
        TheHCurveTool::get_type(curve)
    }
    fn line(curve: &HCurve) -> Result<Line3, Failure> {
        TheHCurveTool::line(curve)
    }
    fn circle(curve: &HCurve) -> Result<Circle3, Failure> {
        TheHCurveTool::circle(curve)
    }
    fn ellipse(curve: &HCurve) -> Result<Ellipse3, Failure> {
        TheHCurveTool::ellipse(curve)
    }
    fn parabola(curve: &HCurve) -> Result<Parabola3, Failure> {
        TheHCurveTool::parabola(curve)
    }
    fn hyperbola(curve: &HCurve) -> Result<Hyperbola3, Failure> {
        TheHCurveTool::hyperbola(curve)
    }
    fn nb_intervals(curve: &HCurve, shape: Shape) -> Result<i32, Failure> {
        TheHCurveTool::nb_intervals(curve, shape)
    }
    fn intervals(
        curve: &HCurve,
        bounds: &mut Array1<'_, f64>,
        shape: Shape,
    ) -> Result<(), Failure> {
        TheHCurveTool::intervals(curve, bounds, shape)
    }
    fn is_periodic(curve: &HCurve) -> Result<bool, Failure> {
        TheHCurveTool::is_periodic(curve)
    }
    fn period(curve: &HCurve) -> Result<f64, Failure> {
        TheHCurveTool::period(curve)
    }
    fn d0(curve: &HCurve, u: f64) -> Result<Point3, Failure> {
        TheHCurveTool::d0(curve, u)
    }
    fn nb_samples(curve: &HCurve, u0: f64, u1: f64) -> Result<i32, Failure> {
        TheHCurveTool::nb_samples(curve, u0, u1)
    }
    fn sample_pars(
        curve: &HCurve,
        u0: f64,
        u1: f64,
        defl: f64,
        nb_min: i32,
    ) -> Result<Array1<'static, f64>, Failure> {
        TheHCurveTool::sample_pars(curve, u0, u1, defl, nb_min)
    }
}
