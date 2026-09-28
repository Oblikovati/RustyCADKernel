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
// Rust translation of FoundationClasses/TKMath/ElCLib/ElCLib.cxx *Parameter at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Parameters of points projected onto elementary curves.
use super::periodic::normalize_angle;
use crate::gp::{
    Axis1, Axis2, Axis2d, Axis22d, Direction2, Direction3, Point2, Point3, Vector2, Vector3, Xyz,
};
use rustycad_tkernel::standard::Failure;

/// Signed distance along the axis direction of the point's offset; `line_parameter(axis,point)`.
pub fn line_parameter(position: Axis1, point: Point3) -> f64 {
    (point.xyz() - position.location().xyz()).dot(Xyz::from(position.direction()))
}
/// Angle in `[0, 2·PI]` of the point projected on the circle plane, or zero when the point is
/// at the centre or on the axis; `circle_parameter(frame,point)`.
pub fn circle_parameter(position: Axis2, point: Point3) -> f64 {
    let offset = Vector3::between(position.location(), point);
    if offset.square_magnitude() < f64::MIN_POSITIVE {
        // Coinciding points admit every parameter.
        return 0.0;
    }
    let normal = Xyz::from(position.direction());
    let projected = normal.cross_crossed(*offset.xyz(), normal);
    if projected.square_modulus() < f64::MIN_POSITIVE {
        return 0.0;
    }
    // The squared norm is at least DBL_MIN, so gp_Dir's zero-norm raise cannot fire.
    let toward = Direction3::normalize_unchecked(*projected.coordinates());
    let angle = position
        .x_direction()
        .angle_with_ref(toward, position.direction());
    normalize_angle(angle)
}
/// Eccentric angle in `[0, 2·PI]`, or zero on the axis; raises gp_VectorWithNullMagnitude when
/// the scaled offset underflows; `ellipse_parameter(frame,3.0,2.0,point)?`.
pub fn ellipse_parameter(
    position: Axis2,
    major: f64,
    minor: f64,
    point: Point3,
) -> Result<f64, Failure> {
    let offset = point.xyz() - position.location().xyz();
    let (x_axis, y_axis) = (
        Xyz::from(position.x_direction()),
        Xyz::from(position.y_direction()),
    );
    let (along_y, along_x) = (offset.dot(y_axis), offset.dot(x_axis));
    if along_x.abs() <= f64::MIN_POSITIVE && along_y.abs() <= f64::MIN_POSITIVE {
        // The point is on the axis of the ellipse.
        return Ok(0.0);
    }
    let circular = x_axis * along_x + y_axis * (along_y * (major / minor));
    let angle = Vector3::from(x_axis)
        .angle_with_ref(Vector3::from(circular), Vector3::from(position.direction()))?;
    Ok(normalize_angle(angle))
}
/// `asinh` of the Y offset over the minor radius (the major radius is unused);
/// `hyperbola_parameter(frame,3.0,2.0,point)`.
pub fn hyperbola_parameter(position: Axis2, _major: f64, minor: f64, point: Point3) -> f64 {
    let y = Vector3::from(position.y_direction());
    rustycad_cmath::asinh(Vector3::between(position.location(), point).dot(y) / minor)
}
/// Y offset of the point; `parabola_parameter(frame,point)`.
pub fn parabola_parameter(position: Axis2, point: Point3) -> f64 {
    Vector3::between(position.location(), point).dot(Vector3::from(position.y_direction()))
}

/// Signed distance along a planar axis; `line_parameter_2d(axis,point)`.
pub fn line_parameter_2d(position: Axis2d, point: Point2) -> f64 {
    (point.xy() - position.location().xy()).dot(position.direction().xy())
}
fn oriented(position: Axis22d, angle: f64) -> f64 {
    let direct = position.x_direction().crossed(position.y_direction()) >= 0.0;
    normalize_angle(if direct { angle } else { -angle })
}
/// Angle in `[0, 2·PI]` measured in the frame sense; the centre raises gp_Dir2d's zero-norm
/// error (unlike 3D); `circle_parameter_2d(frame,point)?`.
pub fn circle_parameter_2d(position: Axis22d, point: Point2) -> Result<f64, Failure> {
    let toward = Direction2::from_vector(Vector2::between(position.location(), point))?;
    Ok(oriented(position, position.x_direction().angle(toward)))
}
/// Eccentric angle in `[0, 2·PI]` in the frame sense; the centre raises
/// gp_VectorWithNullMagnitude; `ellipse_parameter_2d(frame,3.0,2.0,point)?`.
pub fn ellipse_parameter_2d(
    position: Axis22d,
    major: f64,
    minor: f64,
    point: Point2,
) -> Result<f64, Failure> {
    let offset = point.xy() - position.location().xy();
    let (x_axis, y_axis) = (position.x_direction().xy(), position.y_direction().xy());
    let circular = x_axis * offset.dot(x_axis) + y_axis * (offset.dot(y_axis) * (major / minor));
    let angle = Vector2::from(x_axis).angle(Vector2::from(circular))?;
    Ok(oriented(position, angle))
}
/// `asinh` of the Y offset over the minor radius; `hyperbola_parameter_2d(frame,3.0,2.0,point)`.
pub fn hyperbola_parameter_2d(position: Axis22d, _major: f64, minor: f64, point: Point2) -> f64 {
    let y = Vector2::from(position.y_direction());
    rustycad_cmath::asinh(Vector2::between(position.location(), point).dot(y) / minor)
}
/// Y offset of the point; `parabola_parameter_2d(frame,point)`.
pub fn parabola_parameter_2d(position: Axis22d, point: Point2) -> f64 {
    Vector2::between(position.location(), point).dot(Vector2::from(position.y_direction()))
}
