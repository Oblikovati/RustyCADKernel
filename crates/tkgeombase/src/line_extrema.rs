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
// Rust translation of the line cases of Extrema_ExtElC.cxx, Extrema_ExtPElC.cxx and
// IntAna2d_AnaIntersection_1.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The line computations of the Extrema and IntAna2d packages that the gce and GC
//! constructions call: the extremum between two 3D lines
//! (`Extrema_ExtElC(gp_Lin, gp_Lin, Tol)`), the projection of a point on a bounded
//! line (`Extrema_ExtPElC(gp_Pnt, gp_Lin, Tol, Uinf, Usup)`) and the intersection
//! of two 2D lines (`IntAna2d_AnaIntersection(gp_Lin2d, gp_Lin2d)`).
//! They keep the native operation order; the complete Extrema and IntAna2d
//! packages will replace them when they are ported.
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{self, Line2, Line3, Point2, Point3, Vector3, Xyz};

/// The result of `Extrema_ExtElC(gp_Lin, gp_Lin, Tol)`: always done, with one
/// extremum (NbExt() == 1).
#[derive(Clone, Copy, Debug)]
pub(crate) struct LinesExtremum {
    /// myIsPar: the lines are parallel, or the solution is infinite.
    pub parallel: bool,
    /// mySqDist[0]: the square distance of the extremum.
    pub square_distance: f64,
    /// The points of `Points(1, P1, P2)`: the default Extrema_POnCurv (the origin)
    /// on both lines when they are parallel, as the native arrays are never set then.
    pub points: (Point3, Point3),
}

impl LinesExtremum {
    /// The point that the callers' extremum loop keeps: the middle of the points of
    /// the only extremum when its square distance is below RealLast(), else (NaN)
    /// the default gp_Pnt, the origin.
    pub fn middle(&self) -> Point3 {
        if self.square_distance.is_nan() || self.square_distance >= f64::MAX {
            return Point3::default();
        }
        let (on1, on2) = self.points;
        Point3::from((on1.xyz() + on2.xyz()) / 2.0)
    }
}

/// Extrema_ExtElC(theC1, theC2, Tol): the closest points of two lines, solving
/// `u1 - (D1.D2) u2 = L.D1`, `(D1.D2) u1 - u2 = L.D2`; the tolerance is unused.
pub(crate) fn lines_extremum(first: Line3, second: Line3) -> LinesExtremum {
    let Some((u1, u2)) = extremum_parameters(first, second) else {
        return LinesExtremum {
            parallel: true,
            square_distance: second.square_distance(first.location()),
            points: (Point3::default(), Point3::default()),
        };
    };
    let p1: Point3 = elclib::value(u1, &first);
    let p2: Point3 = elclib::value(u2, &second);
    LinesExtremum {
        parallel: false,
        square_distance: p1.square_distance(&p2),
        points: (p1, p2),
    }
}

/// The parameters of the closest points of two lines, or `None` (myIsPar) when
/// the lines are parallel (square sine below gp::Resolution() or directions
/// parallel within Precision::Angular()) or the solution is infinite.
fn extremum_parameters(first: Line3, second: Line3) -> Option<(f64, f64)> {
    let (d1, d2) = (first.direction(), second.direction());
    let cos_a: f64 = d1.dot(d2);
    let sq_sin_a: f64 = 1.0 - cos_a * cos_a;
    if sq_sin_a < gp::resolution() || d1.is_parallel(d2, precision::angular()) {
        return None;
    }
    let l1l2: Xyz = second.location().xyz() - first.location().xyz();
    let (d1l, d2l) = (d1.xyz().dot(l1l2), d2.xyz().dot(l1l2));
    let u1: f64 = (d1l - cos_a * d2l) / sq_sin_a;
    let u2: f64 = (cos_a * d1l - d2l) / sq_sin_a;
    if precision::is_infinite(u1) || precision::is_infinite(u2) {
        return None;
    }
    Some((u1, u2))
}

/// `Extrema_ExtPElC(P, L, Tol, Uinf, Usup).Point(1).Value()`: the projection of
/// `point` on `line`, which exists when its parameter lies in
/// `[uinf - tol, usup + tol]`; otherwise the extremum is not done and `Point(1)`
/// raises StdFail_NotDone through NbExt().
pub(crate) fn line_projection(
    point: Point3,
    line: Line3,
    tolerance: f64,
    bounds: (f64, f64),
) -> Result<Point3, Failure> {
    let v1: Vector3 = Vector3::from(line.direction());
    let origin: Point3 = line.location();
    let distance: f64 = v1.dot(Vector3::between(origin, point));
    if !(distance >= bounds.0 - tolerance && distance <= bounds.1 + tolerance) {
        let message = format!(
            "Extrema_ExtPElC::NbExt() - the projection parameter {distance} is outside [{}, {}]; expected a done extremum",
            bounds.0 - tolerance,
            bounds.1 + tolerance
        );
        return Err(Failure::with_type("StdFail_NotDone", &message, ""));
    }
    Ok(origin.translated(v1 * distance))
}

/// The line-line case of IntAna2d_AnaIntersection, which is always done: one
/// point, parallel distinct lines or identical lines ([`lines2d_point`] reads it
/// as the callers do through IsEmpty and Point(1)).
pub(crate) fn lines2d_intersection(first: Line2, second: Line2) -> Lines2dIntersection {
    let (a1, b1, c1) = first.coefficients();
    let (a2, b2, c2) = second.coefficients();
    // std::max, not f64::max: the pivot choice must follow the native comparisons.
    let det: f64 = std_max(a1.abs(), std_max(a2.abs(), std_max(b1.abs(), b2.abs())));
    let ((al1, be1, ga1), (al2, be2, ga2)) = pivot_rows(det, (a1, b1, c1), (a2, b2, c2));
    let rap: f64 = al2 / al1;
    let denom: f64 = be2 - rap * be1;
    if denom.abs() <= f64::EPSILON {
        if (ga2 - rap * ga1).abs() <= f64::EPSILON {
            return Lines2dIntersection::Identical;
        }
        return Lines2dIntersection::Parallel;
    }
    let xs: f64 = (be1 * ga2 / al1 - be2 * ga1 / al1) / denom;
    let ys: f64 = (rap * ga1 - ga2) / denom;
    // The solution is in the pivoted variables: undo the exchange of X and Y.
    let exchanged: bool = (a1.abs() != det && b1.abs() == det)
        || (a1.abs() != det && b1.abs() != det && a2.abs() != det);
    if exchanged {
        return Lines2dIntersection::Point(Point2::new(ys, xs));
    }
    Lines2dIntersection::Point(Point2::new(xs, ys))
}

/// The outcome of the 2D line intersection.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Lines2dIntersection {
    /// One intersection point (NbPoints() == 1).
    Point(Point2),
    /// Parallel distinct lines: IsEmpty() is true.
    Parallel,
    /// Identical lines: IsEmpty() is false but there is no point.
    Identical,
}

/// `Intp.IsEmpty()` then `Intp.Point(1).Value()`: `Ok(None)` when empty,
/// Standard_OutOfRange for identical lines (NbPoints() == 0).
pub(crate) fn lines2d_point(intersection: Lines2dIntersection) -> Result<Option<Point2>, Failure> {
    match intersection {
        Lines2dIntersection::Point(point) => Ok(Some(point)),
        Lines2dIntersection::Parallel => Ok(None),
        Lines2dIntersection::Identical => Err(Failure::out_of_range(
            "IntAna2d_AnaIntersection::Point(1) - identical lines have 0 points; expected 1 <= N <= NbPoints()",
        )),
    }
}

type Row = (f64, f64, f64);

/// The row and column exchange of IntAna2d_AnaIntersection_1.cxx: the largest
/// coefficient `det` becomes the pivot `al1`.
fn pivot_rows(det: f64, first: Row, second: Row) -> (Row, Row) {
    let ((a1, b1, c1), (a2, b2, c2)) = (first, second);
    if a1.abs() == det {
        ((a1, b1, c1), (a2, b2, c2))
    } else if b1.abs() == det {
        ((b1, a1, c1), (b2, a2, c2))
    } else if a2.abs() == det {
        ((a2, b2, c2), (a1, b1, c1))
    } else {
        ((b2, a2, c2), (b1, a1, c1))
    }
}
