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
// Rust translation of FoundationClasses/TKMath/ElCLib/ElCLib.cxx 2D evaluators at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Points and first to third derivatives of planar elementary curves.
//!
//! The planar D2/D3 evaluators reverse the summed vector (`V2 = -(a·X + b·Y)`) where the 3D
//! ones sum negated terms; the values agree except for the sign of a zero result, so the
//! native order is kept per dimension.
use crate::gp::{Axis2d, Axis22d, Point2, Vector2, Xy};

/// A planar point with its first derivative.
pub type PlaneD1 = (Point2, Vector2);
/// A planar point with its first and second derivatives.
pub type PlaneD2 = (Point2, Vector2, Vector2);
/// A planar point with its first three derivatives.
pub type PlaneD3 = (Point2, Vector2, Vector2, Vector2);

struct Frame {
    x: Xy,
    y: Xy,
    origin: Xy,
}
impl Frame {
    fn of(position: Axis22d) -> Self {
        Self {
            x: position.x_direction().xy(),
            y: position.y_direction().xy(),
            origin: position.location().xy(),
        }
    }
    fn span(&self, a: f64, b: f64) -> Xy {
        self.x * a + self.y * b
    }
    fn point(&self, a: f64, b: f64) -> Point2 {
        Point2::from(self.span(a, b) + self.origin)
    }
}
fn degenerate_focal(focal: f64) -> bool {
    focal.abs() <= f64::MIN_POSITIVE
}

/// Point at `u` along a planar axis; `line_value_2d(2.0,axis)`.
pub fn line_value_2d(u: f64, position: Axis2d) -> Point2 {
    Point2::from(position.direction().xy() * u + position.location().xy())
}
/// Planar circle point; `circle_value_2d(0.5,frame,2.0)`.
pub fn circle_value_2d(u: f64, position: Axis22d, radius: f64) -> Point2 {
    Frame::of(position).point(radius * u.cos(), radius * u.sin())
}
/// Planar ellipse point; `ellipse_value_2d(0.5,frame,3.0,2.0)`.
pub fn ellipse_value_2d(u: f64, position: Axis22d, major: f64, minor: f64) -> Point2 {
    Frame::of(position).point(major * u.cos(), minor * u.sin())
}
/// Planar hyperbola point; `hyperbola_value_2d(0.5,frame,3.0,2.0)`.
pub fn hyperbola_value_2d(u: f64, position: Axis22d, major: f64, minor: f64) -> Point2 {
    Frame::of(position).point(major * u.cosh(), minor * u.sinh())
}
/// Planar parabola point, or the X axis line when `|F| <= resolution`; `parabola_value_2d(1.0,frame,2.0)`.
pub fn parabola_value_2d(u: f64, position: Axis22d, focal: f64) -> Point2 {
    let frame = Frame::of(position);
    if degenerate_focal(focal) {
        return Point2::from(frame.x * u + frame.origin);
    }
    frame.point(u * u / (4.0 * focal), u)
}

/// Point and unit tangent of a planar line; `line_d1_2d(2.0,axis)`.
pub fn line_d1_2d(u: f64, position: Axis2d) -> PlaneD1 {
    let direction = position.direction().xy();
    (
        Point2::from(direction * u + position.location().xy()),
        Vector2::from(direction),
    )
}
/// Planar circle point and first derivative; `circle_d1_2d(0.5,frame,2.0)`.
pub fn circle_d1_2d(u: f64, position: Axis22d, radius: f64) -> PlaneD1 {
    let (x, y) = (radius * u.cos(), radius * u.sin());
    let frame = Frame::of(position);
    (frame.point(x, y), Vector2::from(frame.span(-y, x)))
}
/// Planar ellipse point and first derivative; `ellipse_d1_2d(0.5,frame,3.0,2.0)`.
pub fn ellipse_d1_2d(u: f64, position: Axis22d, major: f64, minor: f64) -> PlaneD1 {
    let (cosine, sine) = (u.cos(), u.sin());
    let frame = Frame::of(position);
    (
        frame.point(cosine * major, sine * minor),
        Vector2::from(frame.span(-sine * major, cosine * minor)),
    )
}
/// Planar hyperbola point and first derivative; `hyperbola_d1_2d(0.5,frame,3.0,2.0)`.
pub fn hyperbola_d1_2d(u: f64, position: Axis22d, major: f64, minor: f64) -> PlaneD1 {
    let (cosh, sinh) = (u.cosh(), u.sinh());
    let frame = Frame::of(position);
    (
        frame.point(cosh * major, sinh * minor),
        Vector2::from(frame.span(sinh * major, cosh * minor)),
    )
}
/// Planar parabola point and first derivative; `parabola_d1_2d(1.0,frame,2.0)`.
pub fn parabola_d1_2d(u: f64, position: Axis22d, focal: f64) -> PlaneD1 {
    let frame = Frame::of(position);
    if degenerate_focal(focal) {
        return (
            Point2::from(frame.x * u + frame.origin),
            Vector2::from(frame.x),
        );
    }
    let tangent = frame.x * (u / (2.0 * focal)) + frame.y;
    (
        frame.point((u * u) / (4.0 * focal), u),
        Vector2::from(tangent),
    )
}

/// Planar circle point and derivatives up to the second; `circle_d2_2d(0.5,frame,2.0)`.
pub fn circle_d2_2d(u: f64, position: Axis22d, radius: f64) -> PlaneD2 {
    let (x, y) = (radius * u.cos(), radius * u.sin());
    let frame = Frame::of(position);
    let centred = frame.span(x, y);
    (
        Point2::from(centred + frame.origin),
        Vector2::from(frame.span(-y, x)),
        Vector2::from(centred.reversed()),
    )
}
/// Planar ellipse point and derivatives up to the second; `ellipse_d2_2d(0.5,frame,3.0,2.0)`.
pub fn ellipse_d2_2d(u: f64, position: Axis22d, major: f64, minor: f64) -> PlaneD2 {
    let (cosine, sine) = (u.cos(), u.sin());
    let frame = Frame::of(position);
    let centred = frame.span(cosine * major, sine * minor);
    (
        Point2::from(centred + frame.origin),
        Vector2::from(frame.span(-sine * major, cosine * minor)),
        Vector2::from(centred.reversed()),
    )
}
/// Planar hyperbola point and derivatives up to the second; `hyperbola_d2_2d(0.5,frame,3.0,2.0)`.
pub fn hyperbola_d2_2d(u: f64, position: Axis22d, major: f64, minor: f64) -> PlaneD2 {
    let (cosh, sinh) = (u.cosh(), u.sinh());
    let frame = Frame::of(position);
    let centred = frame.span(cosh * major, sinh * minor);
    (
        Point2::from(centred + frame.origin),
        Vector2::from(frame.span(sinh * major, cosh * minor)),
        Vector2::from(centred),
    )
}
/// Planar parabola point and derivatives up to the second; the tangent is `u·(X/(2F)) + Y`,
/// rounded differently from the 3D `(u/(2F))·X + Y`; `parabola_d2_2d(1.0,frame,2.0)`.
pub fn parabola_d2_2d(u: f64, position: Axis22d, focal: f64) -> PlaneD2 {
    let frame = Frame::of(position);
    if degenerate_focal(focal) {
        let point = Point2::from(frame.x * u + frame.origin);
        return (point, Vector2::from(frame.x), Vector2::new(0.0, 0.0));
    }
    let second = frame.x * (1.0 / (2.0 * focal));
    let first = second * u + frame.y;
    let point = frame.span(u * u / (4.0 * focal), u) + frame.origin;
    (
        Point2::from(point),
        Vector2::from(first),
        Vector2::from(second),
    )
}

/// Planar circle point and derivatives up to the third; `circle_d3_2d(0.5,frame,2.0)`.
pub fn circle_d3_2d(u: f64, position: Axis22d, radius: f64) -> PlaneD3 {
    let (point, first, second) = circle_d2_2d(u, position, radius);
    (point, first, second, first.reversed())
}
/// Planar ellipse point and derivatives up to the third; `ellipse_d3_2d(0.5,frame,3.0,2.0)`.
pub fn ellipse_d3_2d(u: f64, position: Axis22d, major: f64, minor: f64) -> PlaneD3 {
    let (point, first, second) = ellipse_d2_2d(u, position, major, minor);
    (point, first, second, first.reversed())
}
/// Planar hyperbola point and derivatives up to the third (odd derivatives coincide);
/// `hyperbola_d3_2d(0.5,frame,3.0,2.0)`.
pub fn hyperbola_d3_2d(u: f64, position: Axis22d, major: f64, minor: f64) -> PlaneD3 {
    let (point, first, second) = hyperbola_d2_2d(u, position, major, minor);
    (point, first, second, first)
}
