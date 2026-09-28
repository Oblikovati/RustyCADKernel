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
// Rust translation of FoundationClasses/TKMath/ElCLib/ElCLib.cxx *DN at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! N-th derivatives of elementary curves in 3D and 2D.
use crate::gp::{Axis1, Axis2, Axis2d, Axis22d, Vector2, Vector3, Xy, Xyz};
use rustycad_tkernel::standard::integer::{is_even, is_odd};

/// Chooses the cos/sin pattern of the N-th derivative by `N mod 4` in the native branch order
/// (`N == 1` first); wrapping arithmetic keeps `i32::MAX` defined where C++ overflow is not.
fn elliptic_coefficients(u: f64, major: f64, minor: f64, n: i32) -> (f64, f64) {
    if n == 1 {
        return (major * -u.sin(), minor * u.cos());
    }
    if n.wrapping_add(2) % 4 == 0 {
        return (major * -u.cos(), minor * -u.sin());
    }
    if n.wrapping_add(1) % 4 == 0 {
        return (major * u.sin(), minor * -u.cos());
    }
    if n % 4 == 0 {
        return (major * u.cos(), minor * u.sin());
    }
    if n.wrapping_sub(1) % 4 == 0 {
        return (major * -u.sin(), minor * u.cos());
    }
    (0.0, 0.0)
}
/// Odd derivatives use (sinh, cosh), even ones (cosh, sinh); a negative odd `n` is neither
/// under Standard_Integer's `n % 2 == 1` test, so its coefficients stay zero.
fn hyperbolic_coefficients(u: f64, major: f64, minor: f64, n: i32) -> (f64, f64) {
    if is_odd(n) {
        return (major * u.sinh(), minor * u.cosh());
    }
    if is_even(n) {
        return (major * u.cosh(), minor * u.sinh());
    }
    (0.0, 0.0)
}
fn space_span(position: Axis2, (a, b): (f64, f64)) -> Vector3 {
    let (x, y) = (
        Xyz::from(position.x_direction()),
        Xyz::from(position.y_direction()),
    );
    Vector3::from(x * a + y * b)
}
fn plane_span(position: Axis22d, (a, b): (f64, f64)) -> Vector2 {
    let (x, y) = (position.x_direction().xy(), position.y_direction().xy());
    Vector2::from(x * a + y * b)
}

/// Unit direction for `n == 1`, zero otherwise; `line_dn(0.0,axis,1)`.
pub fn line_dn(_u: f64, position: Axis1, n: i32) -> Vector3 {
    if n == 1 {
        return Vector3::from(position.direction());
    }
    Vector3::new(0.0, 0.0, 0.0)
}
/// N-th circle derivative; `circle_dn(0.5,frame,2.0,3)`.
pub fn circle_dn(u: f64, position: Axis2, radius: f64, n: i32) -> Vector3 {
    space_span(position, elliptic_coefficients(u, radius, radius, n))
}
/// N-th ellipse derivative; `ellipse_dn(0.5,frame,3.0,2.0,3)`.
pub fn ellipse_dn(u: f64, position: Axis2, major: f64, minor: f64, n: i32) -> Vector3 {
    space_span(position, elliptic_coefficients(u, major, minor, n))
}
/// N-th hyperbola derivative; `hyperbola_dn(0.5,frame,3.0,2.0,3)`.
pub fn hyperbola_dn(u: f64, position: Axis2, major: f64, minor: f64, n: i32) -> Vector3 {
    space_span(position, hyperbolic_coefficients(u, major, minor, n))
}
/// N-th parabola derivative; zero beyond the second or for `n <= 0`; the degenerate
/// `|F| <= resolution` case follows the X axis line; `parabola_dn(1.0,frame,2.0,1)`.
pub fn parabola_dn(u: f64, position: Axis2, focal: f64, n: i32) -> Vector3 {
    if !(1..=2).contains(&n) {
        return Vector3::new(0.0, 0.0, 0.0);
    }
    let x = Xyz::from(position.x_direction());
    let degenerate = focal.abs() <= f64::MIN_POSITIVE;
    if n == 1 {
        if degenerate {
            return Vector3::from(x);
        }
        return Vector3::from(x * (u / (2.0 * focal)) + Xyz::from(position.y_direction()));
    }
    if degenerate {
        return Vector3::new(0.0, 0.0, 0.0);
    }
    Vector3::from(x * (1.0 / (2.0 * focal)))
}

/// Unit direction for `n == 1`, zero otherwise; `line_dn_2d(0.0,axis,1)`.
pub fn line_dn_2d(_u: f64, position: Axis2d, n: i32) -> Vector2 {
    if n == 1 {
        return Vector2::from(position.direction());
    }
    Vector2::new(0.0, 0.0)
}
/// N-th planar circle derivative; `circle_dn_2d(0.5,frame,2.0,3)`.
pub fn circle_dn_2d(u: f64, position: Axis22d, radius: f64, n: i32) -> Vector2 {
    plane_span(position, elliptic_coefficients(u, radius, radius, n))
}
/// N-th planar ellipse derivative; `ellipse_dn_2d(0.5,frame,3.0,2.0,3)`.
pub fn ellipse_dn_2d(u: f64, position: Axis22d, major: f64, minor: f64, n: i32) -> Vector2 {
    plane_span(position, elliptic_coefficients(u, major, minor, n))
}
/// N-th planar hyperbola derivative; `hyperbola_dn_2d(0.5,frame,3.0,2.0,3)`.
pub fn hyperbola_dn_2d(u: f64, position: Axis22d, major: f64, minor: f64, n: i32) -> Vector2 {
    plane_span(position, hyperbolic_coefficients(u, major, minor, n))
}
/// N-th planar parabola derivative, as the 3D form; `parabola_dn_2d(1.0,frame,2.0,1)`.
pub fn parabola_dn_2d(u: f64, position: Axis22d, focal: f64, n: i32) -> Vector2 {
    if !(1..=2).contains(&n) {
        return Vector2::new(0.0, 0.0);
    }
    let x: Xy = position.x_direction().xy();
    let degenerate = focal.abs() <= f64::MIN_POSITIVE;
    if n == 1 {
        if degenerate {
            return Vector2::from(x);
        }
        return Vector2::from(x * (u / (2.0 * focal)) + position.y_direction().xy());
    }
    if degenerate {
        return Vector2::new(0.0, 0.0);
    }
    Vector2::from(x * (1.0 / (2.0 * focal)))
}
