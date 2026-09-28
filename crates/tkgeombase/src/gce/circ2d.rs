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
// Rust translation of gce_MakeCirc2d.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! gce_MakeCirc2d: construction of gp_Circ2d.
use super::{ErrorType, gce_result};
use crate::line_extrema::{lines2d_intersection, lines2d_point};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{
    self, Axis2d, Axis22d, Circle2, Direction2, Line2, Point2, StandardDirection2,
};

/// gce_MakeCirc2d: builds a 2D circle from an axis, a frame, three points, a
/// center, or a parallel circle.
///
/// `let circle = MakeCirc2d::from_center(Point2::default(), 1.0, true)?.value()?;`
#[derive(Clone, Copy, Debug, Default)]
pub struct MakeCirc2d {
    the_result: Circle2,
    the_error: ErrorType,
}
gce_result!(MakeCirc2d, Circle2, "gce_MakeCirc2d");

impl MakeCirc2d {
    fn failed(error: ErrorType) -> Self {
        Self {
            the_error: error,
            ..Self::default()
        }
    }
    fn built(circle: Circle2) -> Self {
        Self {
            the_result: circle,
            the_error: ErrorType::Done,
        }
    }

    /// gce_MakeCirc2d(XAxis, Radius, Sense): the circle whose X axis is `x_axis`;
    /// gce_NegativeRadius when `radius < 0`. `MakeCirc2d::new(axis, 1.0, true)?`.
    pub fn new(x_axis: Axis2d, radius: f64, sense: bool) -> Result<Self, Failure> {
        if radius.is_nan() || radius < 0.0 {
            return Ok(Self::failed(ErrorType::NegativeRadius));
        }
        Ok(Self::built(Circle2::from_x_axis(x_axis, radius, sense)?))
    }

    /// gce_MakeCirc2d(Axis, Radius): the circle of frame `frame`;
    /// gce_NegativeRadius when `radius < 0`. `MakeCirc2d::from_frame(frame, 1.0)?`.
    pub fn from_frame(frame: Axis22d, radius: f64) -> Result<Self, Failure> {
        if radius.is_nan() || radius < 0.0 {
            return Ok(Self::failed(ErrorType::NegativeRadius));
        }
        Ok(Self::built(Circle2::new(frame, radius)?))
    }

    /// gce_MakeCirc2d(Circ, Dist1): the circle of `circle`'s frame with radius
    /// `|Radius + dist|`; always done. `MakeCirc2d::from_circ_distance(c, 1.0)?`.
    pub fn from_circ_distance(circle: Circle2, dist: f64) -> Result<Self, Failure> {
        Ok(Self::built(Circle2::new(
            circle.axis(),
            (circle.radius() + dist).abs(),
        )?))
    }

    /// gce_MakeCirc2d(Circ, Point): the circle of `circle`'s frame through
    /// `point`. `MakeCirc2d::from_circ_point(circle, p)?`.
    pub fn from_circ_point(circle: Circle2, point: Point2) -> Result<Self, Failure> {
        Ok(Self::built(Circle2::new(
            circle.axis(),
            point.distance(circle.location()),
        )?))
    }

    /// gce_MakeCirc2d(Center, Radius, Sense): the circle of center `center` and X
    /// direction gp::DX2d(); gce_NegativeRadius when `radius < 0`.
    /// `MakeCirc2d::from_center(p, 1.0, true)?`.
    pub fn from_center(center: Point2, radius: f64, sense: bool) -> Result<Self, Failure> {
        if radius.is_nan() || radius < 0.0 {
            return Ok(Self::failed(ErrorType::NegativeRadius));
        }
        Ok(Self::built(Circle2::from_x_axis(
            x_axis_at(center),
            radius,
            sense,
        )?))
    }

    /// gce_MakeCirc2d(Center, Point, Sense): the circle of center `center` through
    /// `point`, X direction gp::DX2d(). `MakeCirc2d::from_center_point(c, p, true)?`.
    pub fn from_center_point(center: Point2, point: Point2, sense: bool) -> Result<Self, Failure> {
        let radius: f64 = point.distance(center);
        Ok(Self::built(Circle2::from_x_axis(
            x_axis_at(center),
            radius,
            sense,
        )?))
    }

    /// gce_MakeCirc2d(P1, P2, P3): the circle through three points, centered at
    /// the intersection of two perpendicular bisectors and oriented so that the
    /// points follow each other; gce_IntersectionError never happens because the
    /// line intersection is always done. Points within gp::Resolution() give a
    /// null circle; collinear points leave the status unassigned natively (see
    /// [`super::Root`]) and the circle at its default value.
    /// `MakeCirc2d::through_points(a, b, c)?`.
    pub fn through_points(p1: Point2, p2: Point2, p3: Point2) -> Result<Self, Failure> {
        let resolution: f64 = gp::resolution();
        let dists: [f64; 3] = [p1.distance(p2), p1.distance(p3), p2.distance(p3)];
        if dists.iter().all(|&dist| dist < resolution) {
            return Ok(Self::built(Circle2::from_x_axis(x_axis_at(p1), 0.0, true)?));
        }
        let (l1, l2) = bisectors([p1, p2, p3], dists)?;
        match lines2d_point(lines2d_intersection(l1, l2))? {
            Some(center) => Self::around_center(center, [p1, p2, p3]),
            None => Ok(Self::default()),
        }
    }

    /// The circle of center `center` through `points`, of mean radius, reversed
    /// unless the points' parameters increase.
    fn around_center(center: Point2, points: [Point2; 3]) -> Result<Self, Failure> {
        let [p1, p2, p3] = points;
        let (dist1, dist2, dist3) = (
            p1.distance(center),
            p2.distance(center),
            p3.distance(center),
        );
        let (xc, yc) = (center.x(), center.y());
        let d1 = Direction2::new(p1.x() - xc, p1.y() - yc)?;
        let d2 = Direction2::new(xc - p3.x(), yc - p3.y())?;
        let frame = Axis22d::with_directions(center, d1, d2)?;
        let mut circle = Circle2::new(frame, (dist1 + dist2 + dist3) / 3.0)?;
        let alpha1: f64 = elclib::parameter(&circle, p1)?;
        let alpha2: f64 = elclib::parameter(&circle, p2)?;
        let alpha3: f64 = elclib::parameter(&circle, p3)?;
        if alpha1 > alpha2 || alpha2 > alpha3 {
            circle.reverse();
        }
        Ok(Self::built(circle))
    }
}

/// `gp_Ax2d(Center, gp_Dir2d(gp_Dir2d::D::X))`.
fn x_axis_at(center: Point2) -> Axis2d {
    Axis2d::new(center, Direction2::from_standard(StandardDirection2::X))
}

/// The two bisectors of gce_MakeCirc2d(P1, P2, P3): of P1P2 and P1P3, or, when
/// two points are within gp::Resolution(), the normal through P1 that replaces
/// the missing one. A line never assigned stays the default gp_Lin2d().
fn bisectors(points: [Point2; 3], dists: [f64; 3]) -> Result<(Line2, Line2), Failure> {
    let [p1, p2, p3] = points;
    let [dist1, dist2, dist3] = dists;
    let resolution: f64 = gp::resolution();
    let (mut l1, mut l2) = (Line2::default(), Line2::default());
    if dist1 > resolution {
        l1 = Line2::new(Point2::from((p1.xy() + p2.xy()) / 2.0), normal_to(p1, p2)?);
    }
    if dist2 > resolution {
        l2 = Line2::new(Point2::from((p1.xy() + p3.xy()) / 2.0), normal_to(p1, p3)?);
    }
    if dist2 <= resolution {
        l2 = Line2::new(p1, normal_to(p1, p2)?);
    } else if dist1 <= resolution {
        l1 = Line2::new(p1, normal_to(p1, p3)?);
    } else if dist3 <= resolution {
        l2 = Line2::new(p1, normal_to(p1, p2)?);
    }
    Ok((l1, l2))
}

/// `gp_Dir2d(A.Y() - B.Y(), B.X() - A.X())`: the normal of the segment AB.
fn normal_to(a: Point2, b: Point2) -> Result<Direction2, Failure> {
    Direction2::new(a.y() - b.y(), b.x() - a.x())
}
