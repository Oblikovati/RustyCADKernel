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
// Rust translation of FoundationClasses/TKMath/ElCLib/ElCLib.cxx To3d at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! ElCLib::To3d: embeds planar geometry into the XY plane of a 3D frame.
use crate::gp::{
    Axis1, Axis2, Axis2d, Axis22d, Circle2, Circle3, Direction2, Direction3, Ellipse2, Ellipse3,
    Hyperbola2, Hyperbola3, Line2, Line3, Parabola2, Parabola3, Point2, Point3, Vector2, Vector3,
    Xyz,
};
use rustycad_tkernel::standard::Failure;

/// Maps planar coordinates to `x·X + y·Y + origin`; `to3d_point(frame,point)`.
pub fn to3d_point(position: Axis2, point: Point2) -> Point3 {
    let x = Xyz::from(position.x_direction());
    let y = Xyz::from(position.y_direction());
    Point3::from(x * point.x() + y * point.y() + position.location().xyz())
}
/// Maps a planar vector to `vx·X + vy·Y`; `to3d_vector(frame,vector)`.
pub fn to3d_vector(position: Axis2, vector: Vector2) -> Vector3 {
    frame_combination(position, vector.x(), vector.y())
}
/// Maps a planar direction and renormalizes it through gp_Dir; `to3d_direction(frame,direction)?`.
pub fn to3d_direction(position: Axis2, direction: Direction2) -> Result<Direction3, Failure> {
    Direction3::from_vector(frame_combination(position, direction.x(), direction.y()))
}
fn frame_combination(position: Axis2, x: f64, y: f64) -> Vector3 {
    Vector3::from(position.x_direction()) * x + Vector3::from(position.y_direction()) * y
}
/// Maps a planar axis; the mapped direction passes through gp_Vec and is normalized again by
/// gp_Ax1's gp_Dir parameter; `to3d_axis(frame,axis)?`.
pub fn to3d_axis(position: Axis2, axis: Axis2d) -> Result<Axis1, Failure> {
    let location = to3d_point(position, axis.location());
    let direction = Vector3::from(to3d_direction(position, axis.direction())?);
    Ok(Axis1::new(location, Direction3::from_vector(direction)?))
}
/// Maps a planar frame to the right-handed frame with normal `X' ^ Y'`; an indirect planar
/// frame therefore yields a reversed normal; `to3d_frame(frame,planar)?`.
pub fn to3d_frame(position: Axis2, frame: Axis22d) -> Result<Axis2, Failure> {
    let location = to3d_point(position, frame.location());
    let x = Vector3::from(to3d_direction(position, frame.x_direction())?);
    let y = Vector3::from(to3d_direction(position, frame.y_direction())?);
    let normal = Direction3::from_vector(x.crossed(y))?;
    Axis2::with_x_direction(location, normal, Direction3::from_vector(x)?)
}
/// Maps a planar line; `to3d_line(frame,line)?`.
pub fn to3d_line(position: Axis2, line: Line2) -> Result<Line3, Failure> {
    Ok(Line3::from_axis(to3d_axis(position, line.position())?))
}
/// Maps a planar circle; `to3d_circle(frame,circle)?`.
pub fn to3d_circle(position: Axis2, circle: Circle2) -> Result<Circle3, Failure> {
    Circle3::new(to3d_frame(position, circle.axis())?, circle.radius())
}
/// Maps a planar ellipse; `to3d_ellipse(frame,ellipse)?`.
pub fn to3d_ellipse(position: Axis2, ellipse: Ellipse2) -> Result<Ellipse3, Failure> {
    let frame = to3d_frame(position, ellipse.axis())?;
    Ellipse3::new(frame, ellipse.major_radius(), ellipse.minor_radius())
}
/// Maps a planar hyperbola; `to3d_hyperbola(frame,hyperbola)?`.
pub fn to3d_hyperbola(position: Axis2, hyperbola: Hyperbola2) -> Result<Hyperbola3, Failure> {
    let frame = to3d_frame(position, hyperbola.axis())?;
    Hyperbola3::new(frame, hyperbola.major_radius(), hyperbola.minor_radius())
}
/// Maps a planar parabola; `to3d_parabola(frame,parabola)?`.
pub fn to3d_parabola(position: Axis2, parabola: Parabola2) -> Result<Parabola3, Failure> {
    Parabola3::new(to3d_frame(position, parabola.axis())?, parabola.focal())
}
