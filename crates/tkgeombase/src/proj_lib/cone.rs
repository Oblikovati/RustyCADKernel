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
// Rust translation of ModelingData/TKGeomBase/ProjLib/ProjLib_Cone.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ProjLib_Cone`: the exact projection of a generatrix line or of a circle normal
//! to the axis onto a cone, as a 2D line in (U, V).
use super::cylinder::{frame_normal, unit_direction};
use super::projector::{Project, Projector, projector_base};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::elslib;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{
    Axis3, Circle3, Cone3, Direction3, Ellipse3, Hyperbola3, Line2, Line3, Parabola3, Point2,
    Point3, Vector3,
};
use std::f64::consts::PI;

/// `ProjLib_Cone`; the base-class result is reached through `Deref`.
///
/// ```
/// use rustycad_tkgeombase::proj_lib::Cone;
/// use rustycad_tkmath::gp::{Axis2, Circle3, Cone3, Direction3, Point3, Axis3};
/// let cone = Cone3::new(Axis3::default(), 0.5, 5.0).unwrap();
/// let axis = Axis2::new(Point3::new(0.0, 0.0, 10.0), Direction3::new(0.0, 0.0, 1.0).unwrap()).unwrap();
/// let projection = Cone::with_circle(&cone, &Circle3::new(axis, 2.0).unwrap()).unwrap();
/// assert!(projection.is_done());
/// ```
#[derive(Clone, Default)]
pub struct Cone {
    base: Projector,
    cone: Cone3,
}
projector_base!(Cone);

impl Cone {
    /// `ProjLib_Cone()`: an undefined projection; `Cone::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `ProjLib_Cone(Co)`; `Cone::with_cone(&cone)`.
    pub fn with_cone(cone: &Cone3) -> Self {
        let mut projection: Self = Self::new();
        projection.init(cone);
        projection
    }
    /// `ProjLib_Cone(Co, L)`; `Cone::with_line(&cone, &line)?`.
    pub fn with_line(cone: &Cone3, line: &Line3) -> Result<Self, Failure> {
        let mut projection: Self = Self::with_cone(cone);
        projection.project_line(line)?;
        Ok(projection)
    }
    /// `ProjLib_Cone(Co, C)`; `Cone::with_circle(&cone, &circle)?`.
    pub fn with_circle(cone: &Cone3, circle: &Circle3) -> Result<Self, Failure> {
        let mut projection: Self = Self::with_cone(cone);
        projection.project_circle(circle)?;
        Ok(projection)
    }
    /// `Init(Co)`: resets the result to an undone OtherCurve; `projection.init(&cone)`.
    pub fn init(&mut self, cone: &Cone3) {
        self.base.my_type = CurveType::OtherCurve;
        self.cone = *cone;
        self.base.is_periodic = false;
        self.base.is_done = false;
    }

    // The (U, V) of a circle centre: the ElSLib cone formula applied without the
    // frame change; x and y are direction cosines, hence the angular tolerance.
    fn circle_origin(&self, circle: &Circle3) -> Point2 {
        let cone_pos: Axis3 = self.cone.position();
        let circ_x: Direction3 = circle.position().x_direction();
        let x: f64 = cone_pos.x_direction().dot(circ_x);
        let y: f64 = cone_pos.y_direction().dot(circ_x);
        let axis: Vector3 = Vector3::from(cone_pos.direction());
        let z: f64 = Vector3::between(self.cone.location(), circle.location()).dot(axis);
        let angular: f64 = precision::angular();
        let mut u: f64 = if x.abs() <= angular && y.abs() <= angular {
            0.
        } else if -self.cone.ref_radius() > z * self.cone.semi_angle().tan() {
            (-y).atan2(-x)
        } else {
            y.atan2(x)
        };
        if u < 0. {
            u += 2. * PI;
        }
        Point2::new(u, z / self.cone.semi_angle().cos())
    }
}

impl Project for Cone {
    fn projector(&self) -> &Projector {
        &self.base
    }
    fn project_line(&mut self, line: &Line3) -> Result<(), Failure> {
        let (mut point, apex): (Point3, Point3) = (line.location(), self.cone.apex());
        let mut delta_v: f64 = 0.0;
        if point.is_equal(&apex, precision::confusion()) {
            // Take another point of L, which does not coincide with the apex.
            point.translate(Vector3::from(line.direction()));
            delta_v = 1.0; // == ||L.Direction()|| == 1.0
        }
        let (position, radius, angle) = (
            self.cone.position(),
            self.cone.ref_radius(),
            self.cone.semi_angle(),
        );
        let (u, v) = elslib::cone_parameters(position, radius, angle, point);
        let (_, _, vv) = elslib::cone_d1(u, v, position, radius, angle);
        let dv: Direction3 = Direction3::from_vector(vv)?;
        if !dv.is_parallel(line.direction(), precision::angular()) {
            return Ok(());
        }
        // L is parallel to the U-isoline of the cone.
        self.base.my_type = CurveType::Line;
        let sign: f64 = 1.0_f64.copysign(line.direction().dot(dv));
        let p2d: Point2 = Point2::new(u, v - delta_v * sign);
        self.base.lin = Line2::new(p2d, unit_direction(0., sign));
        self.base.is_done = true;
        Ok(())
    }
    // The type is set to Line before the axis test, so a rejected circle leaves a
    // Line type over the previous line (docs/tkgeombase-progress.md).
    fn project_circle(&mut self, circle: &Circle3) -> Result<(), Failure> {
        self.base.my_type = CurveType::Line;
        let cone_pos: Axis3 = self.cone.position();
        let circ_pos: Axis3 = Axis3::from_axis2(circle.position());
        if !cone_pos
            .direction()
            .is_parallel(circ_pos.direction(), precision::angular())
        {
            self.base.is_done = false;
            return Ok(());
        }
        let z_cone: Direction3 = frame_normal(&cone_pos)?;
        let z_cir: Direction3 = frame_normal(&circ_pos)?;
        let p2d: Point2 = self.circle_origin(circle);
        let sense: f64 = if z_cone.dot(z_cir) > 0. { 1. } else { -1. };
        self.base.lin = Line2::new(p2d, unit_direction(sense, 0.));
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
