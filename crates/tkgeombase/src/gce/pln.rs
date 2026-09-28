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
// Rust translation of gce_MakePln.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! gce_MakePln: construction of gp_Pln.
use super::{ErrorType, gce_result};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{self, Axis1, Axis2, Axis3, Direction3, Plane3, Point3, Xyz};

/// gce_MakePln: builds a plane from a frame, a point and a normal, an equation,
/// two or three points, an axis, or a parallel plane.
///
/// `let plane = MakePln::from_frame(Axis2::default()).value()?;`
#[derive(Clone, Copy, Debug, Default)]
pub struct MakePln {
    the_result: Plane3,
    the_error: ErrorType,
}
gce_result!(MakePln, Plane3, "gce_MakePln");

impl MakePln {
    fn failed(error: ErrorType) -> Self {
        Self {
            the_error: error,
            ..Self::default()
        }
    }
    fn built(plane: Plane3) -> Self {
        Self {
            the_result: plane,
            the_error: ErrorType::Done,
        }
    }

    /// gce_MakePln(A2): the plane of frame `frame` (`gp_Pln(gp_Ax3(A2))`).
    /// `MakePln::from_frame(frame)`.
    pub fn from_frame(frame: Axis2) -> Self {
        Self::built(Plane3::from_position(Axis3::from_axis2(frame)))
    }

    /// gce_MakePln(P, V): the plane through `point` of normal `normal`.
    /// `MakePln::new(p, n)?`.
    pub fn new(point: Point3, normal: Direction3) -> Result<Self, Failure> {
        Ok(Self::built(Plane3::new(point, normal)?))
    }

    /// gce_MakePln(A, B, C, D): the plane `a x + b y + c z + d = 0`;
    /// gce_BadEquation when `a² + b² + c²` is at most gp::Resolution().
    /// `MakePln::from_equation(0.0, 0.0, 1.0, 0.0)?`.
    pub fn from_equation(a: f64, b: f64, c: f64, d: f64) -> Result<Self, Failure> {
        if a * a + b * b + c * c <= gp::resolution() {
            return Ok(Self::failed(ErrorType::BadEquation));
        }
        Ok(Self::built(Plane3::from_equation(a, b, c, d)?))
    }

    /// gce_MakePln(P1, P2, P3): the plane through three points, origin `p1`, X
    /// direction P1P2; gce_ColinearPoints when the normal is shorter than
    /// gp::Resolution(). `MakePln::through_points(a, b, c)?`.
    pub fn through_points(p1: Point3, p2: Point3, p3: Point3) -> Result<Self, Failure> {
        let v1: Xyz = p2.xyz() - p1.xyz();
        let v2: Xyz = p3.xyz() - p1.xyz();
        let normal: Xyz = v1.crossed(v2);
        if normal.modulus() < gp::resolution() {
            return Ok(Self::failed(ErrorType::ColinearPoints));
        }
        let (d_normal, dx) = (Direction3::from_xyz(normal)?, Direction3::from_xyz(v1)?);
        Ok(Self::built(Plane3::from_position(Axis3::new(
            p1, d_normal, dx,
        )?)))
    }

    /// gce_MakePln(P1, P2): the plane through `p1` normal to P1P2;
    /// gce_ConfusedPoints when the points are within gp::Resolution().
    /// `MakePln::from_points(a, b)?`.
    pub fn from_points(p1: Point3, p2: Point3) -> Result<Self, Failure> {
        if p1.distance(&p2) <= gp::resolution() {
            return Ok(Self::failed(ErrorType::ConfusedPoints));
        }
        let normal: Direction3 = Direction3::from_xyz(p2.xyz() - p1.xyz())?;
        Ok(Self::built(Plane3::new(p1, normal)?))
    }

    /// gce_MakePln(Pl, Dist): `plane` translated by `dist` along its normal,
    /// keeping its X direction. `MakePln::from_plane_distance(plane, 1.0)?`.
    pub fn from_plane_distance(plane: Plane3, dist: f64) -> Result<Self, Failure> {
        let normal: Direction3 = plane.axis().direction();
        let center: Point3 = Point3::from(plane.location().xyz() + normal.xyz() * dist);
        Self::parallel_through(plane, center)
    }

    /// gce_MakePln(Pl, Point): `plane` moved to pass through `point`.
    /// `MakePln::from_plane_point(plane, p)?`.
    pub fn from_plane_point(plane: Plane3, point: Point3) -> Result<Self, Failure> {
        Self::parallel_through(plane, point)
    }

    /// `gp_Pln(gp_Ax3(Point, Pl.Axis().Direction(), Pl.XAxis().Direction()))`.
    fn parallel_through(plane: Plane3, point: Point3) -> Result<Self, Failure> {
        let frame = Axis3::new(point, plane.axis().direction(), plane.x_axis().direction())?;
        Ok(Self::built(Plane3::from_position(frame)))
    }

    /// gce_MakePln(Axis): the plane through the location of `axis`, normal to it.
    /// `MakePln::from_axis(axis)?`.
    pub fn from_axis(axis: Axis1) -> Result<Self, Failure> {
        Ok(Self::built(Plane3::new(axis.location(), axis.direction())?))
    }
}
