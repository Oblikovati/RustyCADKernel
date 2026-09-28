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
// Rust translation of FoundationClasses/TKMath/ElCLib/ElCLib.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Points and first to third derivatives of 3D elementary curves from their placement.
use crate::gp::{Axis1, Axis2, Point3, Vector3, Xyz};

/// A point with its first derivative.
pub type SpaceD1 = (Point3, Vector3);
/// A point with its first and second derivatives.
pub type SpaceD2 = (Point3, Vector3, Vector3);
/// A point with its first three derivatives.
pub type SpaceD3 = (Point3, Vector3, Vector3, Vector3);

struct Frame {
    x: Xyz,
    y: Xyz,
    origin: Xyz,
}
impl Frame {
    fn of(position: Axis2) -> Self {
        Self {
            x: Xyz::from(position.x_direction()),
            y: Xyz::from(position.y_direction()),
            origin: position.location().xyz(),
        }
    }
    /// `a·X + b·Y` as gp_XYZ::SetLinearForm(a, X, b, Y).
    fn span(&self, a: f64, b: f64) -> Xyz {
        self.x * a + self.y * b
    }
    /// `a·X + b·Y + origin` as gp_XYZ::SetLinearForm(a, X, b, Y, origin).
    fn point(&self, a: f64, b: f64) -> Point3 {
        Point3::from(self.span(a, b) + self.origin)
    }
}
fn vector(xyz: Xyz) -> Vector3 {
    Vector3::from(xyz)
}
fn degenerate_focal(focal: f64) -> bool {
    focal.abs() <= f64::MIN_POSITIVE
}

/// Point at `u` along an axis, `origin + u·direction`; `line_value(2.0,axis)`.
pub fn line_value(u: f64, position: Axis1) -> Point3 {
    let (direction, origin) = (Xyz::from(position.direction()), position.location().xyz());
    Point3::from(direction * u + origin)
}
/// Circle point `R·cos(u)·X + R·sin(u)·Y + origin`; `circle_value(0.5,frame,2.0)`.
pub fn circle_value(u: f64, position: Axis2, radius: f64) -> Point3 {
    Frame::of(position).point(radius * u.cos(), radius * u.sin())
}
/// Ellipse point `a·cos(u)·X + b·sin(u)·Y + origin`; `ellipse_value(0.5,frame,3.0,2.0)`.
pub fn ellipse_value(u: f64, position: Axis2, major: f64, minor: f64) -> Point3 {
    Frame::of(position).point(major * u.cos(), minor * u.sin())
}
/// Hyperbola point `a·cosh(u)·X + b·sinh(u)·Y + origin`; `hyperbola_value(0.5,frame,3.0,2.0)`.
pub fn hyperbola_value(u: f64, position: Axis2, major: f64, minor: f64) -> Point3 {
    Frame::of(position).point(major * u.cosh(), minor * u.sinh())
}
/// Parabola point `u²/(4F)·X + u·Y + origin`, or the X axis line when `|F| <= resolution`;
/// `parabola_value(1.0,frame,2.0)`.
pub fn parabola_value(u: f64, position: Axis2, focal: f64) -> Point3 {
    let frame = Frame::of(position);
    if degenerate_focal(focal) {
        return Point3::from(frame.x * u + frame.origin);
    }
    frame.point(u * u / (4.0 * focal), u)
}

/// Point and unit tangent of a line; `line_d1(2.0,axis)`.
pub fn line_d1(u: f64, position: Axis1) -> SpaceD1 {
    let direction = Xyz::from(position.direction());
    (
        Point3::from(direction * u + position.location().xyz()),
        vector(direction),
    )
}
/// Circle point and first derivative; `circle_d1(0.5,frame,2.0)`.
pub fn circle_d1(u: f64, position: Axis2, radius: f64) -> SpaceD1 {
    let (x, y) = (radius * u.cos(), radius * u.sin());
    let frame = Frame::of(position);
    (frame.point(x, y), vector(frame.span(-y, x)))
}
/// Ellipse point and first derivative; `ellipse_d1(0.5,frame,3.0,2.0)`.
pub fn ellipse_d1(u: f64, position: Axis2, major: f64, minor: f64) -> SpaceD1 {
    let (cosine, sine) = (u.cos(), u.sin());
    let frame = Frame::of(position);
    (
        frame.point(cosine * major, sine * minor),
        vector(frame.span(-sine * major, cosine * minor)),
    )
}
/// Hyperbola point and first derivative; `hyperbola_d1(0.5,frame,3.0,2.0)`.
pub fn hyperbola_d1(u: f64, position: Axis2, major: f64, minor: f64) -> SpaceD1 {
    let (cosh, sinh) = (u.cosh(), u.sinh());
    let frame = Frame::of(position);
    (
        frame.point(cosh * major, sinh * minor),
        vector(frame.span(sinh * major, cosh * minor)),
    )
}
/// Parabola point and first derivative, or the X axis line when degenerate;
/// `parabola_d1(1.0,frame,2.0)`.
pub fn parabola_d1(u: f64, position: Axis2, focal: f64) -> SpaceD1 {
    let frame = Frame::of(position);
    if degenerate_focal(focal) {
        return (Point3::from(frame.x * u + frame.origin), vector(frame.x));
    }
    let tangent = frame.x * (u / (2.0 * focal)) + frame.y;
    (frame.point((u * u) / (4.0 * focal), u), vector(tangent))
}

/// Circle point and derivatives up to the second; `circle_d2(0.5,frame,2.0)`.
pub fn circle_d2(u: f64, position: Axis2, radius: f64) -> SpaceD2 {
    let (x, y) = (radius * u.cos(), radius * u.sin());
    let frame = Frame::of(position);
    let (point, first) = (frame.point(x, y), vector(frame.span(-y, x)));
    (point, first, vector(frame.span(-x, -y)))
}
/// Ellipse point and derivatives up to the second; `ellipse_d2(0.5,frame,3.0,2.0)`.
pub fn ellipse_d2(u: f64, position: Axis2, major: f64, minor: f64) -> SpaceD2 {
    let (cosine, sine) = (u.cos(), u.sin());
    let frame = Frame::of(position);
    (
        frame.point(cosine * major, sine * minor),
        vector(frame.span(-sine * major, cosine * minor)),
        vector(frame.span(-cosine * major, -sine * minor)),
    )
}
/// Hyperbola point and derivatives up to the second (the second derivative is the centred
/// point); `hyperbola_d2(0.5,frame,3.0,2.0)`.
pub fn hyperbola_d2(u: f64, position: Axis2, major: f64, minor: f64) -> SpaceD2 {
    let (cosh, sinh) = (u.cosh(), u.sinh());
    let frame = Frame::of(position);
    let centred = frame.span(cosh * major, sinh * minor);
    (
        Point3::from(centred + frame.origin),
        vector(frame.span(sinh * major, cosh * minor)),
        vector(centred),
    )
}
/// Parabola point and derivatives up to the second; zero curvature when degenerate;
/// `parabola_d2(1.0,frame,2.0)`.
pub fn parabola_d2(u: f64, position: Axis2, focal: f64) -> SpaceD2 {
    let frame = Frame::of(position);
    if degenerate_focal(focal) {
        let point = Point3::from(frame.x * u + frame.origin);
        return (point, vector(frame.x), Vector3::new(0.0, 0.0, 0.0));
    }
    let point = frame.point((u * u) / (4.0 * focal), u);
    let first = frame.x * (u / (2.0 * focal)) + frame.y;
    (
        point,
        vector(first),
        vector(frame.x * (1.0 / (2.0 * focal))),
    )
}

/// Circle point and derivatives up to the third; `circle_d3(0.5,frame,2.0)`.
pub fn circle_d3(u: f64, position: Axis2, radius: f64) -> SpaceD3 {
    let (x, y) = (radius * u.cos(), radius * u.sin());
    let (point, first, second) = circle_d2(u, position, radius);
    (
        point,
        first,
        second,
        vector(Frame::of(position).span(y, -x)),
    )
}
/// Ellipse point and derivatives up to the third; `ellipse_d3(0.5,frame,3.0,2.0)`.
pub fn ellipse_d3(u: f64, position: Axis2, major: f64, minor: f64) -> SpaceD3 {
    let (cosine, sine) = (u.cos(), u.sin());
    let (point, first, second) = ellipse_d2(u, position, major, minor);
    let third = Frame::of(position).span(sine * major, -cosine * minor);
    (point, first, second, vector(third))
}
/// Hyperbola point and derivatives up to the third (odd derivatives coincide);
/// `hyperbola_d3(0.5,frame,3.0,2.0)`.
pub fn hyperbola_d3(u: f64, position: Axis2, major: f64, minor: f64) -> SpaceD3 {
    let (point, first, second) = hyperbola_d2(u, position, major, minor);
    (point, first, second, first)
}
