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
// Rust translation of ModelingData/TKG3d/Adaptor3d/Adaptor3d_Curve.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Adaptor3d_Curve`: the interface that 3D curve algorithms evaluate through.
use crate::geom::errors::not_implemented;
use crate::geom::{
    BasisCurveEvaluator, GeomBSplineCurve, GeomBezierCurve, GeomCurveD1, GeomCurveD2, GeomCurveD3,
    GeomOffsetCurve,
};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient, Transient};
use rustycad_tkernel::{impl_transient, impl_transient_interface};
use rustycad_tkmath::geom_abs::{CurveType, Shape};
use rustycad_tkmath::gp::{Circle3, Ellipse3, Hyperbola3, Line3, Parabola3, Point3, Vector3};

/// Point and three derivatives as the `D3` output parameters.
pub type Adaptor3dD3 = (Point3, Vector3, Vector3, Vector3);

/// A 3D curve seen by geometric algorithms (`Adaptor3d_Curve`).
///
/// Every virtual member of the native base class raises Standard_NotImplemented;
/// the provided methods here do the same, and each concrete adaptor overrides them.
/// The non-virtual `Value`, `D0`..`D3` and `DN` forward to the `EvalD*` members.
/// All members take `&self`: adaptors keep their state behind a lock
/// (docs/geom-design.md); `let point: Point3 = adaptor.value(0.5)?;`.
pub trait Adaptor3dCurve: Transient {
    /// A new adaptor sharing the curve (`ShallowCopy`); `adaptor.shallow_copy()?`.
    fn shallow_copy(&self) -> Result<Handle<dyn Adaptor3dCurve>, Failure> {
        Err(not_implemented("Adaptor3d_Curve::ShallowCopy"))
    }
    /// First parameter (`FirstParameter`); `adaptor.first_parameter()?`.
    fn first_parameter(&self) -> Result<f64, Failure> {
        Err(not_implemented("Adaptor3d_Curve::FirstParameter"))
    }
    /// Last parameter (`LastParameter`); `adaptor.last_parameter()?`.
    fn last_parameter(&self) -> Result<f64, Failure> {
        Err(not_implemented("Adaptor3d_Curve::LastParameter"))
    }
    /// Global continuity (`Continuity`); `adaptor.continuity()?`.
    fn continuity(&self) -> Result<Shape, Failure> {
        Err(not_implemented("Adaptor3d_Curve::Continuity"))
    }
    /// Number of intervals of continuity `shape` (`NbIntervals`); `adaptor.nb_intervals(Shape::C2)?`.
    fn nb_intervals(&self, _shape: Shape) -> Result<i32, Failure> {
        Err(not_implemented("Adaptor3d_Curve::NbIntervals"))
    }
    /// Fill `bounds` with the interval bounds; it needs `NbIntervals + 1` values
    /// (`Intervals`); `adaptor.intervals(&mut bounds, Shape::C2)?`.
    fn intervals(&self, _bounds: &mut Array1<'_, f64>, _shape: Shape) -> Result<(), Failure> {
        Err(not_implemented("Adaptor3d_Curve::Intervals"))
    }
    /// The same curve on `[first, last]` (`Trim`); `adaptor.trim(0.0, 1.0, 1e-7)?`.
    fn trim(
        &self,
        _first: f64,
        _last: f64,
        _tolerance: f64,
    ) -> Result<Handle<dyn Adaptor3dCurve>, Failure> {
        Err(not_implemented("Adaptor3d_Curve::Trim"))
    }
    /// Whether the curve is closed (`IsClosed`); `adaptor.is_closed()?`.
    fn is_closed(&self) -> Result<bool, Failure> {
        Err(not_implemented("Adaptor3d_Curve::IsClosed"))
    }
    /// Whether the curve is periodic (`IsPeriodic`); `adaptor.is_periodic()?`.
    fn is_periodic(&self) -> Result<bool, Failure> {
        Err(not_implemented("Adaptor3d_Curve::IsPeriodic"))
    }
    /// The period (`Period`); `adaptor.period()?`.
    fn period(&self) -> Result<f64, Failure> {
        Err(not_implemented("Adaptor3d_Curve::Period"))
    }
    /// Point of parameter `u` (`Value`, non-virtual); `adaptor.value(0.5)?`.
    fn value(&self, u: f64) -> Result<Point3, Failure> {
        self.eval_d0(u)
    }
    /// Point of parameter `u` (`D0`, non-virtual); `adaptor.d0(0.5)?`.
    fn d0(&self, u: f64) -> Result<Point3, Failure> {
        self.eval_d0(u)
    }
    /// Point and first derivative (`D1`, non-virtual); `let (p, v) = adaptor.d1(0.5)?;`.
    fn d1(&self, u: f64) -> Result<(Point3, Vector3), Failure> {
        let result: GeomCurveD1 = self.eval_d1(u)?;
        Ok((result.point, result.d1))
    }
    /// Point and two derivatives (`D2`, non-virtual); `let (p, v1, v2) = adaptor.d2(0.5)?;`.
    fn d2(&self, u: f64) -> Result<(Point3, Vector3, Vector3), Failure> {
        let result: GeomCurveD2 = self.eval_d2(u)?;
        Ok((result.point, result.d1, result.d2))
    }
    /// Point and three derivatives (`D3`, non-virtual); `let (p, v1, v2, v3) = adaptor.d3(0.5)?;`.
    fn d3(&self, u: f64) -> Result<Adaptor3dD3, Failure> {
        let result: GeomCurveD3 = self.eval_d3(u)?;
        Ok((result.point, result.d1, result.d2, result.d3))
    }
    /// Derivative of order `n` (`DN`, non-virtual); `adaptor.dn(0.5, 2)?`.
    fn dn(&self, u: f64, n: i32) -> Result<Vector3, Failure> {
        self.eval_dn(u, n)
    }
    /// Parametric resolution of the space resolution `r3d` (`Resolution`); `adaptor.resolution(1e-7)?`.
    fn resolution(&self, _r3d: f64) -> Result<f64, Failure> {
        Err(not_implemented("Adaptor3d_Curve::Resolution"))
    }
    /// Kind of the underlying curve (`GetType`); `adaptor.get_type()?`.
    fn get_type(&self) -> Result<CurveType, Failure> {
        Err(not_implemented("Adaptor3d_Curve::GetType"))
    }
    /// The line, for a `Line` type (`Line`); `adaptor.line()?`.
    fn line(&self) -> Result<Line3, Failure> {
        Err(not_implemented("Adaptor3d_Curve::Line"))
    }
    /// The circle, for a `Circle` type (`Circle`); `adaptor.circle()?`.
    fn circle(&self) -> Result<Circle3, Failure> {
        Err(not_implemented("Adaptor3d_Curve::Circle"))
    }
    /// The ellipse, for an `Ellipse` type (`Ellipse`); `adaptor.ellipse()?`.
    fn ellipse(&self) -> Result<Ellipse3, Failure> {
        Err(not_implemented("Adaptor3d_Curve::Ellipse"))
    }
    /// The hyperbola, for a `Hyperbola` type (`Hyperbola`); `adaptor.hyperbola()?`.
    fn hyperbola(&self) -> Result<Hyperbola3, Failure> {
        Err(not_implemented("Adaptor3d_Curve::Hyperbola"))
    }
    /// The parabola, for a `Parabola` type (`Parabola`); `adaptor.parabola()?`.
    fn parabola(&self) -> Result<Parabola3, Failure> {
        Err(not_implemented("Adaptor3d_Curve::Parabola"))
    }
    /// Degree of a Bezier or B-spline curve (`Degree`); `adaptor.degree()?`.
    fn degree(&self) -> Result<i32, Failure> {
        Err(not_implemented("Adaptor3d_Curve::Degree"))
    }
    /// Whether the curve is rational (`IsRational`); `adaptor.is_rational()?`.
    fn is_rational(&self) -> Result<bool, Failure> {
        Err(not_implemented("Adaptor3d_Curve::IsRational"))
    }
    /// Number of poles (`NbPoles`); `adaptor.nb_poles()?`.
    fn nb_poles(&self) -> Result<i32, Failure> {
        Err(not_implemented("Adaptor3d_Curve::NbPoles"))
    }
    /// Number of knots (`NbKnots`); `adaptor.nb_knots()?`.
    fn nb_knots(&self) -> Result<i32, Failure> {
        Err(not_implemented("Adaptor3d_Curve::NbKnots"))
    }
    /// The Bezier curve (`Bezier`); `adaptor.bezier()?`.
    fn bezier(&self) -> Result<Handle<GeomBezierCurve>, Failure> {
        Err(not_implemented("Adaptor3d_Curve::Bezier"))
    }
    /// The B-spline curve (`BSpline`); `adaptor.bspline()?`.
    fn bspline(&self) -> Result<Handle<GeomBSplineCurve>, Failure> {
        Err(not_implemented("Adaptor3d_Curve::BSpline"))
    }
    /// The offset curve (`OffsetCurve`); `adaptor.offset_curve()?`.
    fn offset_curve(&self) -> Result<Handle<GeomOffsetCurve>, Failure> {
        Err(not_implemented("Adaptor3d_Curve::OffsetCurve"))
    }
    /// Point of parameter `u` (`EvalD0`); `adaptor.eval_d0(0.5)?`.
    fn eval_d0(&self, _u: f64) -> Result<Point3, Failure> {
        Err(not_implemented("Adaptor3d_Curve::EvalD0"))
    }
    /// Point and first derivative (`EvalD1`); `adaptor.eval_d1(0.5)?.d1`.
    fn eval_d1(&self, _u: f64) -> Result<GeomCurveD1, Failure> {
        Err(not_implemented("Adaptor3d_Curve::EvalD1"))
    }
    /// Point and two derivatives (`EvalD2`); `adaptor.eval_d2(0.5)?.d2`.
    fn eval_d2(&self, _u: f64) -> Result<GeomCurveD2, Failure> {
        Err(not_implemented("Adaptor3d_Curve::EvalD2"))
    }
    /// Point and three derivatives (`EvalD3`); `adaptor.eval_d3(0.5)?.d3`.
    fn eval_d3(&self, _u: f64) -> Result<GeomCurveD3, Failure> {
        Err(not_implemented("Adaptor3d_Curve::EvalD3"))
    }
    /// Derivative of order `n` (`EvalDN`); `adaptor.eval_dn(0.5, 2)?`.
    fn eval_dn(&self, _u: f64, _n: i32) -> Result<Vector3, Failure> {
        Err(not_implemented("Adaptor3d_Curve::EvalDN"))
    }
}
impl_transient_interface!(dyn Adaptor3dCurve, "Adaptor3d_Curve", StandardTransient);

/// An instance of the non-abstract native base class itself, whose members all raise
/// Standard_NotImplemented; `Handle::new(Adaptor3dCurveBase::default())`.
///
/// Its dynamic type is `Adaptor3d_Curve`, as for `new Adaptor3d_Curve()`.
#[derive(Default)]
pub struct Adaptor3dCurveBase {
    parent: StandardTransient,
}
impl_transient!(
    Adaptor3dCurveBase,
    "Adaptor3d_Curve",
    StandardTransient,
    parent,
    interfaces: [dyn Adaptor3dCurve]
);
impl Adaptor3dCurve for Adaptor3dCurveBase {}

// An adaptor is a basis of the Geom evaluation templates (offset, extrusion and
// revolution utilities), as `*BasisCurve` is natively.
impl BasisCurveEvaluator for dyn Adaptor3dCurve {
    fn basis_first_parameter(&self) -> Result<f64, Failure> {
        self.first_parameter()
    }
    fn basis_last_parameter(&self) -> Result<f64, Failure> {
        self.last_parameter()
    }
    fn basis_d0(&self, u: f64) -> Result<Point3, Failure> {
        self.eval_d0(u)
    }
    fn basis_d1(&self, u: f64) -> Result<GeomCurveD1, Failure> {
        self.eval_d1(u)
    }
    fn basis_d2(&self, u: f64) -> Result<GeomCurveD2, Failure> {
        self.eval_d2(u)
    }
    fn basis_d3(&self, u: f64) -> Result<GeomCurveD3, Failure> {
        self.eval_d3(u)
    }
    fn basis_dn(&self, u: f64, n: i32) -> Result<Vector3, Failure> {
        self.eval_dn(u, n)
    }
}
