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
// Rust translation of FoundationClasses/TKMath/ElSLib/ElSLib.cxx *DN at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Mixed partial derivatives `∂^(nu+nv)/∂u^nu ∂v^nv` of elementary surfaces.
//! Order sums use wrapping arithmetic so `i32` extremes stay defined (C++ overflow is not).
use super::frame::{Frame, flush, torus_tolerance, vector};
use crate::gp::{Axis3, Vector3, Xyz};
use rustycad_tkernel::standard::integer::is_odd;

fn zero() -> Vector3 {
    Vector3::new(0.0, 0.0, 0.0)
}
fn invalid_order(nu: i32, nv: i32) -> bool {
    nu.wrapping_add(nv) < 1 || nu < 0 || nv < 0
}
/// `(n + offset) % 4 == 0` evaluated as the native `int` expression.
fn quarter(n: i32, offset: i32) -> bool {
    n.wrapping_add(offset) % 4 == 0
}
fn negated_unless(xyz: Xyz, keep: bool) -> Vector3 {
    vector(if keep { xyz } else { xyz.reversed() })
}

/// X for `(1,0)`, Y for `(0,1)`, zero otherwise; `plane_dn(0.0,0.0,frame,1,0)`.
pub fn plane_dn(_u: f64, _v: f64, position: Axis3, nu: i32, nv: i32) -> Vector3 {
    match (nu, nv) {
        (0, 1) => Vector3::from(position.y_direction()),
        (1, 0) => Vector3::from(position.x_direction()),
        _ => zero(),
    }
}
/// Cone derivative from the direction at `u + nu·PI/2`; orders `nv > 1` vanish and the point
/// itself is returned for `(0,0)`; `cone_dn(0.5,1.0,frame,2.0,0.3,1,0)`.
pub fn cone_dn(
    u: f64,
    v: f64,
    position: Axis3,
    radius: f64,
    semi_angle: f64,
    nu: i32,
    nv: i32,
) -> Vector3 {
    let frame = Frame::of(position);
    let turned = u + f64::from(nu) * std::f64::consts::FRAC_PI_2;
    let radial = frame.x * turned.cos() + frame.y * turned.sin();
    if nv == 0 {
        let scaled = radial * (radius + v * semi_angle.sin());
        return vector(if nu == 0 {
            scaled + frame.origin
        } else {
            scaled
        });
    }
    if nv == 1 {
        let scaled = radial * semi_angle.sin();
        let axial = frame.z * semi_angle.cos();
        return vector(if nu == 0 { scaled + axial } else { scaled });
    }
    zero()
}
/// Cylinder derivative: cyclic in `nu` for `nv == 0`, the axis for `(0,1)`, zero otherwise;
/// `cylinder_dn(0.5,1.0,frame,2.0,2,0)`.
pub fn cylinder_dn(u: f64, _v: f64, position: Axis3, radius: f64, nu: i32, nv: i32) -> Vector3 {
    if invalid_order(nu, nv) {
        return Vector3::default();
    }
    if nv == 0 {
        let (cosine, sine) = (radius * u.cos(), radius * u.sin());
        let (a, b) = if quarter(nu, 6) {
            (-cosine, -sine)
        } else if quarter(nu, 5) {
            (sine, -cosine)
        } else if quarter(nu, 3) {
            (-sine, cosine)
        } else if nu % 4 == 0 {
            (cosine, sine)
        } else {
            (1.0, 1.0)
        };
        let frame = Frame::of(position);
        return vector(frame.x * a + frame.y * b);
    }
    if nv == 1 && nu == 0 {
        return Vector3::from(position.direction());
    }
    zero()
}
/// Sphere derivative following the native parity tables; `sphere_dn(0.5,0.2,frame,2.0,1,1)`.
pub fn sphere_dn(u: f64, v: f64, position: Axis3, radius: f64, nu: i32, nv: i32) -> Vector3 {
    if invalid_order(nu, nv) {
        return Vector3::default();
    }
    let (cos_u, sin_u, ring) = (u.cos(), u.sin(), radius * v.cos());
    let frame = Frame::of(position);
    if nu == 0 {
        let height = radius * v.sin();
        let local = if is_odd(nv) {
            frame.triple(-height * cos_u, -height * sin_u, ring)
        } else {
            frame.triple(-ring * cos_u, -ring * sin_u, -height)
        };
        return negated_unless(local, quarter(nv, 2) || quarter(nv, 3));
    }
    if nv == 0 {
        let local = if is_odd(nu) {
            frame.pair(-ring * sin_u, ring * cos_u)
        } else {
            frame.pair(ring * cos_u, ring * sin_u)
        };
        return negated_unless(local, !(quarter(nu, 2) || quarter(nu, 1)));
    }
    sphere_mixed(frame, (cos_u, sin_u), (ring, radius * v.sin()), nu, nv)
}
fn sphere_mixed(
    frame: Frame,
    (cos_u, sin_u): (f64, f64),
    (ring, height): (f64, f64),
    nu: i32,
    nv: i32,
) -> Vector3 {
    let (a1, a2) = if is_odd(nu) {
        (-sin_u, cos_u)
    } else {
        (-cos_u, -sin_u)
    };
    let a3 = if is_odd(nv) { -height } else { -ring };
    let local = frame.pair(a1, a2) * a3;
    let u_flips = quarter(nu, 2) || quarter(nu, 3);
    let v_flips = quarter(nv, 2) || quarter(nv, 3);
    // Negate when exactly one of the two orders falls in the flipping residues.
    negated_unless(local, u_flips == v_flips)
}
/// Torus derivative following the native parity tables, flushing terms within
/// `10·(r+R)·ε`; `torus_dn(0.5,0.2,frame,5.0,1.0,1,1)`.
pub fn torus_dn(
    u: f64,
    v: f64,
    position: Axis3,
    major: f64,
    minor: f64,
    nu: i32,
    nv: i32,
) -> Vector3 {
    if invalid_order(nu, nv) {
        return Vector3::default();
    }
    let torus = TorusTerms::new(u, v, major, minor);
    let frame = Frame::of(position);
    if nv == 0 {
        return torus.ring_dn(&frame, nu, major);
    }
    if nu == 0 {
        return torus.meridian_dn(&frame, nv);
    }
    torus.mixed_dn(&frame, nu, nv)
}
/// The trigonometric terms every TorusDN branch reads; each equals the native expression.
struct TorusTerms {
    cos_u: f64,
    sin_u: f64,
    tube_cos: f64,
    tube_sin: f64,
    tolerance: f64,
}
impl TorusTerms {
    fn new(u: f64, v: f64, major: f64, minor: f64) -> Self {
        Self {
            cos_u: u.cos(),
            sin_u: u.sin(),
            tube_cos: minor * v.cos(),
            tube_sin: minor * v.sin(),
            tolerance: torus_tolerance(major, minor),
        }
    }
    fn flush(&self, value: f64) -> f64 {
        flush(value, self.tolerance)
    }
    /// `nv == 0`: derivatives of the ring of radius `R + r·cos(v)`.
    fn ring_dn(&self, frame: &Frame, nu: i32, major: f64) -> Vector3 {
        let ring = major + self.tube_cos;
        let (a1, a2) = if is_odd(nu) {
            (-ring * self.sin_u, ring * self.cos_u)
        } else {
            (-ring * self.cos_u, -ring * self.sin_u)
        };
        let local = frame.pair(self.flush(a1), self.flush(a2));
        negated_unless(local, quarter(nu, 2) || quarter(nu, 3))
    }
    /// `nu == 0`: derivatives along the tube circle.
    fn meridian_dn(&self, frame: &Frame, nv: i32) -> Vector3 {
        let (cos_v, sin_v) = (self.tube_cos, self.tube_sin);
        let (a1, a2, a3) = if is_odd(nv) {
            (-sin_v * self.cos_u, -sin_v * self.sin_u, cos_v)
        } else {
            (-cos_v * self.cos_u, -cos_v * self.sin_u, -sin_v)
        };
        let local = frame.triple(self.flush(a1), self.flush(a2), self.flush(a3));
        negated_unless(local, quarter(nv, 2) || quarter(nv, 3))
    }
    fn mixed_dn(&self, frame: &Frame, nu: i32, nv: i32) -> Vector3 {
        let (a1, a2, flips) = self.mixed_terms(nu, nv);
        let local = frame.pair(self.flush(a1), self.flush(a2));
        negated_unless(local, !flips)
    }
    /// Parity table of the mixed orders: coefficients and whether the result is negated.
    fn mixed_terms(&self, nu: i32, nv: i32) -> (f64, f64, bool) {
        let (cos_u, sin_u, cosine, sine) = (self.cos_u, self.sin_u, self.tube_cos, self.tube_sin);
        match (is_odd(nu), is_odd(nv)) {
            (true, true) => (sine * sin_u, -sine * cos_u, false),
            (false, false) => (cosine * cos_u, cosine * sin_u, false),
            (true, false) => (
                cosine * sin_u,
                -cosine * cos_u,
                quarter(nv.wrapping_add(nu), 3),
            ),
            (false, true) => (sine * cos_u, sine * sin_u, quarter(nu.wrapping_add(nv), 3)),
        }
    }
}
