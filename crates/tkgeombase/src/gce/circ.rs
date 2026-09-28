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
// Rust translation of gce_MakeCirc.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! gce_MakeCirc: construction of gp_Circ.
use super::{ErrorType, gce_result};
use crate::line_extrema::lines_extremum;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{
    self, Axis1, Axis2, Circle3, Direction3, Line3, Plane3, Point3, StandardDirection3, Vector3,
};

/// gce_MakeCirc: builds a circle from a frame, a parallel circle, three points,
/// a center and a normal, or an axis.
///
/// `let circle = MakeCirc::new(Axis2::default(), 2.0)?.value()?;`
#[derive(Clone, Copy, Debug, Default)]
pub struct MakeCirc {
    the_result: Circle3,
    the_error: ErrorType,
}
gce_result!(MakeCirc, Circle3, "gce_MakeCirc");

impl MakeCirc {
    fn failed(error: ErrorType) -> Self {
        Self {
            the_error: error,
            ..Self::default()
        }
    }
    fn built(circle: Circle3) -> Self {
        Self {
            the_result: circle,
            the_error: ErrorType::Done,
        }
    }

    /// gce_MakeCirc(A2, Radius): the circle of frame `frame`; gce_NegativeRadius
    /// when `radius < 0`. `MakeCirc::new(Axis2::default(), 1.0)?`.
    pub fn new(frame: Axis2, radius: f64) -> Result<Self, Failure> {
        if radius < 0.0 {
            return Ok(Self::failed(ErrorType::NegativeRadius));
        }
        Ok(Self::built(Circle3::new(frame, radius)?))
    }

    /// gce_MakeCirc(Circ, Dist): the circle of `circle`'s frame with radius
    /// `Radius + dist`; gce_NegativeRadius when that radius is negative.
    /// `MakeCirc::from_circ_distance(circle, 1.0)?`.
    pub fn from_circ_distance(circle: Circle3, dist: f64) -> Result<Self, Failure> {
        let radius: f64 = circle.radius() + dist;
        if radius < 0.0 {
            return Ok(Self::failed(ErrorType::NegativeRadius));
        }
        Ok(Self::built(Circle3::new(circle.position(), radius)?))
    }

    /// gce_MakeCirc(Circ, Point): the circle of `circle`'s frame through the
    /// distance of `point` from its axis. `MakeCirc::from_circ_point(circle, p)?`.
    pub fn from_circ_point(circle: Circle3, point: Point3) -> Result<Self, Failure> {
        let radius: f64 = Line3::from_axis(circle.axis()).distance(point);
        Ok(Self::built(Circle3::new(circle.position(), radius)?))
    }

    /// gce_MakeCirc(Center, Norm, Radius): the circle of center `center` in the
    /// plane of normal `normal`, its X direction chosen from the smallest normal
    /// component. `MakeCirc::from_center_normal(p, n, 1.0)?`.
    pub fn from_center_normal(
        center: Point3,
        normal: Direction3,
        radius: f64,
    ) -> Result<Self, Failure> {
        if radius < 0.0 {
            return Ok(Self::failed(ErrorType::NegativeRadius));
        }
        let frame: Axis2 = normal_frame(center, normal, normal.xyz().coordinates())?;
        Ok(Self::built(Circle3::new(frame, radius)?))
    }

    /// gce_MakeCirc(Center, Plane, Radius): the circle of center `center` in a
    /// plane parallel to `plane`. The native constructor reads `Value()` of the
    /// center-normal construction, so a negative radius raises StdFail_NotDone.
    /// `MakeCirc::from_center_plane(p, plane, 1.0)?`.
    pub fn from_center_plane(center: Point3, plane: Plane3, radius: f64) -> Result<Self, Failure> {
        let maker: Self = Self::from_center_normal(center, plane.position().direction(), radius)?;
        Ok(Self {
            the_result: maker.value()?,
            the_error: maker.the_error,
        })
    }

    /// gce_MakeCirc(Center, Ptaxis, Radius): the circle of center `center` whose
    /// axis passes through `axis_point`; gce_NegativeRadius, then gce_NullAxis
    /// when the points coincide. `MakeCirc::from_center_axis_point(c, p, 1.0)?`.
    pub fn from_center_axis_point(
        center: Point3,
        axis_point: Point3,
        radius: f64,
    ) -> Result<Self, Failure> {
        if radius < 0.0 {
            return Ok(Self::failed(ErrorType::NegativeRadius));
        }
        if center.distance(&axis_point) <= gp::resolution() {
            return Ok(Self::failed(ErrorType::NullAxis));
        }
        let components: [f64; 3] = *(axis_point.xyz() - center.xyz()).coordinates();
        let normal: Direction3 = super::MakeDir::from_points(center, axis_point)?.value()?;
        let frame: Axis2 = normal_frame(center, normal, &components)?;
        Ok(Self::built(Circle3::new(frame, radius)?))
    }

    /// gce_MakeCirc(Axis, Radius): the circle of center and normal `axis`.
    /// `MakeCirc::from_axis(Axis1::default(), 1.0)?`.
    pub fn from_axis(axis: Axis1, radius: f64) -> Result<Self, Failure> {
        if radius < 0.0 {
            return Ok(Self::failed(ErrorType::NegativeRadius));
        }
        let normal: Direction3 = axis.direction();
        let frame: Axis2 = normal_frame(axis.location(), normal, normal.xyz().coordinates())?;
        Ok(Self::built(Circle3::new(frame, radius)?))
    }

    /// gce_MakeCirc(P1, P2, P3): the circle through three points, centered at the
    /// closest points of two perpendicular bisectors; gce_ConfusedPoints or
    /// gce_ColinearPoints for degenerate points. Three points within
    /// gp::Resolution() give a null circle with a status the native constructor
    /// never assigns (see [`super::Root`]). `MakeCirc::through_points(a, b, c)?`.
    pub fn through_points(p1: Point3, p2: Point3, p3: Point3) -> Result<Self, Failure> {
        let resolution: f64 = gp::resolution();
        let (dist1, dist2, dist3) = (p1.distance(&p2), p1.distance(&p3), p2.distance(&p3));
        if dist1 < resolution && dist2 < resolution && dist3 < resolution {
            return Ok(Self {
                the_result: null_circle(p1)?,
                the_error: ErrorType::Done,
            });
        }
        if dist1 < resolution || dist2 < resolution {
            return Ok(Self::failed(ErrorType::ConfusedPoints));
        }
        let dir1: Direction3 = Direction3::new(p2.x() - p1.x(), p2.y() - p1.y(), p2.z() - p1.z())?;
        let vdir2: Vector3 = Vector3::new(p3.x() - p2.x(), p3.y() - p2.y(), p3.z() - p2.z());
        if Line3::from_axis(Axis1::new(p1, dir1)).distance(p3) < resolution {
            return Ok(Self::failed(ErrorType::ColinearPoints));
        }
        let vdir3: Vector3 = Vector3::from(dir1).crossed(vdir2);
        if vdir3.square_magnitude() < resolution {
            return Ok(Self::failed(ErrorType::ColinearPoints));
        }
        Self::from_bisectors(p1, p2, p3, dir1, (vdir2, vdir3))
    }

    /// The end of gce_MakeCirc(P1, P2, P3): the center is the middle of the
    /// closest points of the bisectors of P1P2 and P2P3 in the plane of normal
    /// `vdir3`.
    fn from_bisectors(
        p1: Point3,
        p2: Point3,
        p3: Point3,
        dir1: Direction3,
        (vdir2, vdir3): (Vector3, Vector3),
    ) -> Result<Self, Failure> {
        let dir3: Direction3 = Direction3::from_vector(vdir3)?;
        let l1 = Line3::new(
            Point3::from((p1.xyz() + p2.xyz()) / 2.0),
            dir1.crossed(dir3)?,
        );
        let dir: Direction3 = Direction3::from_vector(vdir2.crossed(Vector3::from(dir3)))?;
        let l2 = Line3::new(Point3::from((p3.xyz() + p2.xyz()) / 2.0), dir);
        // Extrema_ExtElC(L1, L2, Precision::PConfusion()) is always done on two lines
        // and ignores its tolerance.
        let extremum = lines_extremum(l1, l2);
        if extremum.parallel {
            return Ok(Self::failed(ErrorType::IntersectionError));
        }
        // NbExt() is 1: the loop keeps the only extremum.
        let center: Point3 = extremum.middle();
        Self::around_center(center, [p1, p2, p3], vdir3)
    }

    /// The circle of center `center` through `points`, of mean radius.
    fn around_center(center: Point3, points: [Point3; 3], vdir3: Vector3) -> Result<Self, Failure> {
        let [p1, p2, p3] = points;
        let (dist1, dist2, dist3) = (
            p1.distance(&center),
            p2.distance(&center),
            p3.distance(&center),
        );
        if dist1 < gp::resolution() {
            return Ok(Self {
                the_result: null_circle(center)?,
                the_error: ErrorType::Done,
            });
        }
        let x = Direction3::new(
            p1.x() - center.x(),
            p1.y() - center.y(),
            p1.z() - center.z(),
        )?;
        let frame = Axis2::with_x_direction(center, Direction3::from_vector(vdir3)?, x)?;
        Ok(Self::built(Circle3::new(
            frame,
            (dist1 + dist2 + dist3) / 3.0,
        )?))
    }
}

/// `gp_Circ(gp_Ax2(P, gp::DX(), gp::DZ()), 0.)`: the null circle that the native
/// constructor returns for coincident points (main direction X, X direction Z).
fn null_circle(location: Point3) -> Result<Circle3, Failure> {
    let x: Direction3 = Direction3::from_standard(StandardDirection3::X);
    let z: Direction3 = Direction3::from_standard(StandardDirection3::Z);
    Circle3::new(Axis2::with_x_direction(location, x, z)?, 0.0)
}

/// The frame of gce_MakeCirc for a normal whose (unnormalized) components are
/// `abc`: the X direction lies in the coordinate plane of the two largest
/// components, `gp_Dir(-C, 0, A)`, `gp_Dir(0, -C, B)`, `gp_Dir(-B, A, 0)` or
/// their opposites.
fn normal_frame(center: Point3, normal: Direction3, abc: &[f64; 3]) -> Result<Axis2, Failure> {
    let [a, b, c] = *abc;
    let (aa, ba, ca) = (a.abs(), b.abs(), c.abs());
    let x: Direction3 = if ba <= aa && ba <= ca {
        if aa > ca {
            Direction3::new(-c, 0.0, a)
        } else {
            Direction3::new(c, 0.0, -a)
        }
    } else if aa <= ba && aa <= ca {
        if ba > ca {
            Direction3::new(0.0, -c, b)
        } else {
            Direction3::new(0.0, c, -b)
        }
    } else if aa > ba {
        Direction3::new(-b, a, 0.0)
    } else {
        Direction3::new(b, -a, 0.0)
    }?;
    Axis2::with_x_direction(center, normal, x)
}
