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
// Rust translation of ModelingData/TKGeomBase/ProjLib/ProjLib_Sphere.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ProjLib_Sphere`: the exact projection of a meridian (U iso) or of a parallel
//! (V iso) circle onto a sphere, as a 2D line in (U, V).
use super::projector::{Project, Projector, projector_base};
use crate::native_failure::failure;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{
    Axis2d, Axis3, Circle3, Direction2, Direction3, Ellipse3, Hyperbola3, Line2, Line3, Parabola3,
    Point2, Point3, Sphere3, StandardDirection2, Transform2, Vector2, Vector3,
};
use std::f64::consts::PI;

/// `ProjLib_Sphere`; the base-class result is reached through `Deref`.
///
/// ```
/// use rustycad_tkgeombase::proj_lib::Sphere;
/// use rustycad_tkmath::gp::{Axis2, Circle3, Sphere3};
/// let equator = Circle3::new(Axis2::default(), 1.0).unwrap();
/// let projection = Sphere::with_circle(&Sphere3::default(), &equator).unwrap();
/// assert!(projection.is_done());
/// ```
#[derive(Clone, Default)]
pub struct Sphere {
    base: Projector,
    sphere: Sphere3,
}
projector_base!(Sphere);

// The directions a circle and the sphere expose to the iso tests.
struct Frames {
    xc: Direction3,
    yc: Direction3,
    xs: Direction3,
    ys: Direction3,
    zs: Direction3,
}

impl Sphere {
    /// `ProjLib_Sphere()`: an undefined projection; `Sphere::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `ProjLib_Sphere(Sp)`; `Sphere::with_sphere(&sphere)`.
    pub fn with_sphere(sphere: &Sphere3) -> Self {
        let mut projection: Self = Self::new();
        projection.init(sphere);
        projection
    }
    /// `ProjLib_Sphere(Sp, C)`; `Sphere::with_circle(&sphere, &circle)?`.
    pub fn with_circle(sphere: &Sphere3, circle: &Circle3) -> Result<Self, Failure> {
        let mut projection: Self = Self::with_sphere(sphere);
        projection.project_circle(circle)?;
        Ok(projection)
    }
    /// `Init(Sp)`: resets the result to an undone OtherCurve; `projection.init(&sphere)`.
    pub fn init(&mut self, sphere: &Sphere3) {
        self.base.my_type = CurveType::OtherCurve;
        self.sphere = *sphere;
        self.base.is_periodic = false;
        self.base.is_done = false;
    }

    fn frames(&self, circle: &Circle3) -> Frames {
        let position: Axis3 = self.sphere.position();
        Frames {
            xc: circle.position().x_direction(),
            yc: circle.position().y_direction(),
            xs: position.x_direction(),
            ys: position.y_direction(),
            zs: position.direction(),
        }
    }

    // A meridian: both the circle's X and Y points are evaluated, then the U of the
    // first is aligned with the second (or V mirrored when they lie a half turn apart).
    fn u_iso_line(&self, frames: &Frames) -> Result<Line2, Failure> {
        let mut p1: Point2 = eval_pnt2d(Vector3::from(frames.xc), &self.sphere);
        let mut p2: Point2 = eval_pnt2d(Vector3::from(frames.yc), &self.sphere);
        let pconf: f64 = precision::p_confusion();
        if (p1.y() - PI / 2.).abs() < pconf || (p1.y() + PI / 2.).abs() < pconf {
            // P1 is on the apex of the sphere, where U is undefined: take it from P2.
            p1.set_x(p2.x());
        } else if ((p1.x() - p2.x()).abs() - PI).abs() < pconf {
            // U2 = U1 + PI: assume U1 = U2, so V2 = PI - V2.
            p2.set_x(p1.x());
            let mirrored: f64 = if p2.y() < 0. { -PI - p2.y() } else { PI - p2.y() };
            p2.set_y(mirrored);
        } else {
            p2.set_x(p1.x());
        }
        Ok(Line2::new(p1, Direction2::from_vector(Vector2::between(p1, p2))?))
    }

    // A parallel: U is the angle of the circle's X direction, V its latitude.
    fn v_iso_line(&self, circle: &Circle3, frames: &Frames) -> Result<Line2, Failure> {
        let normal_s: Direction3 = frames.xs.crossed(frames.ys)?;
        let mut u: f64 = frames.xs.angle_with_ref(frames.xc, normal_s);
        if u < 0. {
            u += 2. * PI;
        }
        let z: f64 = Vector3::between(self.sphere.location(), circle.location())
            .dot(Vector3::from(frames.zs));
        let v: f64 = (z / self.sphere.radius()).asin();
        let normal_c: Direction3 = frames.xc.crossed(frames.yc)?;
        Ok(Line2::new(Point2::new(u, v), Direction2::new(normal_c.dot(normal_s), 0.)?))
    }
}

// `EvalPnt2d`: the (U, V) of a unit vector from the centre, U in [0, 2 PI[ and the
// sine of V clamped to [-1, 1].
fn eval_pnt2d(point: Vector3, sphere: &Sphere3) -> Point2 {
    let position: Axis3 = sphere.position();
    let x: f64 = point.dot(Vector3::from(position.x_direction()));
    let y: f64 = point.dot(Vector3::from(position.y_direction()));
    let z: f64 = point.dot(Vector3::from(position.direction())).clamp(-1., 1.);
    let pconf: f64 = precision::p_confusion();
    let u: f64 = if x.abs() > pconf || y.abs() > pconf {
        elclib::in_period(y.atan2(x), 0., 2. * PI)
    } else {
        0.
    };
    Point2::new(u, z.asin())
}

impl Project for Sphere {
    fn projector(&self) -> &Projector {
        &self.base
    }
    fn project_line(&mut self, line: &Line3) -> Result<(), Failure> {
        self.base.project_line(line)
    }
    // Neither iso: the type is kept and the line becomes the default gp_Lin2d.
    fn project_circle(&mut self, circle: &Circle3) -> Result<(), Failure> {
        let frames: Frames = self.frames(circle);
        let origin: Point3 = self.sphere.location();
        let tol: f64 = precision::confusion();
        let zc: Direction3 = frames.xc.crossed(frames.yc)?;
        let is_iso_u: bool = zc.is_normal(frames.zs, tol) && origin.is_equal(&circle.location(), tol);
        let is_iso_v: bool = frames.xc.is_normal(frames.zs, tol) && frames.yc.is_normal(frames.zs, tol);
        let mut line: Line2 = Line2::default();
        if is_iso_u {
            self.base.my_type = CurveType::Line;
            line = self.u_iso_line(&frames)?;
            self.base.is_done = true;
        } else if is_iso_v {
            self.base.my_type = CurveType::Line;
            line = self.v_iso_line(circle, &frames)?;
            self.base.is_done = true;
        }
        self.base.lin = line;
        Ok(())
    }
    fn project_ellipse(&mut self, ellipse: &Ellipse3) -> Result<(), Failure> {
        self.base.project_ellipse(ellipse)
    }
    fn project_parabola(&mut self, parabola: &Parabola3) -> Result<(), Failure> {
        self.base.project_parabola(parabola)
    }
    fn project_hyperbola(&mut self, hyperbola: &Hyperbola3) -> Result<(), Failure> {
        self.base.project_hyperbola(hyperbola)
    }
}

impl Sphere {
    /// `SetInBounds(U)`: moves the line so that its point of parameter `u` lies in
    /// the natural bounds of the sphere, mirroring it across a pole when V leaves
    /// `[-PI/2, PI/2]`; StdFail_NotDone before a successful projection;
    /// `projection.set_in_bounds(0.0)?`.
    pub fn set_in_bounds(&mut self, u: f64) -> Result<(), Failure> {
        if !self.base.is_done {
            return Err(failure("StdFail_NotDone", "ProjLib_Sphere:SetInBounds"));
        }
        // First set the Y of the first point in [-PI, PI].
        let y: f64 = elclib::value(u, &self.base.lin).y();
        let new_y: f64 = elclib::in_period(y, -PI, PI);
        self.base.lin.translate(Vector2::new(0., new_y - y));
        let Some(pole) = self.pole_axis(u) else {
            return Ok(());
        };
        self.base.lin.transform(Transform2::mirror_axis(pole));
        self.base.lin.translate(Vector2::new(PI, 0.));
        // Now adjust the U parameter.
        let x: f64 = elclib::value(u, &self.base.lin).x();
        let new_x: f64 = elclib::in_period(x, 0., 2. * PI);
        self.base.lin.translate(Vector2::new(new_x - x, 0.));
        Ok(())
    }

    // The pole line V = ±PI/2 the point of parameter `u` crossed (or reaches heading
    // outwards), if any. OCC13116: strict comparisons with a 1e-7 tolerance.
    fn pole_axis(&self, u: f64) -> Option<Axis2d> {
        let point: Point2 = elclib::value(u, &self.base.lin);
        let tol: f64 = 1.0e-7;
        let d2d: Direction2 = self.base.lin.direction();
        let dy: Direction2 = Direction2::from_standard(StandardDirection2::Y);
        let dx: Direction2 = Direction2::from_standard(StandardDirection2::X);
        if (point.y() - PI / 2. > tol)
            || ((point.y() - PI / 2.).abs() < tol && d2d.is_equal(dy, tol))
        {
            return Some(Axis2d::new(Point2::new(0., PI / 2.), dx));
        }
        if (point.y() + PI / 2. < -tol)
            || ((point.y() + PI / 2.).abs() < tol && d2d.is_opposite(dy, tol))
        {
            return Some(Axis2d::new(Point2::new(0., -PI / 2.), dx));
        }
        None
    }
}
