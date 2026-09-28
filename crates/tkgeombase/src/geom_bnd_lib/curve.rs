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
// Rust translation of GeomBndLib_Curve at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomBndLib_Curve`: detect the kind of a 3D curve (adaptor or Geom handle) and
//! delegate to its evaluator.
use super::circle::Circle;
use super::ellipse::Ellipse;
use super::hyperbola::Hyperbola;
use super::line::Line;
use super::offset_curve::OffsetCurve;
use super::other_curve::OtherCurve;
use super::parabola::Parabola;
use super::spline_curve::{BSplineCurve, BezierCurve};
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped, exceptions};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomBezierCurve, GeomCircle, GeomCurve, GeomEllipse, GeomHyperbola, GeomLine,
    GeomOffsetCurve, GeomParabola, GeomTrimmedCurve,
};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::geom_abs::CurveType;

// The `std::variant` of evaluators; `Other` borrows the dispatcher's adaptor at call time.
enum Evaluator {
    Line(Line),
    Circle(Circle),
    Ellipse(Ellipse),
    Hyperbola(Hyperbola),
    Parabola(Parabola),
    Bezier(BezierCurve),
    BSpline(BSplineCurve),
    Offset(OffsetCurve),
    Other,
}

/// `GeomBndLib_Curve`: the box of any 3D curve through the evaluator of its kind.
/// An adaptor or a trimmed curve supplies the default range of `Box(Tol)`.
pub struct Curve<'a> {
    adaptor_ref: Option<&'a dyn Adaptor3dCurve>,
    adaptor_owned: Option<GeomAdaptorCurve>,
    evaluator: Evaluator,
    curve_type: CurveType,
}

// Run `$body` on the evaluator of `$curve` bound to `$eval` (an OtherCurve built on the
// adaptor for the `Other` kind).
macro_rules! dispatch {
    ($curve:expr, $eval:ident => $body:expr) => {
        match &$curve.evaluator {
            Evaluator::Line($eval) => $body,
            Evaluator::Circle($eval) => $body,
            Evaluator::Ellipse($eval) => $body,
            Evaluator::Hyperbola($eval) => $body,
            Evaluator::Parabola($eval) => $body,
            Evaluator::Bezier($eval) => $body,
            Evaluator::BSpline($eval) => $body,
            Evaluator::Offset($eval) => $body,
            Evaluator::Other => {
                let $eval = &OtherCurve::new($curve.required_adaptor()?);
                $body
            }
        }
    };
}

// `tryEmplaceEvaluator`: the `occ::down_cast` chain of the handle constructor.
fn detect(curve: &Handle<dyn GeomCurve>) -> Option<(CurveType, Evaluator)> {
    fn kind<T: RuntimeTyped>(curve: &Handle<dyn GeomCurve>) -> Option<Handle<T>> {
        let typed: Handle<T> = curve.cast::<T>();
        (!typed.is_null()).then_some(typed)
    }
    if let Some(h) = kind::<GeomLine>(curve) {
        return Some((CurveType::Line, Evaluator::Line(Line::new(h))));
    }
    if let Some(h) = kind::<GeomCircle>(curve) {
        return Some((CurveType::Circle, Evaluator::Circle(Circle::new(h))));
    }
    if let Some(h) = kind::<GeomEllipse>(curve) {
        return Some((CurveType::Ellipse, Evaluator::Ellipse(Ellipse::new(h))));
    }
    if let Some(h) = kind::<GeomHyperbola>(curve) {
        return Some((
            CurveType::Hyperbola,
            Evaluator::Hyperbola(Hyperbola::new(h)),
        ));
    }
    if let Some(h) = kind::<GeomParabola>(curve) {
        return Some((CurveType::Parabola, Evaluator::Parabola(Parabola::new(h))));
    }
    if let Some(h) = kind::<GeomBezierCurve>(curve) {
        return Some((
            CurveType::BezierCurve,
            Evaluator::Bezier(BezierCurve::new(h)),
        ));
    }
    if let Some(h) = kind::<GeomBSplineCurve>(curve) {
        return Some((
            CurveType::BSplineCurve,
            Evaluator::BSpline(BSplineCurve::new(h)),
        ));
    }
    let offset = kind::<GeomOffsetCurve>(curve)?;
    Some((
        CurveType::OffsetCurve,
        Evaluator::Offset(OffsetCurve::new(offset)),
    ))
}

// The evaluator of an adaptor kind: analytic kinds are rebuilt as Geom objects from the
// adaptor's gp value; an offset needs a GeomAdaptor_Curve on a Geom_OffsetCurve.
fn from_adaptor_kind(
    curve: &dyn Adaptor3dCurve,
    kind: CurveType,
) -> Result<Option<Evaluator>, Failure> {
    Ok(Some(match kind {
        CurveType::Line => {
            Evaluator::Line(Line::new(Handle::new(GeomLine::from_lin(curve.line()?))))
        }
        CurveType::Circle => Evaluator::Circle(Circle::new(Handle::new(GeomCircle::from_circ(
            curve.circle()?,
        )))),
        CurveType::Ellipse => Evaluator::Ellipse(Ellipse::new(Handle::new(
            GeomEllipse::from_elips(curve.ellipse()?),
        ))),
        CurveType::Hyperbola => Evaluator::Hyperbola(Hyperbola::new(Handle::new(
            GeomHyperbola::from_hypr(curve.hyperbola()?),
        ))),
        CurveType::Parabola => Evaluator::Parabola(Parabola::new(Handle::new(
            GeomParabola::from_parab(curve.parabola()?),
        ))),
        CurveType::BezierCurve => Evaluator::Bezier(BezierCurve::new(curve.bezier()?)),
        CurveType::BSplineCurve => Evaluator::BSpline(BSplineCurve::new(curve.bspline()?)),
        CurveType::OffsetCurve => return Ok(adaptor_offset(curve)),
        _ => return Ok(None),
    }))
}

// `dynamic_cast<const GeomAdaptor_Curve*>` then `occ::down_cast<Geom_OffsetCurve>`.
fn adaptor_offset(curve: &dyn Adaptor3dCurve) -> Option<Evaluator> {
    let geom_adaptor = GeomAdaptorCurve::project_from(curve)?;
    let offset: Handle<GeomOffsetCurve> = geom_adaptor.curve().cast();
    (!offset.is_null()).then(|| Evaluator::Offset(OffsetCurve::new(offset)))
}

impl<'a> Curve<'a> {
    /// `GeomBndLib_Curve(const Adaptor3d_Curve&)`: the adaptor's `GetType` selects the
    /// evaluator; an offset adaptor that is not a GeomAdaptor_Curve on a Geom_OffsetCurve,
    /// and every other kind, falls back to OtherCurve with the type OtherCurve;
    /// `Curve::from_adaptor(&adaptor)?`.
    pub fn from_adaptor(curve: &'a dyn Adaptor3dCurve) -> Result<Self, Failure> {
        let kind: CurveType = curve.get_type()?;
        let (curve_type, evaluator) = match from_adaptor_kind(curve, kind)? {
            Some(evaluator) => (kind, evaluator),
            None => (CurveType::OtherCurve, Evaluator::Other),
        };
        Ok(Self {
            adaptor_ref: Some(curve),
            adaptor_owned: None,
            evaluator,
            curve_type,
        })
    }
    /// `GetType()`: the detected kind; `curve.get_type()`.
    pub fn get_type(&self) -> CurveType {
        self.curve_type
    }
    fn adaptor(&self) -> Option<&dyn Adaptor3dCurve> {
        match (self.adaptor_ref, &self.adaptor_owned) {
            (Some(adaptor), _) => Some(adaptor),
            (None, Some(owned)) => Some(owned),
            (None, None) => None,
        }
    }
    fn required_adaptor(&self) -> Result<&dyn Adaptor3dCurve, Failure> {
        self.adaptor().ok_or_else(|| {
            exceptions::null_object("GeomBndLib_Curve: the OtherCurve evaluator has no adaptor")
        })
    }
    /// `Box(Tol)`: the adaptor's range when there is an adaptor, else the evaluator's
    /// whole curve; `curve.bounding_box(0.0)?`.
    pub fn bounding_box(&self, tolerance: f64) -> Result<BoundingBox3, Failure> {
        if let Some(adaptor) = self.adaptor() {
            let (u1, u2) = (adaptor.first_parameter()?, adaptor.last_parameter()?);
            return self.bounding_box_range(u1, u2, tolerance);
        }
        dispatch!(self, eval => eval.bounding_box(tolerance))
    }
    /// `Box(U1, U2, Tol)`; `curve.bounding_box_range(0.0, 1.0, 0.0)?`.
    pub fn bounding_box_range(
        &self,
        u1: f64,
        u2: f64,
        tolerance: f64,
    ) -> Result<BoundingBox3, Failure> {
        dispatch!(self, eval => eval.bounding_box_range(u1, u2, tolerance))
    }
    /// `BoxOptimal(Tol)`: the adaptor's range when there is an adaptor, else the
    /// geometry's `[FirstParameter, LastParameter]`; `curve.bounding_box_optimal(0.0)?`.
    pub fn bounding_box_optimal(&self, tolerance: f64) -> Result<BoundingBox3, Failure> {
        if let Some(adaptor) = self.adaptor() {
            let (u1, u2) = (adaptor.first_parameter()?, adaptor.last_parameter()?);
            return self.bounding_box_optimal_range(u1, u2, tolerance);
        }
        let (u1, u2) = self.geometry_range()?;
        self.bounding_box_optimal_range(u1, u2, tolerance)
    }
    /// `BoxOptimal(U1, U2, Tol)`; `curve.bounding_box_optimal_range(0.0, 1.0, 0.0)?`.
    pub fn bounding_box_optimal_range(
        &self,
        u1: f64,
        u2: f64,
        tolerance: f64,
    ) -> Result<BoundingBox3, Failure> {
        dispatch!(self, eval => eval.bounding_box_optimal_range(u1, u2, tolerance))
    }
    /// `Add(Tol, B)`: `B.Add(Box(Tol))`; `curve.add(0.0, &mut bounds)?`.
    pub fn add(&self, tolerance: f64, bounds: &mut BoundingBox3) -> Result<(), Failure> {
        bounds.add_box(&self.bounding_box(tolerance)?);
        Ok(())
    }
    /// `Add(U1, U2, Tol, B)`; `curve.add_range(0.0, 1.0, 0.0, &mut bounds)?`.
    pub fn add_range(
        &self,
        u1: f64,
        u2: f64,
        tolerance: f64,
        bounds: &mut BoundingBox3,
    ) -> Result<(), Failure> {
        bounds.add_box(&self.bounding_box_range(u1, u2, tolerance)?);
        Ok(())
    }
    /// `AddOptimal(Tol, B)`; `curve.add_optimal(0.0, &mut bounds)?`.
    pub fn add_optimal(&self, tolerance: f64, bounds: &mut BoundingBox3) -> Result<(), Failure> {
        bounds.add_box(&self.bounding_box_optimal(tolerance)?);
        Ok(())
    }
    /// `AddOptimal(U1, U2, Tol, B)`; `curve.add_optimal_range(0.0, 1.0, 0.0, &mut bounds)?`.
    pub fn add_optimal_range(
        &self,
        u1: f64,
        u2: f64,
        tolerance: f64,
        bounds: &mut BoundingBox3,
    ) -> Result<(), Failure> {
        bounds.add_box(&self.bounding_box_optimal_range(u1, u2, tolerance)?);
        Ok(())
    }
    // `theEval.Geometry()->FirstParameter()/LastParameter()` of the evaluator.
    fn geometry_range(&self) -> Result<(f64, f64), Failure> {
        fn range<T: GeomCurve + RuntimeTyped>(h: &Handle<T>) -> Result<(f64, f64), Failure> {
            let curve = super::live(h, "GeomBndLib_Curve")?;
            Ok((curve.first_parameter(), curve.last_parameter()))
        }
        match &self.evaluator {
            Evaluator::Line(e) => range(e.geometry()),
            Evaluator::Circle(e) => range(e.geometry()),
            Evaluator::Ellipse(e) => range(e.geometry()),
            Evaluator::Hyperbola(e) => range(e.geometry()),
            Evaluator::Parabola(e) => range(e.geometry()),
            Evaluator::Bezier(e) => range(e.geometry()),
            Evaluator::BSpline(e) => range(e.geometry()),
            Evaluator::Offset(e) => range(e.geometry()),
            Evaluator::Other => {
                let adaptor = self.required_adaptor()?;
                Ok((adaptor.first_parameter()?, adaptor.last_parameter()?))
            }
        }
    }
}

impl Curve<'static> {
    /// `GeomBndLib_Curve(const occ::handle<Geom_Curve>&)`: the down_cast chain on the
    /// curve, or on the basis of a Geom_TrimmedCurve (whose adaptor then gives the range
    /// of `Box(Tol)`); other kinds use OtherCurve on a GeomAdaptor_Curve;
    /// `Curve::from_curve(&handle)?`.
    pub fn from_curve(curve: &Handle<dyn GeomCurve>) -> Result<Self, Failure> {
        let mut detection: Handle<dyn GeomCurve> = curve.clone();
        let mut adaptor_owned: Option<GeomAdaptorCurve> = None;
        let trimmed: Handle<GeomTrimmedCurve> = curve.cast();
        if let Some(trimmed) = trimmed.get() {
            adaptor_owned = Some(GeomAdaptorCurve::with_curve(curve)?);
            detection = trimmed.basis_curve();
        }
        let (curve_type, evaluator) = match detect(&detection) {
            Some(found) => found,
            None => {
                if adaptor_owned.is_none() {
                    adaptor_owned = Some(GeomAdaptorCurve::with_curve(curve)?);
                }
                (CurveType::OtherCurve, Evaluator::Other)
            }
        };
        Ok(Self {
            adaptor_ref: None,
            adaptor_owned,
            evaluator,
            curve_type,
        })
    }
}
