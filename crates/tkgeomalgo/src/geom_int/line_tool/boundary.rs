// Created on: 1995-02-08
// Created by: Jacques GOUSSARD
// Copyright (c) 1995-1999 Matra Datavision
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
// Rust translation of the static functions AdjustPeriodic, IsPointOnBoundary and
// FindPoint of ModelingAlgorithms/TKGeomAlgo/GeomInt/GeomInt_LineTool.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The parameter-space helpers of the walking line decomposition.
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Point2, Vector2};

/// `AdjustPeriodic(theParameter, parmin, parmax, thePeriod, theOffset)`: the parameter
/// shifted by whole periods into `[par_min, par_max]`, and the shift.
pub(super) fn adjust_periodic(
    parameter: f64,
    par_min: f64,
    par_max: f64,
    period: f64,
) -> (f64, f64) {
    let mut result: f64 = parameter;
    let mut offset: f64 = 0.0;
    while result < par_min {
        result += period;
        offset += period;
    }
    while result > par_max {
        result -= period;
        offset -= period;
    }
    (result, offset)
}

/// `IsPointOnBoundary(theParameter, theFirstBoundary, theSecondBoundary,
/// theResolution, IsOnFirstBoundary)`: Some(on first boundary) when the parameter is
/// within the resolution of a boundary (the first one is tested first).
pub(super) fn is_point_on_boundary(
    parameter: f64,
    first: f64,
    second: f64,
    resolution: f64,
) -> Option<bool> {
    if (parameter - first).abs() < resolution {
        return Some(true);
    }
    if (parameter - second).abs() < resolution {
        return Some(false);
    }
    None
}

/// The iso direction, its normal and the point on the boundary of the case `i` of
/// `FindPoint` (0 and 2 the U bounds, 1 and 3 the V bounds).
fn boundary_case(
    i: i32,
    last: Point2,
    (umin, umax, vmin, vmax): (f64, f64, f64, f64),
) -> (Vector2, Vector2, Point2) {
    let mut projpoint: Point2 = last;
    if i % 2 == 0 {
        projpoint.set_x(if i < 2 { umin } else { umax });
        return (Vector2::new(0.0, 1.0), Vector2::new(1.0, 0.0), projpoint);
    }
    projpoint.set_y(if i < 2 { vmin } else { vmax });
    (Vector2::new(1.0, 0.0), Vector2::new(0.0, 1.0), projpoint)
}

/// `FindPoint(theFirstPoint, theLastPoint, theUmin, theUmax, theVmin, theVmax,
/// theNewPoint)`: the point where the ray from `first` through `last` leaves the
/// parameter box after `last` (each boundary tried in turn, both offsets along it).
pub(super) fn find_point(
    first: Point2,
    last: Point2,
    bounds: (f64, f64, f64, f64),
) -> Result<Option<Point2>, Failure> {
    let (umin, umax, vmin, vmax) = bounds;
    let vec = Vector2::between(first, last);
    for i in 0..4 {
        let (other_vec, other_vec_normal, projpoint) = boundary_case(i, last, bounds);
        let normvec: Vector2 = vec.normalized()?;
        let adot1: f64 = normvec.dot(other_vec_normal);
        if adot1.abs() < precision::angular() {
            continue;
        }
        let adist: f64 = match (i % 2 == 0, i < 2) {
            (true, true) => (last.x() - umin).abs(),
            (true, false) => (last.x() - umax).abs(),
            (false, true) => (last.y() - vmin).abs(),
            (false, false) => (last.y() - vmax).abs(),
        };
        let mut anoffset: f64 = adist * other_vec.dot(normvec) / adot1;
        for j in 0..2 {
            anoffset = if j == 0 { anoffset } else { -anoffset };
            let curpoint = Point2::from(projpoint.xy() + other_vec.xy() * anoffset);
            let curvec = Vector2::between(last, curpoint);
            let dot_x: f64 = vec.dot(curvec);
            let angle_x: f64 = vec.angle(curvec)?;
            if !(dot_x > 0.0 && angle_x.abs() < precision::p_confusion()) {
                continue;
            }
            let inside: bool = if i % 2 == 0 {
                curpoint.y() >= vmin && curpoint.y() <= vmax
            } else {
                curpoint.x() >= umin && curpoint.x() <= umax
            };
            if inside {
                return Ok(Some(curpoint));
            }
        }
    }
    Ok(None)
}
