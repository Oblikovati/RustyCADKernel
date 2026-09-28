// Copyright (c) 1991-1999 Matra Datavision
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
// Rust translation of gce_MakeCylinder.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! gce_MakeCylinder: construction of gp_Cylinder.
use super::{ErrorType, gce_result};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{
    self, Axis1, Axis2, Axis3, Circle3, Cylinder3, Direction3, Line3, Point3,
};

/// gce_MakeCylinder: builds a cylinder from a frame, an axis, three points, a
/// circle, or a parallel cylinder.
///
/// `let cylinder = MakeCylinder::new(Axis2::default(), 1.0)?.value()?;`
#[derive(Clone, Copy, Debug, Default)]
pub struct MakeCylinder {
    the_result: Cylinder3,
    the_error: ErrorType,
}
gce_result!(MakeCylinder, Cylinder3, "gce_MakeCylinder");

impl MakeCylinder {
    fn failed(error: ErrorType) -> Self {
        Self {
            the_error: error,
            ..Self::default()
        }
    }
    fn built(cylinder: Cylinder3) -> Self {
        Self {
            the_result: cylinder,
            the_error: ErrorType::Done,
        }
    }

    /// gce_MakeCylinder(A2, Radius): the cylinder of frame `frame`;
    /// gce_NegativeRadius when `radius < 0`. `MakeCylinder::new(frame, 1.0)?`.
    pub fn new(frame: Axis2, radius: f64) -> Result<Self, Failure> {
        if radius < 0.0 {
            return Ok(Self::failed(ErrorType::NegativeRadius));
        }
        Ok(Self::built(Cylinder3::new(
            Axis3::from_axis2(frame),
            radius,
        )?))
    }

    /// gce_MakeCylinder(Axis, Radius): the cylinder of axis `axis`, its X
    /// direction from `perpendicular_hint`; gce_NegativeRadius when `radius < 0`.
    /// `MakeCylinder::from_axis(Axis1::default(), 1.0)?`.
    pub fn from_axis(axis: Axis1, radius: f64) -> Result<Self, Failure> {
        if radius < 0.0 {
            return Ok(Self::failed(ErrorType::NegativeRadius));
        }
        let direction: Direction3 = axis.direction();
        let frame =
            Axis2::with_x_direction(axis.location(), direction, perpendicular_hint(direction)?)?;
        Ok(Self::built(Cylinder3::new(
            Axis3::from_axis2(frame),
            radius,
        )?))
    }

    /// gce_MakeCylinder(Circ): the cylinder of `circle`'s frame and radius.
    /// `MakeCylinder::from_circ(circle)?`.
    pub fn from_circ(circle: Circle3) -> Result<Self, Failure> {
        Ok(Self::built(Cylinder3::new(
            Axis3::from_axis2(circle.position()),
            circle.radius(),
        )?))
    }

    /// gce_MakeCylinder(P1, P2, P3): the cylinder of axis P1P2 through P3;
    /// gce_ConfusedPoints when P1 and P2 coincide. `MakeCylinder::through_points(a, b, c)?`.
    pub fn through_points(p1: Point3, p2: Point3, p3: Point3) -> Result<Self, Failure> {
        if p1.distance(&p2) < gp::resolution() {
            return Ok(Self::failed(ErrorType::ConfusedPoints));
        }
        let d1: Direction3 = Direction3::from_xyz(p2.xyz() - p1.xyz())?;
        let frame = Axis2::with_x_direction(p1, d1, perpendicular_hint(d1)?)?;
        let radius: f64 = Line3::new(p1, d1).distance(p3);
        Ok(Self::built(Cylinder3::new(
            Axis3::from_axis2(frame),
            radius,
        )?))
    }

    /// gce_MakeCylinder(Cyl, Dist): `cylinder` with radius `Radius + dist`;
    /// gce_NegativeRadius when that radius is negative.
    /// `MakeCylinder::from_cylinder_distance(cylinder, 1.0)?`.
    pub fn from_cylinder_distance(cylinder: Cylinder3, dist: f64) -> Result<Self, Failure> {
        let radius: f64 = cylinder.radius() + dist;
        if radius < 0.0 {
            return Ok(Self::failed(ErrorType::NegativeRadius));
        }
        let mut result: Cylinder3 = cylinder;
        result.set_radius(radius)?;
        Ok(Self::built(result))
    }

    /// gce_MakeCylinder(Cyl, P): `cylinder` with the distance of `point` from its
    /// axis as radius. `MakeCylinder::from_cylinder_point(cylinder, p)?`.
    pub fn from_cylinder_point(cylinder: Cylinder3, point: Point3) -> Result<Self, Failure> {
        let radius: f64 = Line3::from_axis(cylinder.axis()).distance(point);
        let mut result: Cylinder3 = cylinder;
        result.set_radius(radius)?;
        Ok(Self::built(result))
    }
}

/// The X direction that gce_MakeCylinder and gce_MakeCone choose for a main
/// direction D: `(-y, x, 0)` when |x| exceeds gp::Resolution(), else
/// `(0, -z, y)`, else `(z, 0, -x)`, else the default gp_Dir() (Z).
pub(super) fn perpendicular_hint(direction: Direction3) -> Result<Direction3, Failure> {
    let (x, y, z) = (direction.x(), direction.y(), direction.z());
    let resolution: f64 = gp::resolution();
    if x.abs() > resolution {
        Direction3::new(-y, x, 0.0)
    } else if y.abs() > resolution {
        Direction3::new(0.0, -z, y)
    } else if z.abs() > resolution {
        Direction3::new(z, 0.0, -x)
    } else {
        Ok(Direction3::default())
    }
}
