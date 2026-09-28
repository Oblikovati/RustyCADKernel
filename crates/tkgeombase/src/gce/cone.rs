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
// Rust translation of gce_MakeCone.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! gce_MakeCone: construction of gp_Cone.
use super::cylinder::perpendicular_hint;
use super::{ErrorType, Root, gce_result};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{self, Axis1, Axis2, Axis3, Cone3, Direction3, Line3, Point3, Xyz};
use std::f64::consts::PI;

/// gce_MakeCone: builds a cone from a frame, four points, an axis and two
/// points, two points and two radii, or a coaxial cone.
///
/// `let cone = MakeCone::new(Axis2::default(), 0.5, 1.0)?.value()?;`
#[derive(Clone, Copy, Debug, Default)]
pub struct MakeCone {
    the_result: Cone3,
    the_error: ErrorType,
}
gce_result!(MakeCone, Cone3, "gce_MakeCone");

impl MakeCone {
    fn failed(error: ErrorType) -> Self {
        Self {
            the_error: error,
            ..Self::default()
        }
    }
    fn built(cone: Cone3) -> Self {
        Self {
            the_result: cone,
            the_error: ErrorType::Done,
        }
    }

    /// gce_MakeCone(A2, Ang, Radius): the cone of frame `frame`, semi-angle
    /// `angle` and reference radius `radius`; gce_NegativeRadius, then
    /// gce_BadAngle when `angle` is not in (Resolution, PI/2 - Resolution).
    /// `MakeCone::new(frame, 0.5, 1.0)?`.
    pub fn new(frame: Axis2, angle: f64, radius: f64) -> Result<Self, Failure> {
        if radius < 0.0 {
            return Ok(Self::failed(ErrorType::NegativeRadius));
        }
        if angle <= gp::resolution() || PI / 2.0 - angle <= gp::resolution() {
            return Ok(Self::failed(ErrorType::BadAngle));
        }
        Ok(Self::built(Cone3::new(
            Axis3::from_axis2(frame),
            angle,
            radius,
        )?))
    }

    /// gce_MakeCone(P1, P2, P3, P4): the cone of axis P1P2 whose section circles
    /// pass through P3 and P4; gce_ConfusedPoints, gce_NullAngle (P3 and P4 project
    /// to the same point) or gce_NullRadius (null or right angle).
    /// `MakeCone::through_points(p1, p2, p3, p4)?`.
    pub fn through_points(p1: Point3, p2: Point3, p3: Point3, p4: Point3) -> Result<Self, Failure> {
        if p1.distance(&p2) < gp::resolution() || p3.distance(&p4) < gp::resolution() {
            return Ok(Self::failed(ErrorType::ConfusedPoints));
        }
        let d1: Direction3 = Direction3::from_xyz(p2.xyz() - p1.xyz())?;
        let pp4: Point3 = axis_projection(p1, d1, p4)?;
        let pp3: Point3 = axis_projection(p1, d1, p3)?;
        let (dist13, dist14) = (pp3.distance(&p1), pp4.distance(&p1));
        if (dist13 - dist14).abs() < gp::resolution() {
            return Ok(Self::failed(ErrorType::NullAngle));
        }
        let axis = Line3::new(p1, d1);
        let dif_rad: f64 = axis.distance(p3) - axis.distance(p4);
        let mut angle: f64 = (dif_rad / (dist13 - dist14)).atan().abs();
        if (PI / 2.0 - angle).abs() < gp::resolution() || angle.abs() < gp::resolution() {
            return Ok(Self::failed(ErrorType::NullRadius));
        }
        let (r1, r2) = (pp3.distance(&p3), pp4.distance(&p4));
        if r1 < 0.0 || r2 < 0.0 {
            return Ok(Self::failed(ErrorType::NegativeRadius));
        }
        let dd1: Direction3 = Direction3::from_xyz(pp4.xyz() - pp3.xyz())?;
        let frame = Axis2::with_x_direction(pp3, dd1, perpendicular_hint(dd1)?)?;
        if r1 > r2 {
            angle *= -1.0;
        }
        Ok(Self::built(Cone3::new(
            Axis3::from_axis2(frame),
            angle,
            r1,
        )?))
    }

    /// gce_MakeCone(Axis, P1, P2): the four-point cone of axis
    /// (Location, Location + Direction) through P1 and P2.
    /// `MakeCone::from_axis_points(axis, p1, p2)?`.
    pub fn from_axis_points(axis: Axis1, p1: Point3, p2: Point3) -> Result<Self, Failure> {
        Self::from_axis_through(axis.location(), axis.direction(), p1, p2)
    }

    /// gce_MakeCone(Axis, P1, P2) with a gp_Lin axis, as [`Self::from_axis_points`].
    /// `MakeCone::from_line_points(line, p1, p2)?`.
    pub fn from_line_points(axis: Line3, p1: Point3, p2: Point3) -> Result<Self, Failure> {
        Self::from_axis_through(axis.location(), axis.direction(), p1, p2)
    }

    /// The shared body of the axis constructors: the status of the four-point
    /// construction, and its cone when it is done.
    fn from_axis_through(
        location: Point3,
        direction: Direction3,
        p1: Point3,
        p2: Point3,
    ) -> Result<Self, Failure> {
        let p4: Point3 = Point3::from(location.xyz() + direction.xyz());
        let cone: Self = Self::through_points(location, p4, p1, p2)?;
        if cone.is_done() {
            return Ok(Self::built(cone.value()?));
        }
        Ok(Self::failed(cone.status()))
    }

    /// gce_MakeCone(Cone, P): the cone of `cone`'s frame and semi-angle whose
    /// surface passes through `point`, choosing the candidate reference radius
    /// nearest to the original one; gce_NegativeRadius when none is non-negative.
    /// `MakeCone::from_cone_point(cone, p)?`.
    pub fn from_cone_point(cone: Cone3, point: Point3) -> Result<Self, Failure> {
        let to_point: Xyz = point.xyz() - cone.location().xyz();
        let v_param: f64 = to_point.dot(cone.axis().direction().xyz());
        let radius: f64 = Line3::from_axis(cone.axis()).distance(point);
        let tan: f64 = cone.semi_angle().tan();
        let cos: f64 = cone.semi_angle().cos();
        let candidates: (f64, f64) = (radius - v_param * tan, -radius - v_param * tan);
        let result_radius: f64 = nearest_candidate(candidates, cone.ref_radius(), cos);
        if result_radius < 0.0 {
            return Ok(Self::failed(ErrorType::NegativeRadius));
        }
        Ok(Self::built(Cone3::new(
            cone.position(),
            cone.semi_angle(),
            result_radius,
        )?))
    }

    /// gce_MakeCone(Cone, Dist): the cone offset by `dist` along the normal of its
    /// surface, reference radius `RefRadius + dist / cos(SemiAngle)`;
    /// gce_NullAngle when the cosine is null, gce_NegativeRadius when the radius
    /// is negative. `MakeCone::from_cone_distance(cone, 1.0)?`.
    pub fn from_cone_distance(cone: Cone3, dist: f64) -> Result<Self, Failure> {
        let cos: f64 = cone.semi_angle().cos();
        if cos.abs() <= gp::resolution() {
            return Ok(Self::failed(ErrorType::NullAngle));
        }
        let radius: f64 = cone.ref_radius() + dist / cos;
        if radius < 0.0 {
            return Ok(Self::failed(ErrorType::NegativeRadius));
        }
        Ok(Self::built(Cone3::new(
            cone.position(),
            cone.semi_angle(),
            radius,
        )?))
    }

    /// gce_MakeCone(P1, P2, R1, R2): the cone of axis P1P2 with radius `r1` at P1
    /// and `r2` at P2; gce_NullAxis, gce_NegativeRadius, then gce_NullAngle for a
    /// null or right semi-angle. `MakeCone::from_points_radii(p1, p2, 1.0, 2.0)?`.
    pub fn from_points_radii(p1: Point3, p2: Point3, r1: f64, r2: f64) -> Result<Self, Failure> {
        let dist: f64 = p1.distance(&p2);
        if dist < gp::resolution() {
            return Ok(Self::failed(ErrorType::NullAxis));
        }
        if r1 < 0.0 || r2 < 0.0 {
            return Ok(Self::failed(ErrorType::NegativeRadius));
        }
        let mut angle: f64 = ((r1 - r2) / dist).atan().abs();
        if (PI / 2.0 - angle).abs() < gp::resolution() || angle.abs() < gp::resolution() {
            return Ok(Self::failed(ErrorType::NullAngle));
        }
        let d1: Direction3 = Direction3::from_xyz(p2.xyz() - p1.xyz())?;
        let frame = Axis2::with_x_direction(p1, d1, perpendicular_hint(d1)?)?;
        if r1 > r2 {
            angle *= -1.0;
        }
        Ok(Self::built(Cone3::new(
            Axis3::from_axis2(frame),
            angle,
            r1,
        )?))
    }
}

/// The projection of `point` on the axis (P1, D1) of the four-point cone:
/// `P1 + (D1.Dir(P - P1)) * |P1 P| * D1`.
fn axis_projection(p1: Point3, d1: Direction3, point: Point3) -> Result<Point3, Failure> {
    let cos: f64 = d1.dot(Direction3::from_xyz(point.xyz() - p1.xyz())?);
    let dist: f64 = p1.distance(&point);
    Ok(Point3::from(p1.xyz() + d1.xyz() * (cos * dist)))
}

/// The reference radius of gce_MakeCone(Cone, P): the candidates within
/// -gp::Resolution() of non-negative are clamped to 0, and of two valid ones the
/// nearer to `ref_radius` along the surface wins (the first on a tie); -1 when
/// none is valid.
fn nearest_candidate((first, second): (f64, f64), ref_radius: f64, cos: f64) -> f64 {
    let valid = |value: f64| value >= -gp::resolution();
    let clamp = |value: f64| if value < 0.0 { 0.0 } else { value };
    match (valid(first), valid(second)) {
        (true, true) => {
            let (cand1, cand2) = (clamp(first), clamp(second));
            let dist1: f64 = ((cand1 - ref_radius) * cos).abs();
            let dist2: f64 = ((cand2 - ref_radius) * cos).abs();
            if dist1 <= dist2 { cand1 } else { cand2 }
        }
        (true, false) => clamp(first),
        (false, true) => clamp(second),
        (false, false) => -1.0,
    }
}
