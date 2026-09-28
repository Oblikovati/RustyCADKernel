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
//! (`Extrema_ExtElC(gp_Lin, gp_Lin, Tol)`) and the intersection of two 2D lines
//! (`IntAna2d_AnaIntersection(gp_Lin2d, gp_Lin2d)`).
//! They keep the native operation order; the complete Extrema and IntAna2d
//! packages will replace them when they are ported.
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{self, Line2, Line3, Point2, Point3, Xyz};

/// The result of `Extrema_ExtElC(gp_Lin, gp_Lin, Tol)`: always done, with one
/// extremum; parallel lines (myIsPar) have no extremum points.
#[derive(Clone, Copy, Debug)]
pub(crate) struct LinesExtremum {
    /// mySqDist[0]: the square distance of the extremum.
    pub square_distance: f64,
    /// myPoint[0]: the points of the extremum on each line (unset when parallel).
    pub points: Option<(Point3, Point3)>,
}

/// Extrema_ExtElC(theC1, theC2, Tol): the closest points of two lines, solving
/// `u1 - (D1.D2) u2 = L.D1`, `(D1.D2) u1 - u2 = L.D2`; the tolerance is unused.
pub(crate) fn lines_extremum(first: Line3, second: Line3) -> LinesExtremum {
    let (d1, d2) = (first.direction(), second.direction());
    let cos_a: f64 = d1.dot(d2);
    let sq_sin_a: f64 = 1.0 - cos_a * cos_a;
    let mut parameters: (f64, f64) = (0.0, 0.0);
    let mut parallel: bool =
        sq_sin_a < gp::resolution() || d1.is_parallel(d2, precision::angular());
    if !parallel {
        let l1l2: Xyz = second.location().xyz() - first.location().xyz();
        let (d1l, d2l) = (d1.xyz().dot(l1l2), d2.xyz().dot(l1l2));
        parameters = (
            (d1l - cos_a * d2l) / sq_sin_a,
            (cos_a * d1l - d2l) / sq_sin_a,
        );
        parallel = precision::is_infinite(parameters.0) || precision::is_infinite(parameters.1);
    }
    if parallel {
        let square_distance: f64 = second.square_distance(first.location());
        return LinesExtremum {
            square_distance,
            points: None,
        };
    }
    let p1: Point3 = elclib::value(parameters.0, &first);
    let p2: Point3 = elclib::value(parameters.1, &second);
    let square_distance: f64 = p1.square_distance(&p2);
    LinesExtremum {
        square_distance,
        points: Some((p1, p2)),
    }
}

/// The line-line case of IntAna2d_AnaIntersection, which is always done: one
/// point, parallel distinct lines or identical lines ([`lines2d_point`] reads it
/// as the callers do through IsEmpty and Point(1)).
pub(crate) fn lines2d_intersection(first: Line2, second: Line2) -> Lines2dIntersection {
    let (a1, b1, c1) = first.coefficients();
    let (a2, b2, c2) = second.coefficients();
    let det: f64 = a1.abs().max(a2.abs().max(b1.abs().max(b2.abs())));
    let (row1, row2) = pivot_rows(det, (a1, b1, c1), (a2, b2, c2));
    let (al1, be1, ga1) = row1;
    let (al2, be2, ga2) = row2;
    let rap: f64 = al2 / al1;
    let denom: f64 = be2 - rap * be1;
    if denom.abs() <= f64::EPSILON {
        let identical: bool = (ga2 - rap * ga1).abs() <= f64::EPSILON;
        return if identical {
            Lines2dIntersection::Identical
        } else {
            Lines2dIntersection::Parallel
        };
    }
    let mut xs: f64 = (be1 * ga2 / al1 - be2 * ga1 / al1) / denom;
    let mut ys: f64 = (rap * ga1 - ga2) / denom;
    if (a1.abs() != det && b1.abs() == det)
        || (a1.abs() != det && b1.abs() != det && a2.abs() != det)
    {
        std::mem::swap(&mut xs, &mut ys);
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
