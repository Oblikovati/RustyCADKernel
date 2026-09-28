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
// Rust translation of FoundationClasses/TKMath/ElSLib/ElSLib.cxx *Parameters at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! (u, v) parameters of points expressed in a surface's local coordinate system.
use crate::elclib::normalize_angle;
use crate::gp::direction::{DirectionProducts, UncheckedProducts};
use crate::gp::{Axis3, Direction3, Point3, Transform3};

fn local(position: Axis3, point: Point3) -> Point3 {
    point.transformed(Transform3::transformation_to(position))
}

/// Local X and Y coordinates; `plane_parameters(frame,point)`.
pub fn plane_parameters(position: Axis3, point: Point3) -> (f64, f64) {
    let point = local(position, point);
    (point.x(), point.y())
}
/// Normalized longitude and local height (the radius is unused); `cylinder_parameters(frame,2.0,point)`.
pub fn cylinder_parameters(position: Axis3, _radius: f64, point: Point3) -> (f64, f64) {
    let point = local(position, point);
    (normalize_angle(point.y().atan2(point.x())), point.z())
}
/// Longitude (zero on the axis, flipped past the apex) and the generatrix distance;
/// `cone_parameters(frame,2.0,0.3,point)`.
pub fn cone_parameters(position: Axis3, radius: f64, semi_angle: f64, point: Point3) -> (f64, f64) {
    let point = local(position, point);
    let (x, y, z) = (point.x(), point.y(), point.z());
    let u = if x.abs() < f64::MIN_POSITIVE && y.abs() < f64::MIN_POSITIVE {
        0.0
    } else if -radius > z * semi_angle.tan() {
        (-y).atan2(-x)
    } else {
        y.atan2(x)
    };
    let u = normalize_angle(u);
    let v = semi_angle.sin() * (x * u.cos() + y * u.sin() - radius) + semi_angle.cos() * z;
    (u, v)
}
/// Normalized longitude and latitude in `[-PI/2, PI/2]`; on the axis the latitude is ±PI/2 and
/// the longitude zero (the radius is unused); `sphere_parameters(frame,2.0,point)`.
pub fn sphere_parameters(position: Axis3, _radius: f64, point: Point3) -> (f64, f64) {
    let point = local(position, point);
    let (x, y, z) = (point.x(), point.y(), point.z());
    let planar = (x * x + y * y).sqrt();
    if planar < f64::MIN_POSITIVE {
        let v = if z > 0.0 {
            std::f64::consts::FRAC_PI_2
        } else {
            -std::f64::consts::FRAC_PI_2
        };
        return (0.0, v);
    }
    (normalize_angle(y.atan2(x)), (z / planar).atan())
}
/// Longitude (choosing the nearer tube section when `R < r`) and the tube angle measured from
/// the outward radial direction; `torus_parameters(frame,5.0,1.0,point)`.
pub fn torus_parameters(position: Axis3, major: f64, minor: f64, point: Point3) -> (f64, f64) {
    let point = local(position, point);
    let (x, y, z) = (point.x(), point.y(), point.z());
    let mut u = y.atan2(x);
    if major < minor && nearer_opposite_section(x, y, z, u, major, minor) {
        u += std::f64::consts::PI;
    }
    let u = normalize_angle(u);
    (u, tube_angle(x, y, z, u, major))
}
fn nearer_opposite_section(x: f64, y: f64, z: f64, u: f64, major: f64, minor: f64) -> bool {
    let (cos_u, sin_u) = (u.cos(), u.sin());
    let squared_z = z * z;
    let squared_minor = minor * minor;
    let (radial_x, radial_y) = (major * cos_u, major * sin_u);
    let (near_x, near_y) = (x - radial_x, y - radial_y);
    let (far_x, far_y) = (x + radial_x, y + radial_y);
    let near = near_x * near_x + near_y * near_y + squared_z - squared_minor;
    let far = far_x * far_x + far_y * far_y + squared_z - squared_minor;
    far.abs() < near.abs()
}
fn tube_angle(x: f64, y: f64, z: f64, u: f64, major: f64) -> f64 {
    let (cos_u, sin_u) = (u.cos(), u.sin());
    // ElSLib is compiled with No_Exception, so these gp_Dir constructions never raise.
    let radial = Direction3::normalize_unchecked([cos_u, sin_u, 0.0]);
    let offset = [x - major * cos_u, y - major * sin_u, z];
    let magnitude = (offset[0] * offset[0] + offset[1] * offset[1] + offset[2] * offset[2]).sqrt();
    if magnitude <= f64::MIN_POSITIVE {
        return normalize_angle(0.0);
    }
    let toward = Direction3::normalize_unchecked(offset);
    let Ok(reference) = UncheckedProducts::crossed(radial, Direction3::Z);
    normalize_angle(radial.angle_with_ref(toward, reference))
}
