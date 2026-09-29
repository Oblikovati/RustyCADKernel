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
// Rust translation of Function_SetUVBounds (ProjLib_ComputeApprox.cxx), cylinder and
// torus cases, at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Function_SetUVBounds`: the parameter window of the projected curve. The curve
//! is sampled, the parameter angle unwrapped across the seam, and the window is
//! either the natural range or a period centred on the sampled range.
use super::function::{Projection, UvBounds, compute_step};
use crate::scan_step;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::geom::GeomSurfaceD1;
use rustycad_tkmath::elslib;
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};
use rustycad_tkmath::gp::{Cylinder3, Point3, Torus3};
use std::f64::consts::PI;

/// The unwrapping of one sampled angle: each new value is shifted by whole turns
/// to stay within PI of the previous one, and the extreme values are recorded with
/// the parameters where they occur.
pub(super) struct AngleTrack {
    delta: f64,
    prev: f64,
    pub(super) min: f64,
    pub(super) max: f64,
    pmin: f64,
    pmax: f64,
    dmax: f64,
}

impl AngleTrack {
    /// The track started at `start`, sampled from parameter `w1`.
    pub(super) fn new(start: f64, w1: f64) -> Self {
        Self {
            delta: 0.,
            prev: start,
            min: start,
            max: start,
            pmin: w1,
            pmax: w1,
            dmax: 0.,
        }
    }

    /// Adds the raw angle `raw` sampled at `par`.
    pub(super) fn push(&mut self, par: f64, raw: f64) {
        let eps: f64 = PI;
        let mut u: f64 = raw + self.delta;
        let mut d: f64 = u - self.prev;
        if d > eps {
            u -= self.delta;
            self.delta -= 2. * PI;
            u += self.delta;
            d = u - self.prev;
        } else if d < -eps {
            u -= self.delta;
            self.delta += 2. * PI;
            u += self.delta;
            d = u - self.prev;
        }
        self.dmax = self.dmax.max(d.abs());
        if u < self.min {
            self.min = u;
            self.pmin = par;
        }
        if u > self.max {
            self.max = u;
            self.pmax = par;
        }
        self.prev = u;
    }

    /// Widens the range by half the largest step at an extreme found inside the
    /// curve (not at an end).
    pub(super) fn widen(&mut self, w1: f64, w2: f64) {
        widen_range(
            &mut self.min,
            &mut self.max,
            [self.pmin, self.pmax],
            self.dmax,
            [w1, w2],
        );
    }

    /// The window: the range itself inside `[0, 2 PI]`, otherwise the period
    /// centred on it; returns `(first, last, couture)`.
    pub(super) fn window(&self) -> (f64, f64, bool) {
        centred_window(self.min, self.max)
    }
}

/// Moves `min` down (and `max` up) by half of `dmax` when its parameter is not at an
/// end of `[w1, w2]`.
pub(super) fn widen_range(min: &mut f64, max: &mut f64, p: [f64; 2], dmax: f64, w: [f64; 2]) {
    let pconf: f64 = precision::p_confusion();
    if (p[0] - w[0]).abs() > pconf && (p[0] - w[1]).abs() > pconf {
        *min -= dmax * 0.5;
    }
    if (p[1] - w[0]).abs() > pconf && (p[1] - w[1]).abs() > pconf {
        *max += dmax * 0.5;
    }
}

/// The range itself when inside `[0, 2 PI]`, else the period centred on it.
pub(super) fn centred_window(min: f64, max: f64) -> (f64, f64, bool) {
    if (0. ..=2. * PI).contains(&min) && (0. ..=2. * PI).contains(&max) {
        return (min, max, false);
    }
    let u: f64 = (min + max) / 2.;
    (u - PI, u + PI, true)
}

/// The next sample `par + step` of a window scan over `range`: the shared guard of
/// [`crate::scan_step`], which refuses the increment where the native scan
/// `for (par = W1 + Step; par <= W2; par += Step)` stalls (GeomProjLib::Curve2d of a
/// line trimmed to `[1, 1]` on a sphere never returns natively);
/// `par = next_scan_parameter(par, step, [s.w1, s.w2])?`.
pub(super) fn next_scan_parameter(par: f64, step: f64, range: [f64; 2]) -> Result<f64, Failure> {
    scan_step::next_scan_parameter("ProjLib_ComputeApprox", par, step, range)
}

/// The curve ends and middle sampled by the window search.
pub(super) struct Samples {
    pub(super) w1: f64,
    pub(super) w2: f64,
    pub(super) w: f64,
    pub(super) p1: Point3,
    pub(super) p2: Point3,
    pub(super) p: Point3,
}

impl Samples {
    fn of(projection: &Projection<'_>) -> Result<Self, Failure> {
        let curve = projection.curve;
        let (w1, w2) = (curve.first_parameter()?, curve.last_parameter()?);
        let w: f64 = 0.5 * (w1 + w2);
        Ok(Self {
            w1,
            w2,
            w,
            p1: curve.value(w1)?,
            p2: curve.value(w2)?,
            p: curve.value(w)?,
        })
    }
}

/// `Function_SetUVBounds`: the window of a curve on an elementary surface (no seam
/// for a plane or any other surface); `set_uv_bounds(&projection)?`.
pub(super) fn set_uv_bounds(projection: &Projection<'_>) -> Result<UvBounds, Failure> {
    let samples: Samples = Samples::of(projection)?;
    let mut bounds: UvBounds = UvBounds::default();
    match projection.surface.get_type()? {
        SurfaceType::Cone => super::cone_bounds::cone_bounds(projection, samples, &mut bounds)?,
        SurfaceType::Cylinder => cylinder_bounds(projection, &samples, &mut bounds)?,
        SurfaceType::Sphere => {
            super::sphere_bounds::sphere_bounds(projection, &samples, &mut bounds)?
        }
        SurfaceType::Torus => torus_bounds(projection, &samples, &mut bounds)?,
        _ => (bounds.u_couture, bounds.v_couture) = (false, false),
    }
    Ok(bounds)
}

fn cylinder_bounds(p: &Projection<'_>, s: &Samples, b: &mut UvBounds) -> Result<(), Failure> {
    let cylinder: Cylinder3 = p.surface.cylinder()?;
    b.v_couture = false;
    if p.curve.get_type()? == CurveType::Ellipse {
        return cylinder_ellipse_bounds(p, &cylinder, s, b);
    }
    let (u1, _) = elslib::parameters(&cylinder, s.p1);
    let step: f64 = compute_step(p.curve, cylinder.radius())?;
    let mut track: AngleTrack = AngleTrack::new(u1, s.w1);
    let plim: f64 = s.w2 + 0.1 * step;
    let mut par: f64 = s.w1 + step;
    while par <= plim {
        let (u, _) = elslib::parameters(&cylinder, p.curve.value(par)?);
        track.push(par, u);
        par = next_scan_parameter(par, step, [s.w1, s.w2])?;
    }
    track.widen(s.w1, s.w2);
    (b.u1, b.u2, b.u_couture) = track.window();
    Ok(())
}

// An ellipse on a cylinder: the window from its ends (and middle), in the sense of
// the curve for a closed ellipse.
fn cylinder_ellipse_bounds(
    p: &Projection<'_>,
    cylinder: &Cylinder3,
    s: &Samples,
    b: &mut UvBounds,
) -> Result<(), Failure> {
    let (u1, _) = elslib::parameters(cylinder, s.p1);
    let (u2, _) = elslib::parameters(cylinder, s.p2);
    let (mut u, _) = elslib::parameters(cylinder, s.p);
    b.u1 = u1.min(u2);
    b.u2 = u1.max(u2);
    b.u_couture = true;
    if !p.curve.is_closed()? {
        if b.u1 < u && u < b.u2 {
            u = (b.u1 + b.u2) / 2.;
            (b.u1, b.u2) = (u - PI, u + PI);
        } else {
            u = (b.u1 + b.u2) / 2.;
            (b.u1, b.u2) = if b.u1 < u {
                (u - 2. * PI, u)
            } else {
                (u, u + 2. * PI)
            };
        }
        return Ok(());
    }
    let (_, tangent) = p.curve.d1(s.w1)?;
    // The native code evaluates the surface at (U1, U2), not at (U1, V1).
    let d: GeomSurfaceD1 = p.surface.d1(u1, u2)?;
    if tangent.dot(d.d1u) > 0. {
        b.u2 = b.u1 + 2. * PI;
    } else {
        b.u2 = b.u1;
        b.u1 -= 2. * PI;
    }
    Ok(())
}

fn torus_bounds(p: &Projection<'_>, s: &Samples, b: &mut UvBounds) -> Result<(), Failure> {
    let torus: Torus3 = p.surface.torus()?;
    let (u1, v1) = elslib::parameters(&torus, s.p1);
    let step: f64 = compute_step(p.curve, torus.minor_radius())?;
    let mut track_u: AngleTrack = AngleTrack::new(u1, s.w1);
    let mut track_v: AngleTrack = AngleTrack::new(v1, s.w1);
    let plim: f64 = s.w2 + 0.1 * step;
    let mut par: f64 = s.w1 + step;
    while par <= plim {
        let (u, v) = elslib::parameters(&torus, p.curve.value(par)?);
        track_u.push(par, u);
        track_v.push(par, v);
        par = next_scan_parameter(par, step, [s.w1, s.w2])?;
    }
    track_u.widen(s.w1, s.w2);
    track_v.widen(s.w1, s.w2);
    (b.u1, b.u2, b.u_couture) = natural_or_centred(&track_u);
    (b.v1, b.v2, b.v_couture) = track_v.window();
    Ok(())
}

/// The U window of the sphere and torus: `[0, 2 PI]` itself when the range is
/// inside it, otherwise the centred period.
pub(super) fn natural_or_centred(track: &AngleTrack) -> (f64, f64, bool) {
    match track.window() {
        (_, _, false) => (0., 2. * PI, false),
        window => window,
    }
}
