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
// Rust translation of the gp_Pln overloads of ProjLib::Project (ProjLib.cxx) and of
// ProjLib_Plane::Project (ProjLib_Plane.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ProjLib::Project(const gp_Pln&, ...)`: a point, line or conic expressed in the
//! 2D coordinates of a plane. A point uses `ElSLib::Parameters`; the curves use the
//! dot products of `ProjLib_Plane` (`EvalPnt2d`, `EvalDir2d`).
//!
//! The ProjLib package is not ported yet; this crate-private module ports exactly
//! these overloads for ExtremaPC, and the ProjLib port will replace it. The gp
//! constructors can raise (a null direction, parallel axes, a negative radius), so
//! each projection returns `Result` as the native exception would propagate.
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::elslib;
use rustycad_tkmath::gp::{
    Axis2, Axis22d, Circle2, Circle3, Direction2, Direction3, Ellipse2, Ellipse3, Hyperbola2,
    Hyperbola3, Line2, Line3, Parabola2, Parabola3, Plane3, Point2, Point3, Vector3,
};

/// `ProjLib::Project(Pl, P)`: `ElSLib::Parameters(Pl, P, U, V)`.
pub(crate) fn project_point(plane: &Plane3, point: Point3) -> Point2 {
    let (u, v) = elslib::parameters(plane, point);
    Point2::new(u, v)
}

// `EvalPnt2d`: the plane coordinates of a point by dot products with the plane axes.
fn eval_pnt2d(point: Point3, plane: &Plane3) -> Point2 {
    let op: Vector3 = Vector3::between(plane.location(), point);
    let position = plane.position();
    Point2::new(
        op.dot(Vector3::from(position.x_direction())),
        op.dot(Vector3::from(position.y_direction())),
    )
}

// `EvalDir2d`: the plane components of a direction, normalized by gp_Dir2d.
fn eval_dir2d(direction: Direction3, plane: &Plane3) -> Result<Direction2, Failure> {
    let position = plane.position();
    Direction2::new(
        direction.dot(position.x_direction()),
        direction.dot(position.y_direction()),
    )
}

// The gp_Ax22d of a conic position: its location, X and Y directions in the plane.
fn eval_frame(position: Axis2, plane: &Plane3) -> Result<Axis22d, Failure> {
    let location: Point2 = eval_pnt2d(position.location(), plane);
    let x: Direction2 = eval_dir2d(position.x_direction(), plane)?;
    let y: Direction2 = eval_dir2d(position.y_direction(), plane)?;
    Axis22d::with_directions(location, x, y)
}

/// `ProjLib::Project(Pl, L)`.
pub(crate) fn project_line(plane: &Plane3, line: &Line3) -> Result<Line2, Failure> {
    let location: Point2 = eval_pnt2d(line.location(), plane);
    Ok(Line2::new(location, eval_dir2d(line.direction(), plane)?))
}

/// `ProjLib::Project(Pl, C)`.
pub(crate) fn project_circle(plane: &Plane3, circle: &Circle3) -> Result<Circle2, Failure> {
    Circle2::new(eval_frame(circle.position(), plane)?, circle.radius())
}

/// `ProjLib::Project(Pl, E)`.
pub(crate) fn project_ellipse(plane: &Plane3, ellipse: &Ellipse3) -> Result<Ellipse2, Failure> {
    let frame: Axis22d = eval_frame(ellipse.position(), plane)?;
    Ellipse2::new(frame, ellipse.major_radius(), ellipse.minor_radius())
}

/// `ProjLib::Project(Pl, P)` of a parabola.
pub(crate) fn project_parabola(plane: &Plane3, parabola: &Parabola3) -> Result<Parabola2, Failure> {
    Parabola2::new(eval_frame(parabola.position(), plane)?, parabola.focal())
}

/// `ProjLib::Project(Pl, H)`.
pub(crate) fn project_hyperbola(
    plane: &Plane3,
    hyperbola: &Hyperbola3,
) -> Result<Hyperbola2, Failure> {
    let frame: Axis22d = eval_frame(hyperbola.position(), plane)?;
    Hyperbola2::new(frame, hyperbola.major_radius(), hyperbola.minor_radius())
}
