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
// Rust translation of ModelingData/TKG2d/Adaptor2d/Adaptor2d_Curve2d.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Adaptor2d_Curve2d`: the interface that 2D curve algorithms evaluate through.
use crate::geom2d::errors::not_implemented;
use crate::geom2d::{
    Geom2dBSplineCurve, Geom2dBezierCurve, Geom2dCurveD1, Geom2dCurveD2, Geom2dCurveD3,
};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient, Transient};
use rustycad_tkernel::{impl_transient, impl_transient_interface};
use rustycad_tkmath::geom_abs::{CurveType, Shape};
use rustycad_tkmath::gp::{Circle2, Ellipse2, Hyperbola2, Line2, Parabola2, Point2, Vector2};

/// Point and three derivatives as the `D3` output parameters.
pub type Adaptor2dD3 = (Point2, Vector2, Vector2, Vector2);

/// A 2D curve seen by geometric algorithms (`Adaptor2d_Curve2d`).
///
/// Every virtual member of the native base class raises Standard_NotImplemented,
/// except `NbSamples`, which returns 20; the provided methods here do the same, and
/// each concrete adaptor overrides them. The non-virtual `Value`, `D0`..`D3` and
/// `DN` forward to the `EvalD*` members. All members take `&self`: adaptors keep
/// their state behind a lock (docs/geom-design.md);
/// `let point: Point2 = adaptor.value(0.5)?;`.
pub trait Adaptor2dCurve2d: Transient {
    /// A new adaptor sharing the curve (`ShallowCopy`); `adaptor.shallow_copy()?`.
    fn shallow_copy(&self) -> Result<Handle<dyn Adaptor2dCurve2d>, Failure> {
        Err(not_implemented("Adaptor2d_Curve2d::ShallowCopy"))
    }
    /// First parameter (`FirstParameter`); `adaptor.first_parameter()?`.
    fn first_parameter(&self) -> Result<f64, Failure> {
        Err(not_implemented("Adaptor2d_Curve2d::FirstParameter"))
    }
    /// Last parameter (`LastParameter`); `adaptor.last_parameter()?`.
    fn last_parameter(&self) -> Result<f64, Failure> {
        Err(not_implemented("Adaptor2d_Curve2d::LastParameter"))
    }
    /// Global continuity (`Continuity`); `adaptor.continuity()?`.
    fn continuity(&self) -> Result<Shape, Failure> {
        Err(not_implemented("Adaptor2d_Curve2d::Continuity"))
    }
    /// Number of intervals of continuity `shape` (`NbIntervals`); `adaptor.nb_intervals(Shape::C2)?`.
    fn nb_intervals(&self, _shape: Shape) -> Result<i32, Failure> {
        Err(not_implemented("Adaptor2d_Curve2d::NbIntervals"))
    }
    /// Fill `bounds` with the interval bounds; it needs `NbIntervals + 1` values
    /// (`Intervals`); `adaptor.intervals(&mut bounds, Shape::C2)?`.
    fn intervals(&self, _bounds: &mut Array1<'_, f64>, _shape: Shape) -> Result<(), Failure> {
        Err(not_implemented("Adaptor2d_Curve2d::Intervals"))
    }
    /// The same curve on `[first, last]` (`Trim`); `adaptor.trim(0.0, 1.0, 1e-7)?`.
    fn trim(
        &self,
        _first: f64,
        _last: f64,
        _tolerance: f64,
    ) -> Result<Handle<dyn Adaptor2dCurve2d>, Failure> {
        Err(not_implemented("Adaptor2d_Curve2d::Trim"))
    }
    /// Whether the curve is closed (`IsClosed`); `adaptor.is_closed()?`.
    fn is_closed(&self) -> Result<bool, Failure> {
        Err(not_implemented("Adaptor2d_Curve2d::IsClosed"))
    }
    /// Whether the curve is periodic (`IsPeriodic`); `adaptor.is_periodic()?`.
    fn is_periodic(&self) -> Result<bool, Failure> {
        Err(not_implemented("Adaptor2d_Curve2d::IsPeriodic"))
    }
    /// The period (`Period`); `adaptor.period()?`.
    fn period(&self) -> Result<f64, Failure> {
        Err(not_implemented("Adaptor2d_Curve2d::Period"))
    }
    /// Point of parameter `u` (`Value`, non-virtual); `adaptor.value(0.5)?`.
    fn value(&self, u: f64) -> Result<Point2, Failure> {
        self.eval_d0(u)
    }
    /// Point of parameter `u` (`D0`, non-virtual); `adaptor.d0(0.5)?`.
    fn d0(&self, u: f64) -> Result<Point2, Failure> {
        self.eval_d0(u)
    }
    /// Point and first derivative (`D1`, non-virtual); `let (p, v) = adaptor.d1(0.5)?;`.
    fn d1(&self, u: f64) -> Result<(Point2, Vector2), Failure> {
        let result: Geom2dCurveD1 = self.eval_d1(u)?;
        Ok((result.point, result.d1))
    }
    /// Point and two derivatives (`D2`, non-virtual); `let (p, v1, v2) = adaptor.d2(0.5)?;`.
    fn d2(&self, u: f64) -> Result<(Point2, Vector2, Vector2), Failure> {
        let result: Geom2dCurveD2 = self.eval_d2(u)?;
        Ok((result.point, result.d1, result.d2))
    }
    /// Point and three derivatives (`D3`, non-virtual); `let (p, v1, v2, v3) = adaptor.d3(0.5)?;`.
    fn d3(&self, u: f64) -> Result<Adaptor2dD3, Failure> {
        let result: Geom2dCurveD3 = self.eval_d3(u)?;
        Ok((result.point, result.d1, result.d2, result.d3))
    }
    /// Derivative of order `n` (`DN`, non-virtual); `adaptor.dn(0.5, 2)?`.
    fn dn(&self, u: f64, n: i32) -> Result<Vector2, Failure> {
        self.eval_dn(u, n)
    }
    /// Parametric resolution of the space resolution `r3d` (`Resolution`); `adaptor.resolution(1e-7)?`.
    fn resolution(&self, _r3d: f64) -> Result<f64, Failure> {
        Err(not_implemented("Adaptor2d_Curve2d::Resolution"))
    }
    /// Kind of the underlying curve (`GetType`); `adaptor.get_type()?`.
    fn get_type(&self) -> Result<CurveType, Failure> {
        Err(not_implemented("Adaptor2d_Curve2d::GetType"))
    }
    /// The line, for a `Line` type (`Line`); `adaptor.line()?`.
    fn line(&self) -> Result<Line2, Failure> {
        Err(not_implemented("Adaptor2d_Curve2d::Line"))
    }
    /// The circle, for a `Circle` type (`Circle`); `adaptor.circle()?`.
    fn circle(&self) -> Result<Circle2, Failure> {
        Err(not_implemented("Adaptor2d_Curve2d::Circle"))
    }
    /// The ellipse, for an `Ellipse` type (`Ellipse`); `adaptor.ellipse()?`.
    fn ellipse(&self) -> Result<Ellipse2, Failure> {
        Err(not_implemented("Adaptor2d_Curve2d::Ellipse"))
    }
    /// The hyperbola, for a `Hyperbola` type (`Hyperbola`); `adaptor.hyperbola()?`.
    fn hyperbola(&self) -> Result<Hyperbola2, Failure> {
        Err(not_implemented("Adaptor2d_Curve2d::Hyperbola"))
    }
    /// The parabola, for a `Parabola` type (`Parabola`); `adaptor.parabola()?`.
    fn parabola(&self) -> Result<Parabola2, Failure> {
        Err(not_implemented("Adaptor2d_Curve2d::Parabola"))
    }
    /// Degree of a Bezier or B-spline curve (`Degree`); `adaptor.degree()?`.
    fn degree(&self) -> Result<i32, Failure> {
        Err(not_implemented("Adaptor2d_Curve2d::Degree"))
    }
    /// Whether the curve is rational (`IsRational`); `adaptor.is_rational()?`.
    fn is_rational(&self) -> Result<bool, Failure> {
        Err(not_implemented("Adaptor2d_Curve2d::IsRational"))
    }
    /// Number of poles (`NbPoles`; the native message says `NbPole`); `adaptor.nb_poles()?`.
    fn nb_poles(&self) -> Result<i32, Failure> {
        Err(not_implemented("Adaptor2d_Curve2d::NbPole"))
    }
    /// Number of knots (`NbKnots`); `adaptor.nb_knots()?`.
    fn nb_knots(&self) -> Result<i32, Failure> {
        Err(not_implemented("Adaptor2d_Curve2d::NbKnots"))
    }
    /// Suggested number of sample points, 20 by default (`NbSamples`); `adaptor.nb_samples()?`.
    fn nb_samples(&self) -> Result<i32, Failure> {
        Ok(20)
    }
    /// The Bezier curve (`Bezier`); `adaptor.bezier()?`.
    fn bezier(&self) -> Result<Handle<Geom2dBezierCurve>, Failure> {
        Err(not_implemented("Adaptor2d_Curve2d::Bezier"))
    }
    /// The B-spline curve (`BSpline`); `adaptor.bspline()?`.
    fn bspline(&self) -> Result<Handle<Geom2dBSplineCurve>, Failure> {
        Err(not_implemented("Adaptor2d_Curve2d::BSpline"))
    }
    /// Point of parameter `u` (`EvalD0`); `adaptor.eval_d0(0.5)?`.
    fn eval_d0(&self, _u: f64) -> Result<Point2, Failure> {
        Err(not_implemented("Adaptor2d_Curve2d::EvalD0"))
    }
    /// Point and first derivative (`EvalD1`); `adaptor.eval_d1(0.5)?.d1`.
    fn eval_d1(&self, _u: f64) -> Result<Geom2dCurveD1, Failure> {
        Err(not_implemented("Adaptor2d_Curve2d::EvalD1"))
    }
    /// Point and two derivatives (`EvalD2`); `adaptor.eval_d2(0.5)?.d2`.
    fn eval_d2(&self, _u: f64) -> Result<Geom2dCurveD2, Failure> {
        Err(not_implemented("Adaptor2d_Curve2d::EvalD2"))
    }
    /// Point and three derivatives (`EvalD3`); `adaptor.eval_d3(0.5)?.d3`.
    fn eval_d3(&self, _u: f64) -> Result<Geom2dCurveD3, Failure> {
        Err(not_implemented("Adaptor2d_Curve2d::EvalD3"))
    }
    /// Derivative of order `n` (`EvalDN`); `adaptor.eval_dn(0.5, 2)?`.
    fn eval_dn(&self, _u: f64, _n: i32) -> Result<Vector2, Failure> {
        Err(not_implemented("Adaptor2d_Curve2d::EvalDN"))
    }
}
impl_transient_interface!(dyn Adaptor2dCurve2d, "Adaptor2d_Curve2d", StandardTransient);

/// An instance of the non-abstract native base class itself, whose members all raise
/// Standard_NotImplemented but `NbSamples`; `Handle::new(Adaptor2dCurve2dBase::default())`.
///
/// Its dynamic type is `Adaptor2d_Curve2d`, as for `new Adaptor2d_Curve2d()`.
#[derive(Default)]
pub struct Adaptor2dCurve2dBase {
    parent: StandardTransient,
}
impl_transient!(
    Adaptor2dCurve2dBase,
    "Adaptor2d_Curve2d",
    StandardTransient,
    parent,
    interfaces: [dyn Adaptor2dCurve2d]
);
impl Adaptor2dCurve2d for Adaptor2dCurve2dBase {}
