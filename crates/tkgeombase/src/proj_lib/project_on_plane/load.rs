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
// Rust translation of ProjLib_ProjectOnPlane::Load (lines, circles and ellipses)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ProjLib_ProjectOnPlane::Load`: the dispatch on the curve kind, the line case and
//! the common circle and ellipse case.
use super::OnPlaneState;
use super::approx::{new_array, perform_approx};
use crate::geom_convert::curve_to_bspline_curve;
use crate::geom_lib::Tool as GeomLibTool;
use crate::native_failure::failure;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomCircle, GeomCurve, GeomEllipse, GeomLine, GeomTrimmedCurve,
};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::convert::ParameterisationType;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{
    Axis1, Axis2, Circle3, Direction3, Ellipse3, Line3, Point3, Transform2, Transform3, Vector3,
};
use rustycad_tkmath::math::{Jacobi, Matrix};

/// The frame and radii of a circle or an ellipse, and the kind it came from.
#[derive(Clone, Copy)]
struct ConicFrame {
    axis: Axis2,
    r1: f64,
    r2: f64,
}

/// The canonical axes and radii of the projected conic: `(Dx, Dy, Major, Minor)`.
type ProjectedAxes = (Direction3, Direction3, f64, f64);

impl OnPlaneState {
    /// `Load(C, Tolerance, KeepParametrization)` on the plain state.
    pub(super) fn load(
        &mut self,
        curve: &Handle<dyn Adaptor3dCurve>,
        tolerance: f64,
        keep_parametrization: bool,
    ) -> Result<(), Failure> {
        self.curve = curve.clone();
        self.curve_type = CurveType::OtherCurve;
        self.is_approx = false;
        self.tolerance = tolerance;
        let kind: CurveType = self.curve()?.get_type()?;
        self.keep_param = keep_parametrization;
        match kind {
            CurveType::Line => self.load_line(),
            CurveType::Circle => {
                let circle: Circle3 = self.curve()?.circle()?;
                self.load_circular(circle_frame(&circle))
            }
            CurveType::Ellipse => {
                let ellipse: Ellipse3 = self.curve()?.ellipse()?;
                self.load_circular(ellipse_frame(&ellipse))
            }
            CurveType::Parabola => self.load_parabola(),
            CurveType::Hyperbola => self.load_hyperbola(),
            CurveType::BezierCurve => self.load_bezier(),
            CurveType::BSplineCurve => self.load_bspline(),
            _ => self.load_other(),
        }
    }

    /// The curve bounds.
    pub(super) fn curve_bounds(&self) -> Result<(f64, f64), Failure> {
        let curve: &dyn Adaptor3dCurve = self.curve()?;
        Ok((curve.first_parameter()?, curve.last_parameter()?))
    }

    // P(u) = O + u Xc projects to Q(u) = f(O) + u f(Xc).
    fn load_line(&mut self) -> Result<(), Failure> {
        let line: Line3 = self.curve()?.line()?;
        let oblique = self.oblique();
        let xc: Vector3 = oblique.project_vector(Vector3::from(line.direction()))?;
        let origin: Point3 = oblique.project_point(line.location());
        if xc.magnitude() < precision::confusion() {
            // A line orthogonal to the plane: a degenerate linear B-spline.
            self.curve_type = CurveType::BSplineCurve;
            return self.load_point_line(origin);
        }
        self.curve_type = CurveType::Line;
        let projected: Line3 = Line3::new(origin, Direction3::from_vector(xc)?);
        let geom_line: Handle<dyn GeomCurve> = Handle::new(GeomLine::from_lin(projected)).cast();
        let (first, last) = self.curve_bounds()?;
        if (xc.magnitude() - 1.).abs() < precision::confusion() {
            (self.first_par, self.last_par) = (first, last);
            self.result = Handle::new(GeomAdaptorCurve::with_range(&geom_line, first, last)?);
            return Ok(());
        }
        let p1: Point3 = oblique.project_point(self.curve()?.value(first)?);
        let p2: Point3 = oblique.project_point(self.curve()?.value(last)?);
        let along: Vector3 = Vector3::from(projected.direction());
        self.first_par = along.dot(Vector3::between(origin, p1));
        self.last_par = along.dot(Vector3::between(origin, p2));
        if !self.keep_param {
            let (f, l) = (self.first_par, self.last_par);
            self.result = Handle::new(GeomAdaptorCurve::with_range(&geom_line, f, l)?);
            return Ok(());
        }
        self.curve_type = CurveType::BSplineCurve;
        self.refuse_nan_segment(line)?;
        self.load_reparametrized_line(&geom_line, first, last)
    }

    // Rust deviation: a projection direction parallel to the plane (the default
    // direction X with the default plane XOY, for example) makes every projected
    // parameter NaN. Natively Geom_TrimmedCurve::SetTrim then calls Reverse, which
    // calls SetTrim again, without end (the program never returns); the port
    // overflowed its stack. Refuse the degenerate input instead, with the
    // Standard_ConstructionError of the commented-out native constructor check.
    fn refuse_nan_segment(&self, line: Line3) -> Result<(), Failure> {
        if !self.first_par.is_nan() && !self.last_par.is_nan() {
            return Ok(());
        }
        Err(failure(
            "Standard_ConstructionError",
            &format!(
                "ProjLib_ProjectOnPlane: the direction {} is parallel to the plane of \
                 normal {}, so the line of direction {} projects to the NaN segment \
                 [{}, {}]; expected a direction not parallel to the plane to keep the \
                 parametrization",
                triple(self.direction),
                triple(self.plane.direction()),
                triple(line.direction()),
                self.first_par,
                self.last_par
            ),
        ))
    }

    // The two-pole degree-1 B-spline of a line projected to the point `origin`.
    fn load_point_line(&mut self, origin: Point3) -> Result<(), Failure> {
        let (first, last) = self.curve_bounds()?;
        let mut mults: Array1<'static, i32> = new_array(2)?;
        (mults[1], mults[2]) = (2, 2);
        let mut poles: Array1<'static, Point3> = new_array(2)?;
        (poles[1], poles[2]) = (origin, origin);
        let mut knots: Array1<'static, f64> = new_array(2)?;
        (knots[1], knots[2]) = (first, last);
        let bsp: Handle<dyn GeomCurve> =
            Handle::new(GeomBSplineCurve::new(&poles, &knots, &mults, 1, false)?).cast();
        self.result = Handle::new(GeomAdaptorCurve::with_curve(&bsp)?);
        Ok(())
    }

    // The projected segment as a degree-1 B-spline on the curve range.
    fn load_reparametrized_line(
        &mut self,
        geom_line: &Handle<dyn GeomCurve>,
        first: f64,
        last: f64,
    ) -> Result<(), Failure> {
        let trimmed = GeomTrimmedCurve::new(geom_line, self.first_par, self.last_par, true, true)?;
        let trimmed: Handle<dyn GeomCurve> = Handle::new(trimmed).cast();
        let converted = curve_to_bspline_curve(&trimmed, ParameterisationType::TgtThetaOver2)?;
        let spline: &GeomBSplineCurve = converted.get().ok_or_else(null_curve)?;
        let mut knots: Array1<'static, f64> = spline.knots();
        bsplclib::reparametrize(first, last, &mut knots)?;
        spline.set_knots(&knots)?;
        self.result = Handle::new(GeomAdaptorCurve::with_curve(&converted.cast())?);
        Ok(())
    }
}

// `(x, y, z)` of a direction, for the failure messages.
fn triple(direction: Direction3) -> String {
    format!("({}, {}, {})", direction.x(), direction.y(), direction.z())
}

fn null_curve() -> Failure {
    failure("Standard_NullObject", "ProjLib_ProjectOnPlane: null curve")
}

fn circle_frame(circle: &Circle3) -> ConicFrame {
    ConicFrame {
        axis: circle.position(),
        r1: circle.radius(),
        r2: circle.radius(),
    }
}

fn ellipse_frame(ellipse: &Ellipse3) -> ConicFrame {
    ConicFrame {
        axis: ellipse.position(),
        r1: ellipse.major_radius(),
        r2: ellipse.minor_radius(),
    }
}

impl OnPlaneState {
    // P(u) = O + R1 cos(u) Xc + R2 sin(u) Yc projects to
    // Q(u) = f(O) + R1 cos(u) f(Xc) + R2 sin(u) f(Yc).
    fn load_circular(&mut self, frame: ConicFrame) -> Result<(), Failure> {
        let oblique = self.oblique();
        let x: Direction3 = frame.axis.x_direction();
        let y: Direction3 = frame.axis.y_direction();
        let vdx: Vector3 = oblique.project_vector(Vector3::from(x))?;
        let vdy: Vector3 = oblique.project_vector(Vector3::from(y))?;
        let tol2: f64 = self.tolerance * self.tolerance;
        if vdx.square_magnitude() < tol2
            || vdy.square_magnitude() < tol2
            || vdx.cross_square_magnitude(vdy) < tol2
        {
            self.is_approx = true;
        }
        let mut canonical: Handle<dyn GeomCurve> = Handle::default();
        if !self.is_approx {
            canonical = self.canonical_conic(frame, vdx, vdy)?;
        }
        if self.is_approx {
            self.curve_type = CurveType::BSplineCurve;
            return self.load_approximation(self.curve.clone());
        }
        if canonical.is_null() {
            return Ok(());
        }
        self.trim_periodic_result(&canonical)
    }

    // The projected circle or ellipse, or a null handle when is_approx is set.
    fn canonical_conic(
        &mut self,
        frame: ConicFrame,
        vdx: Vector3,
        vdy: Vector3,
    ) -> Result<Handle<dyn GeomCurve>, Failure> {
        let (dx, dy) = (Direction3::from_vector(vdx)?, Direction3::from_vector(vdy)?);
        let oblique = self.oblique();
        let o: Point3 = frame.axis.location();
        let p: Point3 = oblique.project_point(o);
        let x_end: Point3 = o.translated(Vector3::from(frame.axis.x_direction()) * frame.r1);
        let y_end: Point3 = o.translated(Vector3::from(frame.axis.y_direction()) * frame.r2);
        let major: f64 = p.distance(&oblique.project_point(x_end));
        let minor: f64 = p.distance(&oblique.project_point(y_end));
        let axes: ProjectedAxes = if self.keep_param {
            self.is_approx = !dx.is_normal(dy, precision::angular());
            (dx, dy, major, minor)
        } else {
            match self.eigen_axes(frame, vdx, vdy, p)? {
                Some(axes) => axes,
                None => {
                    self.is_approx = true;
                    (dx, dy, major, minor)
                }
            }
        };
        if self.is_approx {
            return Ok(Handle::default());
        }
        self.build_conic(p, axes)
    }

    // The circle when the radii agree, the ellipse when Major > Minor, otherwise
    // the approximation.
    fn build_conic(
        &mut self,
        p: Point3,
        axes: ProjectedAxes,
    ) -> Result<Handle<dyn GeomCurve>, Failure> {
        let (dx, dy, major, minor) = axes;
        let axe: Axis2 = Axis2::with_x_direction(p, dx.crossed(dy)?, dx)?;
        if (major - minor).abs() < precision::confusion() {
            self.curve_type = CurveType::Circle;
            let circle: Handle<dyn GeomCurve> =
                Handle::new(GeomCircle::from_circ(Circle3::new(axe, major)?)).cast();
            self.result = Handle::new(GeomAdaptorCurve::with_curve(&circle)?);
            return Ok(circle);
        }
        if major > minor {
            self.curve_type = CurveType::Ellipse;
            let ellipse: Handle<dyn GeomCurve> =
                Handle::new(GeomEllipse::from_elips(Ellipse3::new(axe, major, minor)?)).cast();
            self.result = Handle::new(GeomAdaptorCurve::with_curve(&ellipse)?);
            return Ok(ellipse);
        }
        self.is_approx = true;
        Ok(Handle::default())
    }

    // The parallel projection of an ellipse (geometrictools.com,
    // ParallelProjectionEllipse.pdf): the eigen decomposition of
    // M = A^-T Delta^2 A^-1 gives the radii and the in-plane rotation of the axes.
    fn eigen_axes(
        &self,
        frame: ConicFrame,
        vdx: Vector3,
        vdy: Vector3,
        p: Point3,
    ) -> Result<Option<ProjectedAxes>, Failure> {
        let m: Matrix<'static> = projected_quadratic(&self.plane, frame, vdx, vdy)?;
        let eigen: Jacobi = Jacobi::new(&m)?;
        if !eigen.is_done() {
            return Ok(None);
        }
        let minor: f64 = 1.0 / eigen.value(1)?.sqrt();
        let major: f64 = 1.0 / eigen.value(2)?.sqrt();
        let ev: &Matrix<'static> = eigen.vectors()?;
        let mut in_plane: Transform2 = Transform2::default();
        in_plane.set_values(ev[(1, 2)], ev[(1, 1)], 0.0, ev[(2, 2)], ev[(2, 1)], 0.0)?;
        let mut rotation: Transform3 = Transform3::default();
        rotation.set_rotation(
            Axis1::new(p, self.plane.direction()),
            in_plane.rotation_part(),
        )?;
        let dx: Direction3 = self.plane.x_direction().transformed(rotation);
        let dy: Direction3 = self.plane.y_direction().transformed(rotation);
        Ok(Some((dx, dy, major, minor)))
    }

    // The B-spline approximation of the whole curve as the result.
    pub(super) fn load_approximation(
        &mut self,
        curve: Handle<dyn Adaptor3dCurve>,
    ) -> Result<(), Failure> {
        let adaptor: &dyn Adaptor3dCurve = curve.get().ok_or_else(null_curve)?;
        let approximated = perform_approx(adaptor, self.oblique())?;
        // A failed fit leaves a null curve, which GeomAdaptor_Curve refuses.
        let approximated: Handle<dyn GeomCurve> = approximated.cast();
        self.result = Handle::new(GeomAdaptorCurve::with_curve(&approximated)?);
        Ok(())
    }

    // The parameters of the projected end points on the closed canonical curve, the
    // last one moved by periods above the first. GeomLib_Tool::Parameter leaves a
    // bound unchanged when the point is not found, as natively.
    fn trim_periodic_result(&mut self, canonical: &Handle<dyn GeomCurve>) -> Result<(), Failure> {
        let (first, last) = self.curve_bounds()?;
        let oblique = self.oblique();
        let first_point: Point3 = oblique.project_point(self.curve()?.value(first)?);
        let last_point: Point3 = oblique.project_point(self.curve()?.value(last)?);
        let conf: f64 = precision::confusion();
        if let Some(u) = GeomLibTool::parameter(canonical, first_point, conf)? {
            self.first_par = u;
        }
        if let Some(u) = GeomLibTool::parameter(canonical, last_point, conf)? {
            self.last_par = u;
        }
        let period: f64 = self.result()?.period()?;
        while self.last_par <= self.first_par {
            self.last_par += period;
        }
        Ok(())
    }
}

// M = A^-T Delta^2 A^-1 with A = Jp^T Pr(Je) (the projected conic axes in the plane
// frame) and Delta^2 = diag(1 / R1^2, 1 / R2^2).
fn projected_quadratic(
    plane: &rustycad_tkmath::gp::Axis3,
    frame: ConicFrame,
    vdx: Vector3,
    vdy: Vector3,
) -> Result<Matrix<'static>, Failure> {
    let mut a: Matrix<'static> = matrix_2x2(0.0)?;
    a[(1, 1)] = plane.x_direction().xyz().dot(*vdx.xyz());
    a[(1, 2)] = plane.x_direction().xyz().dot(*vdy.xyz());
    a[(2, 1)] = plane.y_direction().xyz().dot(*vdx.xyz());
    a[(2, 2)] = plane.y_direction().xyz().dot(*vdy.xyz());
    let mut delta2: Matrix<'static> = matrix_2x2(0.0)?;
    delta2[(1, 1)] = 1.0 / (frame.r1 * frame.r1);
    delta2[(2, 2)] = 1.0 / (frame.r2 * frame.r2);
    let a_inv: Matrix<'static> = a.inverse()?;
    let product: Matrix<'static> = a_inv.transposed().multiplied_matrix(&delta2)?;
    product.multiplied_matrix(&a_inv)
}

fn matrix_2x2(value: f64) -> Result<Matrix<'static>, Failure> {
    Matrix::filled(1, 2, 1, 2, value).map_err(|error| error.into_failure("ProjLib_ProjectOnPlane"))
}
