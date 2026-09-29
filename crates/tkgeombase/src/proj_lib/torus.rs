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
// Rust translation of ModelingData/TKGeomBase/ProjLib/ProjLib_Torus.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ProjLib_Torus`: the projection of an iso circle onto a torus, as a 2D line in
//! (U, V). The class does not check that the circle lies on the torus: a circle
//! whose normal is the axis (or centred at the torus centre) is taken as a V iso,
//! any other circle as a U iso.
use super::projector::{Project, Projector, projector_base};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{
    Axis3, Circle3, Direction2, Ellipse3, Hyperbola3, Line2, Line3, Parabola3, Point2,
    StandardDirection2, Torus3, Vector2, Vector3,
};
use std::f64::consts::PI;

/// `ProjLib_Torus`; the base-class result is reached through `Deref`.
///
/// ```
/// use rustycad_tkgeombase::proj_lib::Torus;
/// use rustycad_tkmath::gp::{Axis2, Axis3, Circle3, Torus3};
/// let torus = Torus3::new(Axis3::default(), 3.0, 1.0).unwrap();
/// let circle = Circle3::new(Axis2::default(), 4.0).unwrap();
/// assert!(Torus::with_circle(&torus, &circle).unwrap().is_done());
/// ```
#[derive(Clone, Default)]
pub struct Torus {
    base: Projector,
    torus: Torus3,
}
projector_base!(Torus);

impl Torus {
    /// `ProjLib_Torus()`: an undefined projection; `Torus::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `ProjLib_Torus(To)`; `Torus::with_torus(&torus)`.
    pub fn with_torus(torus: &Torus3) -> Self {
        let mut projection: Self = Self::new();
        projection.init(torus);
        projection
    }
    /// `ProjLib_Torus(To, C)`; `Torus::with_circle(&torus, &circle)?`.
    pub fn with_circle(torus: &Torus3, circle: &Circle3) -> Result<Self, Failure> {
        let mut projection: Self = Self::with_torus(torus);
        projection.project_circle(circle)?;
        Ok(projection)
    }
    /// `Init(To)`: resets the result to an undone OtherCurve; `projection.init(&torus)`.
    pub fn init(&mut self, torus: &Torus3) {
        self.base.my_type = CurveType::OtherCurve;
        self.torus = *torus;
        self.base.is_periodic = false;
        self.base.is_done = false;
    }

    // A V iso: U of the circle's X and Y points, V from the height of the centre.
    fn v_iso_line(&self, circle: &Circle3, oc: Vector3) -> Result<Line2, Failure> {
        let xc: Vector3 = Vector3::from(circle.position().x_direction());
        let yc: Vector3 = Vector3::from(circle.position().y_direction());
        let mut p1: Point2 = eval_pnt2d(xc, &self.torus);
        let mut p2: Point2 = eval_pnt2d(yc, &self.torus);
        let mut z: f64 = oc.dot(Vector3::from(self.torus.position().direction()));
        z /= self.torus.minor_radius();
        // A simple protection against Z slightly outside [-1, 1].
        let mut v: f64 = if z > 1. {
            PI / 2.
        } else if z < -1. {
            -PI / 2.
        } else {
            z.asin()
        };
        if circle.radius() < self.torus.major_radius() {
            v = PI - v;
        } else if v < 0. {
            v += 2. * PI;
        }
        p1.set_y(v);
        p2.set_y(v);
        let mut v2d: Vector2 = Vector2::between(p1, p2);
        // Normally |P1.X - P2.X| = PI/2; otherwise the period was crossed.
        if (p1.x() - p2.x()).abs() > PI {
            v2d.reverse();
        }
        let d2: Direction2 = Direction2::from_vector(v2d)?;
        if p1.x() < 0. {
            p1.set_x(2. * PI + p1.x());
        }
        Ok(Line2::new(p1, d2))
    }

    // A U iso: U is the angle of OC, V the angle of the circle's X direction; the
    // gp_Vec angles raise gp_VectorWithNullMagnitude when OC is along the axis.
    fn u_iso_line(&self, circle: &Circle3, oc: Vector3) -> Result<Line2, Failure> {
        let position: Axis3 = self.torus.position();
        let xt: Vector3 = Vector3::from(position.x_direction());
        let yt: Vector3 = Vector3::from(position.y_direction());
        let zt: Vector3 = Vector3::from(position.direction());
        let xc: Vector3 = Vector3::from(circle.position().x_direction());
        let yc: Vector3 = Vector3::from(circle.position().y_direction());
        let mut u: f64 = xt.angle_with_ref(oc, xt.crossed(yt))?;
        if u < 0. {
            u += 2. * PI;
        }
        let mut v1: f64 = oc.angle_with_ref(xc, oc.crossed(zt))?;
        if v1 < 0. {
            v1 += 2. * PI;
        }
        let mut d2: Direction2 = Direction2::from_standard(StandardDirection2::Y);
        if oc.crossed(zt).dot(xc.crossed(yc)) < 0. {
            d2.reverse();
        }
        Ok(Line2::new(Point2::new(u, v1), d2))
    }
}

// `EvalPnt2d`: the U of a direction in the torus frame (0 on the axis); V is 0.
fn eval_pnt2d(direction: Vector3, torus: &Torus3) -> Point2 {
    let x: f64 = direction.dot(Vector3::from(torus.position().x_direction()));
    let y: f64 = direction.dot(Vector3::from(torus.position().y_direction()));
    let pconf: f64 = precision::p_confusion();
    let u: f64 = if x.abs() > pconf || y.abs() > pconf { y.atan2(x) } else { 0. };
    Point2::new(u, 0.)
}

impl Project for Torus {
    fn projector(&self) -> &Projector {
        &self.base
    }
    fn project_line(&mut self, line: &Line3) -> Result<(), Failure> {
        self.base.project_line(line)
    }
    fn project_circle(&mut self, circle: &Circle3) -> Result<(), Failure> {
        self.base.my_type = CurveType::Line;
        let oc: Vector3 = Vector3::between(self.torus.location(), circle.location());
        let axis_parallel: bool = circle
            .position()
            .direction()
            .is_parallel(self.torus.position().direction(), precision::angular());
        self.base.lin = if oc.magnitude() < precision::confusion() || axis_parallel {
            self.v_iso_line(circle, oc)?
        } else {
            self.u_iso_line(circle, oc)?
        };
        self.base.is_done = true;
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
