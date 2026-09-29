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
// Rust translation of ModelingData/TKGeomBase/ProjLib/ProjLib_Cylinder.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ProjLib_Cylinder`: the exact projection of a generatrix line or of a circle
//! normal to the axis onto a cylinder, as a 2D line in (U, V).
use super::projector::{Project, Projector, projector_base};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{
    Axis2, Axis3, Circle3, Cylinder3, Direction2, Direction3, Ellipse3, Hyperbola3, Line2, Line3,
    Parabola3, Point2, Point3, Vector3,
};
use std::f64::consts::PI;

/// `ProjLib_Cylinder`; the base-class result is reached through `Deref`.
///
/// ```
/// use rustycad_tkgeombase::proj_lib::Cylinder;
/// use rustycad_tkmath::gp::{Cylinder3, Direction3, Line3, Point3};
/// let line = Line3::new(Point3::new(1.0, 0.0, 0.0), Direction3::new(0.0, 0.0, 1.0).unwrap());
/// let projection = Cylinder::with_line(&Cylinder3::default(), &line);
/// assert!(projection.is_done());
/// ```
#[derive(Clone, Default)]
pub struct Cylinder {
    base: Projector,
    cylinder: Cylinder3,
}
projector_base!(Cylinder);

impl Cylinder {
    /// `ProjLib_Cylinder()`: an undefined projection; `Cylinder::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `ProjLib_Cylinder(Cyl)`; `Cylinder::with_cylinder(&cylinder)`.
    pub fn with_cylinder(cylinder: &Cylinder3) -> Self {
        let mut projection: Self = Self::new();
        projection.init(cylinder);
        projection
    }
    /// `ProjLib_Cylinder(Cyl, L)`: never fails; `Cylinder::with_line(&cylinder, &line)`.
    pub fn with_line(cylinder: &Cylinder3, line: &Line3) -> Self {
        let mut projection: Self = Self::with_cylinder(cylinder);
        projection.line_projection(line);
        projection
    }
    /// `ProjLib_Cylinder(Cyl, C)`; `Cylinder::with_circle(&cylinder, &circle)?`.
    pub fn with_circle(cylinder: &Cylinder3, circle: &Circle3) -> Result<Self, Failure> {
        let mut projection: Self = Self::with_cylinder(cylinder);
        projection.circle_projection(circle)?;
        Ok(projection)
    }
    /// `ProjLib_Cylinder(Cyl, E)`: the ellipse is left to the approximation, so the
    /// result is not done; `Cylinder::with_ellipse(&cylinder, &ellipse)`.
    pub fn with_ellipse(cylinder: &Cylinder3, _ellipse: &Ellipse3) -> Self {
        Self::with_cylinder(cylinder)
    }
    /// `Init(Cyl)`: resets the result to an undone OtherCurve; `projection.init(&cylinder)`.
    pub fn init(&mut self, cylinder: &Cylinder3) {
        self.base.my_type = CurveType::OtherCurve;
        self.cylinder = *cylinder;
        self.base.is_periodic = false;
        self.base.is_done = false;
    }

    // `Project(gp_Lin)`: only a line parallel to the axis projects exactly.
    fn line_projection(&mut self, line: &Line3) {
        let axis: Direction3 = self.cylinder.position().direction();
        let angular: f64 = precision::angular();
        if line.direction().xyz().cross_square_magnitude(axis.xyz()) > angular * angular {
            return;
        }
        self.base.my_type = CurveType::Line;
        let mut p2d: Point2 = eval_pnt2d(line.location(), &self.cylinder);
        if p2d.x() < 0. {
            p2d.set_x(p2d.x() + 2. * PI);
        }
        let signe: f64 = if line.direction().dot(axis) > 0. { 1. } else { -1. };
        self.base.lin = Line2::new(p2d, unit_direction(0., signe));
        self.base.is_done = true;
    }

    // `Project(gp_Circ)`: only a circle whose normal is the axis projects exactly.
    fn circle_projection(&mut self, circle: &Circle3) -> Result<(), Failure> {
        let cyl_pos: Axis3 = self.cylinder.position();
        let circ_pos: Axis2 = circle.position();
        let angular: f64 = precision::angular();
        let cross: f64 = cyl_pos.direction().xyz().cross_square_magnitude(circ_pos.direction().xyz());
        if cross > angular * angular {
            return Ok(());
        }
        self.base.my_type = CurveType::Line;
        let z_cyl: Direction3 = frame_normal(&cyl_pos)?;
        let u: f64 = cyl_pos.x_direction().angle_with_ref(circ_pos.x_direction(), z_cyl);
        let op: Vector3 = Vector3::between(self.cylinder.location(), circle.location());
        let v: f64 = op.dot(Vector3::from(cyl_pos.direction()));
        let sense: f64 = if z_cyl.dot(circ_pos.direction()) > 0. { 1. } else { -1. };
        self.base.lin = Line2::new(Point2::new(u, v), unit_direction(sense, 0.));
        self.base.is_done = true;
        Ok(())
    }
}

// `XDirection().Crossed(YDirection())`: the normal of the frame's XY plane, which
// is the main direction of a right-handed frame and its opposite otherwise.
pub(super) fn frame_normal(position: &Axis3) -> Result<Direction3, Failure> {
    position.x_direction().crossed(position.y_direction())
}

// A unit axis direction (`gp_Dir2d(±1, 0)` or `(0, ±1)`), which cannot be null.
pub(super) fn unit_direction(x: f64, y: f64) -> Direction2 {
    Direction2::new(x, y).unwrap_or_default()
}

// `EvalPnt2d`: the cylinder (U, V) of a point, with U = 0 on the axis.
fn eval_pnt2d(point: Point3, cylinder: &Cylinder3) -> Point2 {
    let op: Vector3 = Vector3::between(cylinder.location(), point);
    let position: Axis3 = cylinder.position();
    let x: f64 = op.dot(Vector3::from(position.x_direction()));
    let y: f64 = op.dot(Vector3::from(position.y_direction()));
    let z: f64 = op.dot(Vector3::from(position.direction()));
    let pconf: f64 = precision::p_confusion();
    let u: f64 = if x.abs() > pconf || y.abs() > pconf { y.atan2(x) } else { 0. };
    Point2::new(u, z)
}

impl Project for Cylinder {
    fn projector(&self) -> &Projector {
        &self.base
    }
    fn project_line(&mut self, line: &Line3) -> Result<(), Failure> {
        self.line_projection(line);
        Ok(())
    }
    fn project_circle(&mut self, circle: &Circle3) -> Result<(), Failure> {
        self.circle_projection(circle)
    }
    // Due to widespread issues with poorly handled periodicity, the native class
    // delegates the ellipse to the approximation: the result is left untouched.
    fn project_ellipse(&mut self, _ellipse: &Ellipse3) -> Result<(), Failure> {
        Ok(())
    }
    fn project_parabola(&mut self, parabola: &Parabola3) -> Result<(), Failure> {
        self.base.project_parabola(parabola)
    }
    fn project_hyperbola(&mut self, hyperbola: &Hyperbola3) -> Result<(), Failure> {
        self.base.project_hyperbola(hyperbola)
    }
}
