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
// Rust translation of GeomBndLib_Circle and GeomBndLib_Circle2d at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomBndLib_Circle` and `GeomBndLib_Circle2d`: analytic boxes of circles and arcs.
use super::elliptic_arc::{ArcTangent, EllipticArc, adjusted_arc, is_full_period};
use super::live;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCircle;
use rustycad_tkg3d::geom::GeomCircle;
use rustycad_tkmath::bnd::{BoundingBox2, BoundingBox3};
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Circle2, Circle3, Point2, Xy};

/// `GeomBndLib_Circle`: the box of a `Geom_Circle`, full or on an arc; the static
/// members take a `gp_Circ` directly.
pub struct Circle {
    geom: Handle<GeomCircle>,
}

// The ElCLib evaluator of a 3D circle and its extremum analysis.
fn circle_arc<'a>(
    circle: &Circle3,
    eval: &'a dyn Fn(f64) -> rustycad_tkmath::gp::Point3,
) -> EllipticArc<'a, rustycad_tkmath::gp::Point3> {
    let position = circle.position();
    EllipticArc {
        major: circle.radius(),
        minor: circle.radius(),
        origin: *position.location().coordinates(),
        x_dir: *position.x_direction().coordinates(),
        y_dir: *position.y_direction().coordinates(),
        tangent: ArcTangent::Circle,
        eval,
    }
}

impl Circle {
    /// The evaluator of `circle`; `Circle::new(handle)`.
    pub fn new(circle: Handle<GeomCircle>) -> Self {
        Self { geom: circle }
    }
    /// `Geometry()`; `evaluator.geometry()`.
    pub fn geometry(&self) -> &Handle<GeomCircle> {
        &self.geom
    }
    fn circ(&self) -> Result<Circle3, Failure> {
        live(&self.geom, "GeomBndLib_Circle")?.circ()
    }
    /// `Box(Tol)`: the whole circle; `evaluator.bounding_box(0.0)?`.
    pub fn bounding_box(&self, tolerance: f64) -> Result<BoundingBox3, Failure> {
        Ok(Self::circ_box(&self.circ()?, tolerance))
    }
    /// `Box(U1, U2, Tol)`: the arc; `evaluator.bounding_box_range(0.0, 1.0, 0.0)?`.
    pub fn bounding_box_range(
        &self,
        u1: f64,
        u2: f64,
        tolerance: f64,
    ) -> Result<BoundingBox3, Failure> {
        Ok(Self::circ_arc_box(&self.circ()?, u1, u2, tolerance))
    }
    /// `BoxOptimal(U1, U2, Tol)`: the analytic box is already exact;
    /// `evaluator.bounding_box_optimal_range(0.0, 1.0, 0.0)?`.
    pub fn bounding_box_optimal_range(
        &self,
        u1: f64,
        u2: f64,
        tolerance: f64,
    ) -> Result<BoundingBox3, Failure> {
        self.bounding_box_range(u1, u2, tolerance)
    }
    /// `Box(gp_Circ, Tol)`: per-coordinate extent `O +/- R sqrt(Xk^2 + Yk^2)`;
    /// `Circle::circ_box(&circle, 0.0)`.
    pub fn circ_box(circle: &Circle3, tolerance: f64) -> BoundingBox3 {
        let eval = |u: f64| elclib::circle_value(u, circle.position(), circle.radius());
        circle_arc(circle, &eval).full_box(tolerance)
    }
    /// `Box(gp_Circ, U1, U2, Tol)`: a span of at least a period is the whole circle;
    /// otherwise the arc brought into `[0, 2Pi)` with its ends and the coordinate
    /// extrema inside it; `Circle::circ_arc_box(&circle, 0.0, 1.0, 0.0)`.
    pub fn circ_arc_box(circle: &Circle3, u1: f64, u2: f64, tolerance: f64) -> BoundingBox3 {
        if is_full_period(u1, u2) {
            return Self::circ_box(circle, tolerance);
        }
        let (a1, a2) = adjusted_arc(u1, u2);
        let eval = |u: f64| elclib::circle_value(u, circle.position(), circle.radius());
        let mut bounds: BoundingBox3 = BoundingBox3::default();
        circle_arc(circle, &eval).add_arc(&mut bounds, a1, a2);
        bounds.enlarge(tolerance);
        bounds
    }
}

/// `GeomBndLib_Circle2d`: the box of a `Geom2d_Circle`; the static members take a
/// `gp_Circ2d`.
pub struct Circle2d {
    geom: Handle<Geom2dCircle>,
}

// The native lambda `gp_Pnt2d(aO + aR cos(t) aXd + aR sin(t) aYd)` and the analysis.
fn circle2d_eval(circle: &Circle2, u: f64) -> Point2 {
    let frame = circle.position();
    let origin: Xy = frame.location().xy();
    let x_dir: Xy = frame.x_direction().xy();
    let y_dir: Xy = frame.y_direction().xy();
    let radius: f64 = circle.radius();
    Point2::from(origin + x_dir * (radius * u.cos()) + y_dir * (radius * u.sin()))
}

fn circle2d_arc<'a>(circle: &Circle2, eval: &'a dyn Fn(f64) -> Point2) -> EllipticArc<'a, Point2> {
    let frame = circle.position();
    let (location, x, y) = (frame.location(), frame.x_direction(), frame.y_direction());
    EllipticArc {
        major: circle.radius(),
        minor: circle.radius(),
        origin: [location.x(), location.y(), 0.0],
        x_dir: [x.x(), x.y(), 0.0],
        y_dir: [y.x(), y.y(), 0.0],
        tangent: ArcTangent::Circle,
        eval,
    }
}

impl Circle2d {
    /// The evaluator of `circle`; `Circle2d::new(handle)`.
    pub fn new(circle: Handle<Geom2dCircle>) -> Self {
        Self { geom: circle }
    }
    /// `Geometry()`; `evaluator.geometry()`.
    pub fn geometry(&self) -> &Handle<Geom2dCircle> {
        &self.geom
    }
    fn circ2d(&self) -> Result<Circle2, Failure> {
        live(&self.geom, "GeomBndLib_Circle2d")?.circ2d()
    }
    /// `Box(Tol)`: the whole circle; `evaluator.bounding_box(0.0)?`.
    pub fn bounding_box(&self, tolerance: f64) -> Result<BoundingBox2, Failure> {
        Ok(Self::circ_box(&self.circ2d()?, tolerance))
    }
    /// `Box(U1, U2, Tol)`: the arc; `evaluator.bounding_box_range(0.0, 1.0, 0.0)?`.
    pub fn bounding_box_range(
        &self,
        u1: f64,
        u2: f64,
        tolerance: f64,
    ) -> Result<BoundingBox2, Failure> {
        Ok(Self::circ_arc_box(&self.circ2d()?, u1, u2, tolerance))
    }
    /// `BoxOptimal(U1, U2, Tol)`: the analytic box;
    /// `evaluator.bounding_box_optimal_range(0.0, 1.0, 0.0)?`.
    pub fn bounding_box_optimal_range(
        &self,
        u1: f64,
        u2: f64,
        tolerance: f64,
    ) -> Result<BoundingBox2, Failure> {
        self.bounding_box_range(u1, u2, tolerance)
    }
    /// `Box(gp_Circ2d, Tol)`: per-coordinate extent of the whole circle;
    /// `Circle2d::circ_box(&circle, 0.0)`.
    pub fn circ_box(circle: &Circle2, tolerance: f64) -> BoundingBox2 {
        let eval = |u: f64| circle2d_eval(circle, u);
        circle2d_arc(circle, &eval).full_box(tolerance)
    }
    /// `Box(gp_Circ2d, U1, U2, Tol)`: unlike the 3D class, the arc parameters are used
    /// as given (no `AdjustPeriodic`); `Circle2d::circ_arc_box(&circle, 0.0, 1.0, 0.0)`.
    pub fn circ_arc_box(circle: &Circle2, u1: f64, u2: f64, tolerance: f64) -> BoundingBox2 {
        if is_full_period(u1, u2) {
            return Self::circ_box(circle, tolerance);
        }
        let eval = |u: f64| circle2d_eval(circle, u);
        let mut bounds: BoundingBox2 = BoundingBox2::default();
        circle2d_arc(circle, &eval).add_arc(&mut bounds, u1, u2);
        bounds.enlarge(tolerance);
        bounds
    }
}
