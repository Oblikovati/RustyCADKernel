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
// Rust translation of the cone case of Function_SetUVBounds (ProjLib_ComputeApprox.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The window of a curve on a cone. Samples at the apex are moved by PConfusion
//! first, since the cone parameters of the apex give a wrong U; open conics use
//! their ends, other curves an unwrapping scan with seam protections.
use super::function::{Projection, UvBounds, compute_step};
use super::uv_bounds::{Samples, centred_window, next_scan_parameter, widen_range};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::epsilon;
use rustycad_tkmath::elslib;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Cone3, Line3, Point3};
use std::f64::consts::PI;

// OFV: `IsEqual(Check, With, Toler)`, a strict absolute comparison.
fn near(check: f64, with: f64, toler: f64) -> bool {
    (check - with).abs() < toler
}

/// The cone case of `Function_SetUVBounds`.
pub(super) fn cone_bounds(
    p: &Projection<'_>,
    mut s: Samples,
    b: &mut UvBounds,
) -> Result<(), Failure> {
    let cone: Cone3 = p.surface.cone()?;
    b.v_couture = false;
    move_off_apex(p, &cone, &mut s)?;
    match p.curve.get_type()? {
        CurveType::Parabola | CurveType::Hyperbola | CurveType::Ellipse => {
            conic_window(p, &cone, &s, b)
        }
        _ => scan_window(p, &cone, &s, b),
    }
}

// A sample on the apex is replaced by one PConfusion inside the curve.
fn move_off_apex(p: &Projection<'_>, cone: &Cone3, s: &mut Samples) -> Result<(), Failure> {
    let tol: f64 = epsilon(1.);
    let ptol: f64 = precision::p_confusion();
    let apex: Point3 = cone.apex();
    if apex.xyz().is_equal(&s.p1.xyz(), tol) {
        s.w1 += ptol;
        s.p1 = p.curve.value(s.w1)?;
    }
    if apex.xyz().is_equal(&s.p2.xyz(), tol) {
        s.w2 -= ptol;
        s.p2 = p.curve.value(s.w2)?;
    }
    if apex.xyz().is_equal(&s.p.xyz(), tol) {
        s.w += ptol;
        s.p = p.curve.value(s.w)?;
    }
    Ok(())
}

// Open conics: the ends give the window, a full period when the middle is outside
// or the curve is closed.
fn conic_window(
    p: &Projection<'_>,
    cone: &Cone3,
    s: &Samples,
    b: &mut UvBounds,
) -> Result<(), Failure> {
    let (u1, _) = elslib::parameters(cone, s.p1);
    let (u2, _) = elslib::parameters(cone, s.p2);
    let (u, _) = elslib::parameters(cone, s.p);
    b.u1 = u1.min(u2);
    b.u2 = u1.max(u2);
    if (u1 < u && u < u2) && !p.curve.is_closed()? {
        b.u_couture = false;
    } else {
        b.u_couture = true;
        b.u2 = b.u1 + 2. * PI;
    }
    Ok(())
}

/// The state of the cone scan: the unwrapped U and its extremes.
struct ConeScan {
    delta: f64,
    u1: f64,
    uf: f64,
    ul: f64,
    min: f64,
    max: f64,
    pmin: f64,
    pmax: f64,
    dmax: f64,
    is_first: bool,
    isclandper: bool,
}

impl ConeScan {
    // A jump of more than PI up: the seam is crossed (or the first or last point is
    // on it), so the value or the offset is corrected.
    fn jump_up(&mut self, mut u: f64, par: f64, step: f64, w2: f64) -> f64 {
        if ((near(u, 2. * PI, 1.0e-10) && (self.u1 >= 0. && self.u1 <= PI))
            && (near(u, self.ul, 1.0e-10) && !near(self.uf, 0., 1.0e-10)))
            && self.isclandper
        {
            u = 0.0;
        } else if self.is_first {
            // Protection against a first or last point on the seam.
            self.u1 = 2. * PI;
        } else if par + step >= w2 {
            u = 0.0;
        } else {
            self.delta -= 2. * PI;
        }
        u + self.delta
    }
    // The mirror of `jump_up` for a jump of more than PI down.
    fn jump_down(&mut self, mut u: f64, par: f64, step: f64, w2: f64) -> f64 {
        if ((near(u, 0., 1.0e-10) && (self.u1 >= PI && self.u1 <= 2. * PI))
            && (near(u, self.ul, 1.0e-10) && !near(self.uf, 2. * PI, 1.0e-10)))
            && self.isclandper
        {
            u = 2. * PI;
        } else if self.is_first {
            self.u1 = 0.0;
        } else if par + step >= w2 {
            u = 2. * PI;
        } else {
            self.delta += 2. * PI;
        }
        u + self.delta
    }
    fn push(&mut self, par: f64, raw: f64, step: f64, w2: f64) {
        let mut u: f64 = raw + self.delta;
        let mut d: f64 = u - self.u1;
        if d > PI {
            u = self.jump_up(u, par, step, w2);
            d = u - self.u1;
        } else if d < -PI {
            u = self.jump_down(u, par, step, w2);
            d = u - self.u1;
        }
        self.dmax = self.dmax.max(d.abs());
        if u < self.min {
            (self.min, self.pmin) = (u, par);
        }
        if u > self.max {
            (self.max, self.pmax) = (u, par);
        }
        self.u1 = u;
        self.is_first = false;
    }
}

// Other curves: the U of samples every `Step` (two steps for a closed or periodic
// curve), unwrapped across the seam.
fn scan_window(
    p: &Projection<'_>,
    cone: &Cone3,
    s: &Samples,
    b: &mut UvBounds,
) -> Result<(), Failure> {
    let (u1, _) = elslib::parameters(cone, s.p1);
    let (ul, _) = elslib::parameters(cone, s.p2);
    let axis: Line3 = Line3::from(cone.axis());
    let r: f64 = (axis.distance(s.p1) + axis.distance(s.p2) + axis.distance(s.p)) / 3.;
    let step: f64 = if p.curve.get_type()? == CurveType::Line {
        let nbp: i32 = 3;
        (s.w2 - s.w1) / f64::from(nbp - 1)
    } else {
        compute_step(p.curve, r)?
    };
    let isclandper: bool = !p.curve.is_closed()? && !p.curve.is_periodic()?;
    let mut scan = ConeScan {
        delta: 0.,
        u1,
        uf: u1,
        ul,
        min: u1,
        max: u1,
        pmin: s.w1,
        pmax: s.w1,
        dmax: 0.,
        is_first: true,
        isclandper,
    };
    let mut par: f64 = s.w1 + step;
    while par <= s.w2 {
        if !isclandper {
            par += step;
        }
        let (u, _) = elslib::parameters(cone, p.curve.value(par)?);
        scan.push(par, u, step, s.w2);
        par = next_scan_parameter(par, step, [s.w1, s.w2])?;
    }
    widen_range(
        &mut scan.min,
        &mut scan.max,
        [scan.pmin, scan.pmax],
        scan.dmax,
        [s.w1, s.w2],
    );
    (b.u1, b.u2, b.u_couture) = centred_window(scan.min, scan.max);
    Ok(())
}
