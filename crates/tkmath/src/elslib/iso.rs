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
// Rust translation of FoundationClasses/TKMath/ElSLib/ElSLib.cxx *UIso/*VIso at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Iso-parametric lines and circles of elementary surfaces.
//!
//! ElSLib.cxx defines `No_Exception`, so the inline gp_Dir, gp_Ax2 and gp_Circ checks it
//! instantiates are compiled out: directions are normalized without the zero-norm raise and
//! circles accept a negative radius (for example a negative torus minor radius).
use super::first::{cone_d1, cylinder_d1};
use crate::gp::direction::{DirectionProducts, UncheckedProducts};
use crate::gp::{Axis2, Axis3, Circle3, Direction3, Line3, Vector3};

fn unchecked_direction(vector: Vector3) -> Direction3 {
    Direction3::normalize_unchecked(*vector.xyz().coordinates())
}
fn shifted_line(position: Axis3, offset: Vector3, direction: Direction3) -> Line3 {
    Line3::new(position.location(), direction).translated(offset)
}
/// Frame whose X axis points to longitude `u` and whose normal is `X ^ Z`.
fn meridian_frame(position: Axis3, u: f64) -> (Axis2, Direction3) {
    let dx = Vector3::from(position.x_direction());
    let dy = Vector3::from(position.y_direction());
    let radial = unchecked_direction(dx * u.cos() + dy * u.sin());
    let Ok(normal) = UncheckedProducts::crossed(radial, position.direction());
    let Ok(frame) = Axis2::frame_using::<UncheckedProducts>(position.location(), normal, radial);
    (frame, radial)
}
fn axial_offset(position: Axis3, distance: f64) -> Vector3 {
    Vector3::from(position.direction()) * distance
}

/// The line at `u` parallel to Y; `plane_u_iso(frame,1.0)`.
pub fn plane_u_iso(position: Axis3, u: f64) -> Line3 {
    let offset = Vector3::from(position.x_direction()) * u;
    shifted_line(position, offset, position.y_direction())
}
/// The line at `v` parallel to X; `plane_v_iso(frame,1.0)`.
pub fn plane_v_iso(position: Axis3, v: f64) -> Line3 {
    let offset = Vector3::from(position.y_direction()) * v;
    shifted_line(position, offset, position.x_direction())
}
/// The generatrix at longitude `u`; `cylinder_u_iso(frame,2.0,0.5)`.
pub fn cylinder_u_iso(position: Axis3, radius: f64, u: f64) -> Line3 {
    let (point, _du, dv) = cylinder_d1(u, 0.0, position, radius);
    Line3::new(point, unchecked_direction(dv))
}
/// The generatrix at longitude `u`; `cone_u_iso(frame,2.0,0.3,0.5)`.
pub fn cone_u_iso(position: Axis3, radius: f64, semi_angle: f64, u: f64) -> Line3 {
    let (point, _du, dv) = cone_d1(u, 0.0, position, radius, semi_angle);
    Line3::new(point, unchecked_direction(dv))
}
/// The meridian circle at longitude `u`; `sphere_u_iso(frame,2.0,0.5)`.
pub fn sphere_u_iso(position: Axis3, radius: f64, u: f64) -> Circle3 {
    let (frame, _radial) = meridian_frame(position, u);
    Circle3::from_parts_unchecked(frame, radius)
}
/// The tube circle at longitude `u`, centred at distance `R` from the axis;
/// `torus_u_iso(frame,5.0,1.0,0.5)`.
pub fn torus_u_iso(position: Axis3, major: f64, minor: f64, u: f64) -> Circle3 {
    let (frame, radial) = meridian_frame(position, u);
    let frame = frame.translated(Vector3::from(radial) * major);
    Circle3::from_parts_unchecked(frame, minor)
}
/// The parallel circle at height `v`; `cylinder_v_iso(frame,2.0,1.0)`.
pub fn cylinder_v_iso(position: Axis3, radius: f64, v: f64) -> Circle3 {
    let frame = position
        .ax2_unchecked()
        .translated(axial_offset(position, v));
    Circle3::from_parts_unchecked(frame, radius)
}
/// The parallel circle at generatrix distance `v`; a negative ring radius reverses X/Y;
/// `cone_v_iso(frame,2.0,0.3,1.0)`.
pub fn cone_v_iso(position: Axis3, radius: f64, semi_angle: f64, v: f64) -> Circle3 {
    let frame = position.translated(axial_offset(position, v * semi_angle.cos()));
    let ring = radius + v * semi_angle.sin();
    reversed_ring_circle(frame, ring)
}
/// The parallel circle at latitude `v`; beyond the poles the normal is reversed;
/// `sphere_v_iso(frame,2.0,0.3)`.
pub fn sphere_v_iso(position: Axis3, radius: f64, v: f64) -> Circle3 {
    let offset = axial_offset(position, radius * v.sin());
    let mut frame = position.ax2_unchecked().translated(offset);
    let mut ring = radius * v.cos();
    if ring < 0.0 {
        frame.set_direction_unchecked(frame.direction().reversed());
        ring = -ring;
    }
    Circle3::from_parts_unchecked(frame, ring)
}
/// The parallel circle at tube angle `v`; a negative ring radius reverses X/Y;
/// `torus_v_iso(frame,5.0,1.0,0.3)`.
pub fn torus_v_iso(position: Axis3, major: f64, minor: f64, v: f64) -> Circle3 {
    let frame = Axis3::from_axis2(position.ax2_unchecked())
        .translated(axial_offset(position, minor * v.sin()));
    reversed_ring_circle(frame, major + minor * v.cos())
}
fn reversed_ring_circle(mut frame: Axis3, ring: f64) -> Circle3 {
    let mut ring = ring;
    if ring < 0.0 {
        frame.x_reverse();
        frame.y_reverse();
        ring = -ring;
    }
    Circle3::from_parts_unchecked(frame.ax2_unchecked(), ring)
}
