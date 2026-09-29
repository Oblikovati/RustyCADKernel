// Copyright (c) 1994-1999 Matra Datavision
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
// Rust translation of ProjLib_ProjectOnPlane::Load (parabolas, hyperbolas, Bezier and
// B-spline curves), GetTrimmedResult, BuildParabolaByApex, BuildHyperbolaByApex and
// BuildByApprox at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The open conics and the pole-based curves of `ProjLib_ProjectOnPlane::Load`.
use super::approx::perform_approx;
use super::{OnPlaneState, ProjectOnPlane};
use crate::failure_capture::FailureCapture;
use crate::gce::{MakeDir, MakeParab, Root};
use crate::lprop::CLProps3d;
use crate::native_failure::failure;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomBezierCurve, GeomCurve, GeomHyperbola, GeomLine, GeomParabola,
};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Axis2, Direction3, Hyperbola3, Line3, Parabola3, Point3, Vector3};
use rustycad_tkmath::math::{BrentMinimum, Function};

// The parameter range an infinite parabola or hyperbola is cut to before the
// approximation (`aParabolaLimit`, `aHyperbolaLimit`).
const PARABOLA_LIMIT: f64 = 20000.;
const HYPERBOLA_LIMIT: f64 = 10.;

/// The projected axes and origin of a parabola or hyperbola frame.
struct OpenConic {
    xc: Vector3,
    yc: Vector3,
    p: Point3,
}

impl OnPlaneState {
    fn open_conic(&self, axe_ref: Axis2) -> Result<OpenConic, Failure> {
        let oblique = self.oblique();
        Ok(OpenConic {
            xc: oblique.project_vector(Vector3::from(axe_ref.x_direction()))?,
            yc: oblique.project_vector(Vector3::from(axe_ref.y_direction()))?,
            p: oblique.project_point(axe_ref.location()),
        })
    }

    // P(u) = O + (u^2 / 4f) Xc + u Yc projects to Q(u) = f(O) + (u^2 / 4f) f(Xc) + u f(Yc).
    pub(super) fn load_parabola(&mut self) -> Result<(), Failure> {
        let parab: Parabola3 = self.curve()?.parabola()?;
        let c: OpenConic = self.open_conic(parab.position())?;
        self.is_approx = false;
        let conf: f64 = precision::confusion();
        let mut projected: Handle<dyn GeomCurve> = Handle::default();
        if (c.yc.magnitude() - 1.).abs() < conf && c.xc.magnitude() < conf {
            // The parabola projects on a line along Yc; the native code builds that
            // line but then trims the null parabola handle, which it dereferences
            // (get_trimmed_result reports Standard_NullObject).
            self.curve_type = CurveType::Line;
        } else if c.xc.is_normal(c.yc, precision::angular())? {
            self.curve_type = CurveType::Parabola;
            let focal: f64 = parab.focal() / c.xc.magnitude();
            let normal: Direction3 = Direction3::from_vector(c.xc.crossed(c.yc))?;
            let frame: Axis2 =
                Axis2::with_x_direction(c.p, normal, Direction3::from_vector(c.xc)?)?;
            projected = Handle::new(GeomParabola::from_parab(Parabola3::new(frame, focal)?)).cast();
        } else if c.yc.magnitude() < conf || c.yc.is_parallel(c.xc, precision::angular())? {
            self.is_approx = true;
        } else if !self.keep_param {
            // Try building the parabola with the help of the apex position.
            projected = self.build_parabola_by_apex()?;
            self.is_approx = projected.is_null();
        } else {
            self.is_approx = true;
        }
        if self.is_approx {
            return self.build_by_approx(PARABOLA_LIMIT);
        }
        self.get_trimmed_result(&projected)
    }

    // P(u) = O + R1 cosh(u) Xc + R2 sinh(u) Yc projects to
    // Q(u) = f(O) + R1 cosh(u) f(Xc) + R2 sinh(u) f(Yc).
    pub(super) fn load_hyperbola(&mut self) -> Result<(), Failure> {
        let hypr: Hyperbola3 = self.curve()?.hyperbola()?;
        let c: OpenConic = self.open_conic(hypr.position())?;
        let (r1, r2) = (hypr.major_radius(), hypr.minor_radius());
        let z: Direction3 = self.plane.direction();
        self.is_approx = false;
        let conf: f64 = precision::confusion();
        let mut projected: Handle<dyn GeomCurve> = Handle::default();
        if c.xc.magnitude() < conf {
            self.curve_type = CurveType::Hyperbola;
            let x: Direction3 = Direction3::from_vector(c.yc)?.crossed(z)?;
            let frame: Axis2 = Axis2::with_x_direction(c.p, z, x)?;
            projected = hyperbola(frame, 0., r2 * c.yc.magnitude())?;
        } else if c.yc.magnitude() < conf {
            self.curve_type = CurveType::Hyperbola;
            let frame: Axis2 = Axis2::with_x_direction(c.p, z, Direction3::from_vector(c.xc)?)?;
            projected = hyperbola(frame, r1 * c.xc.magnitude(), 0.)?;
        } else if c.xc.is_normal(c.yc, precision::angular())? {
            self.curve_type = CurveType::Hyperbola;
            let normal: Direction3 = Direction3::from_vector(c.xc.crossed(c.yc))?;
            let frame: Axis2 =
                Axis2::with_x_direction(c.p, normal, Direction3::from_vector(c.xc)?)?;
            projected = hyperbola(frame, r1 * c.xc.magnitude(), r2 * c.yc.magnitude())?;
        } else if c.yc.magnitude() < conf || c.yc.is_parallel(c.xc, precision::angular())? {
            self.is_approx = true;
        } else if !self.keep_param {
            projected = self.build_hyperbola_by_apex()?;
            self.is_approx = projected.is_null();
        } else {
            self.is_approx = true;
        }
        if self.is_approx {
            return self.build_by_approx(HYPERBOLA_LIMIT);
        }
        self.get_trimmed_result(&projected)
    }

    // A copy of the Bezier curve with its poles projected; the parametrization is kept.
    pub(super) fn load_bezier(&mut self) -> Result<(), Failure> {
        let source: Handle<GeomBezierCurve> = self.curve()?.bezier()?;
        let bezier: &GeomBezierCurve = source.get().ok_or_else(null_curve)?;
        let projected: GeomBezierCurve = GeomBezierCurve::copy_of(bezier)?;
        (self.keep_param, self.is_approx) = (true, false);
        self.curve_type = CurveType::BezierCurve;
        let oblique = self.oblique();
        for i in 1..=bezier.nb_poles() {
            projected.set_pole(i, oblique.project_point(bezier.pole(i)?))?;
        }
        let curve: Handle<dyn GeomCurve> = Handle::new(projected).cast();
        self.result = Handle::new(GeomAdaptorCurve::with_curve(&curve)?);
        Ok(())
    }

    // A copy of the B-spline curve with its poles projected; the parametrization is kept.
    pub(super) fn load_bspline(&mut self) -> Result<(), Failure> {
        let source: Handle<GeomBSplineCurve> = self.curve()?.bspline()?;
        let bspline: &GeomBSplineCurve = source.get().ok_or_else(null_curve)?;
        let projected: GeomBSplineCurve = GeomBSplineCurve::copy_of(bspline)?;
        (self.keep_param, self.is_approx) = (true, false);
        self.curve_type = CurveType::BSplineCurve;
        let oblique = self.oblique();
        for i in 1..=bspline.nb_poles() {
            projected.set_pole(i, oblique.project_point(bspline.pole(i)?))?;
        }
        let curve: Handle<dyn GeomCurve> = Handle::new(projected).cast();
        self.result = Handle::new(GeomAdaptorCurve::with_curve(&curve)?);
        Ok(())
    }

    // Any other curve: the B-spline approximation, with the parametrization kept.
    pub(super) fn load_other(&mut self) -> Result<(), Failure> {
        (self.keep_param, self.is_approx) = (true, true);
        self.curve_type = CurveType::BSplineCurve;
        self.load_approximation(self.curve.clone())
    }
}

fn hyperbola(frame: Axis2, major: f64, minor: f64) -> Result<Handle<dyn GeomCurve>, Failure> {
    Ok(Handle::new(GeomHyperbola::from_hypr(Hyperbola3::new(
        frame, major, minor,
    )?))
    .cast())
}

fn null_curve() -> Failure {
    failure("Standard_NullObject", "ProjLib_ProjectOnPlane: null curve")
}

impl OnPlaneState {
    /// `GetTrimmedResult(theProjCurve)`: the projected line, parabola or hyperbola
    /// bounded by the parameters of the projected curve ends (the curve's own
    /// bounds when infinite).
    fn get_trimmed_result(&mut self, projected: &Handle<dyn GeomCurve>) -> Result<(), Failure> {
        let curve: &dyn GeomCurve = projected.get().ok_or_else(|| {
            failure(
                "Standard_NullObject",
                "ProjLib_ProjectOnPlane::GetTrimmedResult: null projected curve (a parabola projected on a line)",
            )
        })?;
        self.first_par = curve.first_parameter();
        self.last_par = curve.last_parameter();
        let (first, last) = self.curve_bounds()?;
        if !precision::is_infinite(first) {
            self.first_par = self.projected_parameter(projected, first)?;
        }
        if !precision::is_infinite(last) {
            self.last_par = self.projected_parameter(projected, last)?;
        }
        self.result = Handle::new(GeomAdaptorCurve::with_range(
            projected,
            self.first_par,
            self.last_par,
        )?);
        Ok(())
    }

    // The ElCLib parameter of the projected point of parameter `u` on the result.
    fn projected_parameter(
        &self,
        projected: &Handle<dyn GeomCurve>,
        u: f64,
    ) -> Result<f64, Failure> {
        let point: Point3 = self.oblique().project_point(self.curve()?.value(u)?);
        match self.curve_type {
            CurveType::Line => {
                let line: Handle<GeomLine> = projected.cast();
                Ok(elclib::parameter(
                    &line.get().ok_or_else(null_curve)?.lin(),
                    point,
                )?)
            }
            CurveType::Parabola => {
                let parab: Handle<GeomParabola> = projected.cast();
                Ok(elclib::parameter(
                    &parab.get().ok_or_else(null_curve)?.parab()?,
                    point,
                )?)
            }
            _ => {
                let hypr: Handle<GeomHyperbola> = projected.cast();
                Ok(elclib::parameter(
                    &hypr.get().ok_or_else(null_curve)?.hypr()?,
                    point,
                )?)
            }
        }
    }

    /// `BuildByApprox(theLimitParameter)`: the B-spline approximation, on the curve
    /// cut to `[-limit, limit]` when it is infinite; a failed fit leaves a null curve
    /// that the native code dereferences (Standard_NullObject here).
    fn build_by_approx(&mut self, limit: f64) -> Result<(), Failure> {
        self.curve_type = CurveType::BSplineCurve;
        let (first, last) = self.curve_bounds()?;
        let source: Handle<dyn Adaptor3dCurve> =
            if precision::is_infinite(first) || precision::is_infinite(last) {
                let (f, l) = ((-limit).max(first), limit.min(last));
                self.curve()?.trim(f, l, precision::confusion())?
            } else {
                self.curve.clone()
            };
        let adaptor: &dyn Adaptor3dCurve = source.get().ok_or_else(null_curve)?;
        let approximated = perform_approx(adaptor, self.oblique())?;
        let spline: &GeomBSplineCurve = approximated.get().ok_or_else(|| {
            failure(
                "Standard_NullObject",
                "ProjLib_ProjectOnPlane::BuildByApprox: the approximation failed",
            )
        })?;
        self.first_par = spline.first_parameter();
        self.last_par = spline.last_parameter();
        self.result = Handle::new(GeomAdaptorCurve::with_curve(&approximated.cast())?);
        Ok(())
    }
}

/// `ProjLib_MaxCurvature`: minus the curvature of the projected curve, so that the
/// Brent minimum is the curvature maximum (the apex).
struct MaxCurvature<'a> {
    props: &'a mut CLProps3d,
    failure: FailureCapture,
}

impl Function for MaxCurvature<'_> {
    fn value(&mut self, x: f64, result: &mut f64) -> bool {
        let curvature = self
            .props
            .set_parameter(x)
            .and_then(|()| self.props.curvature());
        let Some(curvature) = self.failure.keep(curvature) else {
            return false;
        };
        *result = -curvature;
        true
    }
}

impl OnPlaneState {
    // The projection evaluated as by the OtherCurve type: `ShallowCopy()` taken while
    // myType is OtherCurve, as `BuildParabolaByApex` does.
    fn apex_props(&self) -> Result<CLProps3d, Failure> {
        let mut copy: OnPlaneState = self.shallow_copy()?;
        copy.curve_type = CurveType::OtherCurve;
        let adaptor: Handle<dyn Adaptor3dCurve> =
            Handle::new(ProjectOnPlane::from_state(copy)).cast();
        CLProps3d::new(adaptor, 2, precision::confusion())
    }

    // The Brent location of the curvature maximum in `[a, c]` from `b`, with the
    // properties left at it; None when the search fails.
    fn curvature_apex(
        &self,
        props: &mut CLProps3d,
        bounds: [f64; 3],
    ) -> Result<Option<f64>, Failure> {
        let mut function = MaxCurvature {
            props,
            failure: FailureCapture::default(),
        };
        let mut solver: BrentMinimum = BrentMinimum::new(precision::p_confusion());
        solver.perform(&mut function, bounds[0], bounds[1], bounds[2]);
        function.failure.take()?;
        if !solver.is_done() {
            return Ok(None);
        }
        let t: f64 = solver.location()?;
        props.set_parameter(t)?;
        Ok(Some(t))
    }

    /// `BuildParabolaByApex`: the parabola through the curvature maximum of the
    /// projection (its apex), with the focal from one more point; a null handle when
    /// the search or the construction fails.
    fn build_parabola_by_apex(&mut self) -> Result<Handle<dyn GeomCurve>, Failure> {
        let focal: f64 = self.curve()?.parabola()?.focal();
        let mut props: CLProps3d = self.apex_props()?;
        let Some(t) = self.curvature_apex(&mut props, [-10. * focal, 0., 10. * focal])? else {
            return Ok(Handle::default());
        };
        let p0: Point3 = props.value();
        let y_dir: Direction3 = Direction3::from_vector(props.d1()?)?;
        let curvature: f64 = props.curvature()?;
        if precision::is_infinite(curvature) || curvature < precision::confusion() {
            return Ok(Handle::default());
        }
        let x_dir: Direction3 = props.normal()?;
        let x_line: Line3 = Line3::new(p0, x_dir);
        let p1: Point3 = self.oblique().value(self.curve()?, t + 10. * focal)?;
        let x: f64 = elclib::line_parameter(x_line.position(), p1);
        let y: f64 = x_line.distance(p1);
        let new_focal: f64 = y * y / 4. / x;
        let frame: Axis2 = Axis2::with_x_direction(p0, x_dir.crossed(y_dir)?, x_dir)?;
        let maker: MakeParab = MakeParab::new(frame, new_focal)?;
        if !maker.is_done() {
            return Ok(Handle::default());
        }
        self.curve_type = CurveType::Parabola;
        Ok(Handle::new(GeomParabola::from_parab(maker.value()?)).cast())
    }

    /// `BuildHyperbolaByApex`: the hyperbola through the curvature maximum of the
    /// projection, centred at the projected centre; a null handle when the search
    /// fails or the curvature is not finite and positive.
    fn build_hyperbola_by_apex(&mut self) -> Result<Handle<dyn GeomCurve>, Failure> {
        let mut props: CLProps3d = self.apex_props()?;
        let Some(t) = self.curvature_apex(&mut props, [-5., 0., 5.])? else {
            return Ok(Handle::default());
        };
        let curvature: f64 = props.curvature()?;
        if precision::is_infinite(curvature) || curvature < precision::confusion() {
            return Ok(Handle::default());
        }
        let hypr: Hyperbola3 = self.curve()?.hyperbola()?;
        let oblique = self.oblique();
        let p: Point3 = oblique.project_point(hypr.position().location());
        let z: Direction3 = self.plane.direction();
        let p0: Point3 = props.value();
        let x_dir: Direction3 = MakeDir::from_points(p, p0)?.value()?;
        let y_dir: Direction3 = MakeDir::from_vector(props.d1()?)?.value()?;
        let maj_rad: f64 = p.distance(&p0);
        let p1: Point3 = oblique.value(self.curve()?, t + 1.)?;
        let v: Vector3 = Vector3::between(p, p1);
        let (x, y) = (v.dot(Vector3::from(x_dir)), v.dot(Vector3::from(y_dir)));
        let min_rad: f64 = y / (x * x / maj_rad / maj_rad - 1.).sqrt();
        let frame: Axis2 = Axis2::with_x_direction(p, z, x_dir)?;
        let projected = hyperbola(frame, maj_rad, min_rad)?;
        self.curve_type = CurveType::Hyperbola;
        Ok(projected)
    }
}
