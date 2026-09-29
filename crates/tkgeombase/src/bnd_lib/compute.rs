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
// Rust translation of the `Compute` template of BndLib.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Compute`: the box of an elliptic arc `O + Ra cos(t) Xd + Rb sin(t) Yd` from its two
//! ends and an 8-point circumscribed polygon (radii divided by cos(Pi/8), or by the
//! arrow of a short arc). The 2D circle/ellipse arcs and every torus section use it.
use crate::geom_bnd_lib::space::{SampleBounds, SamplePoint};
use rustycad_tkmath::bvh::BvhScalar;
use std::f64::consts::PI;

/// `THE_COS_PI8`: cos(Pi/8), the 8-point polygon factor (the native literal).
#[allow(clippy::excessive_precision)]
pub const COS_PI8: f64 = 0.92387953251128674;
/// `THE_COS_PI4`: cos(Pi/4) = sin(Pi/4) (the native literal).
#[allow(clippy::excessive_precision)]
pub const COS_PI4: f64 = 0.70710678118654746;
// The X and Y multipliers of the polygon vertex at `i * 45` degrees.
const X_MULT: [f64; 8] = [1.0, COS_PI4, 0.0, -COS_PI4, -1.0, -COS_PI4, 0.0, COS_PI4];
const Y_MULT: [f64; 8] = [0.0, COS_PI4, 1.0, COS_PI4, 0.0, -COS_PI4, -1.0, -COS_PI4];

/// One elliptic arc of `Compute`: radii and frame coordinates (`theXd`, `theYd`, `theO`).
pub struct ArcFrame<P: SamplePoint> {
    /// Radius along X (`theRa`).
    pub ra: f64,
    /// Radius along Y (`theRb`).
    pub rb: f64,
    /// X direction as a point.
    pub x_dir: P,
    /// Y direction as a point.
    pub y_dir: P,
    /// Centre.
    pub origin: P,
}

impl<P: SamplePoint> ArcFrame<P> {
    // `PointType(theO.Coord() + a * theXd.Coord() + b * theYd.Coord())`.
    fn point(&self, a: f64, b: f64) -> P {
        let values: Vec<f64> = (0..P::DIM)
            .map(|k| {
                self.origin.coordinate(k)
                    + self.x_dir.coordinate(k) * a
                    + self.y_dir.coordinate(k) * b
            })
            .collect();
        P::from_coordinates(&values)
    }
}

// The ordered arc `[teta1, teta2]`, `teta1` brought into `[0, 2Pi)` (the whole period for
// a span beyond 2Pi), and the span.
fn normalized_arc(p1: f64, p2: f64) -> (f64, f64, f64) {
    let (mut teta1, mut teta2) = if p2 < p1 { (p2, p1) } else { (p1, p2) };
    let delta: f64 = (teta2 - teta1).abs();
    if delta > 2.0 * PI {
        return (0.0, 2.0 * PI, delta);
    }
    teta1 %= 2.0 * PI;
    if teta1 < 0.0 {
        teta1 += 2.0 * PI;
    }
    teta2 = teta1 + delta;
    (teta1, teta2, delta)
}

/// `Compute(P1, P2, Ra, Rb, Xd, Yd, O, B)`: add the arc ends, the ends at the enlarged
/// radii and the polygon vertices `int(teta1 / (Pi/4)) + 1 ..= int(teta2 / (Pi/4))`
/// (modulo 8) to `bounds`; `compute(&mut bounds, 0.0, 1.0, &frame)`.
pub fn compute<B: SampleBounds>(bounds: &mut B, p1: f64, p2: f64, frame: &ArcFrame<B::Point>) {
    let (teta1, teta2, delta) = normalized_arc(p1, p2);
    let (cn1, sn1, cn2, sn2) = (teta1.cos(), teta1.sin(), teta2.cos(), teta2.sin());
    bounds.add_sample(frame.point(frame.ra * cn1, frame.rb * sn1));
    bounds.add_sample(frame.point(frame.ra * cn2, frame.rb * sn2));
    let (ram, rbm): (f64, f64) = if delta > PI / 8.0 {
        (frame.ra / COS_PI8, frame.rb / COS_PI8)
    } else {
        let arrow: f64 = (delta / 2.0).cos();
        (frame.ra / arrow, frame.rb / arrow)
    };
    bounds.add_sample(frame.point(ram * cn1, rbm * sn1));
    bounds.add_sample(frame.point(ram * cn2, rbm * sn2));
    // `static_cast<int>`: x86-64 truncation (INT_MIN when out of range or NaN).
    let first: i32 = (teta1 / (PI / 4.0)).truncate_to_int().wrapping_add(1);
    let last: i32 = (teta2 / (PI / 4.0)).truncate_to_int();
    for i in first..=last {
        let index: usize = (i % 8) as usize;
        bounds.add_sample(frame.point(ram * X_MULT[index], rbm * Y_MULT[index]));
    }
}
