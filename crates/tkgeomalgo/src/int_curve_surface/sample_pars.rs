// Copyright (c) 1993-1999 Matra Datavision
// Copyright (c) 1999-2024 OPEN CASCADE SAS
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
// Rust translation of the B-spline branch of IntCurveSurface_TheHCurveTool::SamplePars at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The B-spline sample parameters of `IntCurveSurface_TheHCurveTool::SamplePars`:
//! the knot spans of `[U0, U1]` split `Degree` times, then thinned while the chords
//! stay within the deflection.
use super::h_curve_tool::uniform_pars;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomCurve};
use rustycad_tkgeombase::gce::MakeLin;
use rustycad_tkmath::gp::{Line3, Point3};

/// `myMinPnts`: the fewest parameters SamplePars returns for a B-spline.
const MY_MIN_PNTS: i32 = 5;

/// The candidate parameters and their kept flags, both indexed from 1.
struct Candidates {
    pars: Vec<f64>,
    flags: Vec<bool>,
}

impl Candidates {
    fn len(&self) -> i32 {
        self.pars.len() as i32
    }
    fn par(&self, i: i32) -> f64 {
        self.pars[(i - 1) as usize]
    }
    fn flag(&self, i: i32) -> bool {
        self.flags[(i - 1) as usize]
    }
    fn set_flag(&mut self, i: i32) {
        self.flags[(i - 1) as usize] = true;
    }
}

/// `SamplePars` of a B-spline curve `bc` on `[u0, u1]`;
/// `bspline_sample_pars(&curve, 0.0, 1.0, 0.1, 10)?`.
pub(super) fn bspline_sample_pars(
    bc: &Handle<GeomBSplineCurve>,
    u0: f64,
    u1: f64,
    defl: f64,
    nb_min: i32,
) -> Result<rustycad_tkernel::ncollection::Array1<'static, f64>, Failure> {
    let mut candidates: Candidates = candidates(bc, u0, u1, nb_min)?;
    let nb_samples: i32 = thin(bc, &mut candidates, defl)?;
    if nb_samples < MY_MIN_PNTS {
        return uniform_pars(MY_MIN_PNTS, u0, u1);
    }
    let kept: Vec<f64> = (1..=candidates.len())
        .filter(|&i| candidates.flag(i))
        .map(|i| candidates.par(i))
        .collect();
    let mut pars = rustycad_tkernel::ncollection::Array1::with_bounds(1, nb_samples)
        .map_err(|e| e.into_failure("NCollection_HArray1"))?;
    for (j, par) in (1..).zip(kept) {
        pars[j] = par;
    }
    Ok(pars)
}

/// The knot span indices that hold `u0` and `u1` (the native search, which keeps the
/// first and last knot indices when a bound lies outside the knots).
fn span_indices(bc: &GeomBSplineCurve, u0: f64, u1: f64) -> Result<(i32, i32), Failure> {
    let (mut ui1, mut ui2) = (bc.first_u_knot_index()?, bc.last_u_knot_index()?);
    for i in ui1..ui2 {
        if u0 >= bc.knot(i)? && u0 < bc.knot(i + 1)? {
            ui1 = i;
            break;
        }
    }
    let mut i: i32 = ui2;
    while i > ui1 {
        if u1 <= bc.knot(i)? && u1 > bc.knot(i - 1)? {
            ui2 = i;
            break;
        }
        i -= 1;
    }
    Ok((ui1, ui2))
}

/// The candidate parameters: each span of `[u0, u1]` split `Degree` times, or
/// `nb_min` uniform ones when that gives fewer; no flag set.
fn candidates(bc: &GeomBSplineCurve, u0: f64, u1: f64, nb_min: i32) -> Result<Candidates, Failure> {
    let (ui1, ui2) = span_indices(bc, u0, u1)?;
    let degree: i32 = bc.degree();
    let mut nbsu: i32 = ui2 - ui1 + 1;
    nbsu += (nbsu - 1) * (degree - 1);
    if nbsu < nb_min {
        let pars: Vec<f64> = uniform_pars(nb_min, u0, u1)?.iter().copied().collect();
        let flags: Vec<bool> = vec![false; pars.len()];
        return Ok(Candidates { pars, flags });
    }
    let mut pars: Vec<f64> = Vec::with_capacity(nbsu as usize);
    let mut t1: f64 = u0;
    for i in ui1 + 1..=ui2 {
        let t2: f64 = if i == ui2 { u1 } else { bc.knot(i)? };
        let dt: f64 = (t2 - t1) / f64::from(degree);
        for _ in 0..degree.max(1) {
            pars.push(t1);
            t1 += dt;
        }
        t1 = t2;
    }
    pars.push(t1);
    let flags: Vec<bool> = vec![false; pars.len()];
    Ok(Candidates { pars, flags })
}

/// Flags the kept candidates (both ends, and each one before the first chord whose
/// inner points leave the deflection) and returns their count.
fn thin(bc: &GeomBSplineCurve, c: &mut Candidates, defl: f64) -> Result<i32, Failure> {
    let nbsu: i32 = c.len();
    let defl2: f64 = std_max(defl * defl, 1.0e-9);
    let tol: f64 = std_max(0.01 * defl2, 1.0e-9);
    let mut nb_samples: i32 = 2;
    c.set_flag(1);
    c.set_flag(nbsu);
    let mut j: i32 = 1;
    let mut cont: bool = true;
    while j < nbsu - 1 && cont {
        if c.flag(j + 1) {
            j += 1;
            continue;
        }
        let k: i32 = next_chord_end(bc, c, j, [defl2, tol], &mut nb_samples, &mut j)?;
        if k >= nbsu {
            cont = false;
        }
    }
    Ok(nb_samples)
}

/// One pass of the chord loop from `start`: the index `k` it stopped at (past the
/// end when it ran out), after moving `j` and flagging as the native loop does.
fn next_chord_end(
    bc: &GeomBSplineCurve,
    c: &mut Candidates,
    start: i32,
    [defl2, tol]: [f64; 2],
    nb_samples: &mut i32,
    j: &mut i32,
) -> Result<i32, Failure> {
    let nbsu: i32 = c.len();
    let p1: Point3 = bc.value(c.par(start))?;
    let mut k: i32 = start + 2;
    while k <= nbsu {
        let p2: Point3 = bc.value(c.par(k))?;
        if p1.square_distance(&p2) > tol {
            let lin: Line3 = MakeLin::from_points(p1, p2)?.value()?;
            if !chord_holds(bc, c, &lin, start + 1..k, defl2)? {
                *j = k - 1;
                c.set_flag(*j);
                *nb_samples += 1;
                return Ok(k);
            }
            if c.flag(k) {
                *j = k;
                return Ok(k);
            }
        }
        k += 1;
    }
    Ok(k)
}

/// Whether every candidate of `inner` is unflagged and within `defl2` (squared) of
/// the chord `lin`.
fn chord_holds(
    bc: &GeomBSplineCurve,
    c: &Candidates,
    lin: &Line3,
    inner: std::ops::Range<i32>,
    defl2: f64,
) -> Result<bool, Failure> {
    for l in inner {
        if c.flag(l) {
            return Ok(false);
        }
        if lin.square_distance(bc.value(c.par(l))?) > defl2 {
            return Ok(false);
        }
    }
    Ok(true)
}
