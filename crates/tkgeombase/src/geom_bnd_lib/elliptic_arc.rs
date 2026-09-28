// Copyright (c) 2025 OPEN CASCADE SAS
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
// Rust translation of the shared bodies of GeomBndLib_Circle(2d).cxx and
// GeomBndLib_Ellipse(2d).cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The analytic box of a circle or ellipse, 3D or 2D: the four GeomBndLib classes run
//! the same per-coordinate extremum analysis on `O + Ra cos(t) Xd + Rb sin(t) Yd`. They
//! differ only by the extremal-parameter tangent (`Yk / Xk` for a circle, `(Rb Yk) /
//! (Ra Xk)` for an ellipse) and by the point evaluator, which [`EllipticArc`] carries.
use super::space::{SampleBounds, SamplePoint};
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp;
use std::f64::consts::PI;

/// Which tangent the extremal parameter uses.
#[derive(Clone, Copy)]
pub enum ArcTangent {
    /// `atan(Yk / Xk)` (GeomBndLib_Circle, GeomBndLib_Circle2d).
    Circle,
    /// `atan((Rb Yk) / (Ra Xk))` (GeomBndLib_Ellipse, GeomBndLib_Ellipse2d).
    Ellipse,
}

/// One circle or ellipse: radii, frame coordinates and the point evaluator.
pub struct EllipticArc<'a, P: SamplePoint> {
    /// Major radius (the radius of a circle).
    pub major: f64,
    /// Minor radius (the radius of a circle).
    pub minor: f64,
    /// Centre coordinates.
    pub origin: [f64; 3],
    /// X direction coordinates.
    pub x_dir: [f64; 3],
    /// Y direction coordinates.
    pub y_dir: [f64; 3],
    /// The tangent rule of the extremal parameter.
    pub tangent: ArcTangent,
    /// The point at a parameter (ElCLib for 3D, the native lambda for 2D).
    pub eval: &'a dyn Fn(f64) -> P,
}

impl<P: SamplePoint> EllipticArc<'_, P> {
    /// The whole curve: `Update(O - A, O + A)` per coordinate with the amplitude
    /// `A = sqrt(Ra^2 Xk^2 + Rb^2 Yk^2)`, then `Enlarge(tol)`; `arc.full_box(0.0)`.
    pub fn full_box<B: SampleBounds<Point = P>>(&self, tolerance: f64) -> B {
        let mut min: Vec<f64> = Vec::with_capacity(P::DIM);
        let mut max: Vec<f64> = Vec::with_capacity(P::DIM);
        for k in 0..P::DIM {
            let (xk, yk) = (self.x_dir[k], self.y_dir[k]);
            let amplitude: f64 =
                (self.major * self.major * xk * xk + self.minor * self.minor * yk * yk).sqrt();
            min.push(self.origin[k] - amplitude);
            max.push(self.origin[k] + amplitude);
        }
        let mut bounds: B = B::default();
        bounds.update_min_max(&min, &max);
        bounds.enlarge_by(tolerance);
        bounds
    }
    // The extremal parameters (min, max) of coordinate k, ordered by value.
    fn extremal_parameters(&self, k: usize) -> (f64, f64) {
        let (xk, yk) = (self.x_dir[k], self.y_dir[k]);
        let mut t_min: f64 = PI / 2.0;
        if xk.abs() > gp::resolution() {
            let ratio: f64 = match self.tangent {
                ArcTangent::Circle => yk / xk,
                ArcTangent::Ellipse => (self.minor * yk) / (self.major * xk),
            };
            t_min = elclib::in_period(ratio.atan(), 0.0, 2.0 * PI);
        }
        let mut t_max: f64 = if t_min <= PI { t_min + PI } else { t_min - PI };
        let value = |t: f64| self.major * t.cos() * xk + self.minor * t.sin() * yk + self.origin[k];
        if value(t_min) > value(t_max) {
            std::mem::swap(&mut t_min, &mut t_max);
        }
        (t_min, t_max)
    }
    /// The arc `[u1, u2]` (already brought into period by the caller where the native
    /// class does): its two ends, then every coordinate extremum whose parameter lies in
    /// the arc; `arc.add_arc(&mut bounds, 0.0, 1.0)`.
    pub fn add_arc<B: SampleBounds<Point = P>>(&self, bounds: &mut B, u1: f64, u2: f64) {
        bounds.add_sample((self.eval)(u1));
        bounds.add_sample((self.eval)(u2));
        for k in 0..P::DIM {
            let (t_min, t_max) = self.extremal_parameters(k);
            for t in [t_min, t_max] {
                let tk: f64 = elclib::in_period(t, u1, u1 + 2.0 * PI);
                if tk >= u1 && tk <= u2 {
                    bounds.add_sample((self.eval)(t));
                }
            }
        }
    }
}

/// The 3D arc parameters of GeomBndLib_Circle/Ellipse: `ElCLib::AdjustPeriodic(0, 2Pi,
/// Epsilon(1.), U1, U2)`; `adjusted_arc(0.0, 7.0)`.
pub fn adjusted_arc(u1: f64, u2: f64) -> (f64, f64) {
    let tolerance: f64 = rustycad_tkernel::standard::real::epsilon(1.0);
    elclib::adjust_periodic(0.0, 2.0 * PI, tolerance, u1, u2)
}

/// Whether `[u1, u2]` spans a whole period (`U2 - U1 >= 2Pi - PConfusion`), which the
/// four classes answer with the full box; `is_full_period(0.0, 7.0)`.
pub fn is_full_period(u1: f64, u2: f64) -> bool {
    u2 - u1 >= 2.0 * PI - rustycad_tkernel::precision::p_confusion()
}
