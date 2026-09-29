// Copyright (c) 1997-1999 Matra Datavision
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
// Rust translation of the point chain of the non-analytic case of
// ProjLib_ComputeApproxOnPolarSurface::BuildInitialCurve2d
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The projection of the following curve points on a Bezier, B-spline or offset
//! surface: a local search from the previous point, then a global one, then local
//! searches from the mirror of the previous point across the periodic bounds, then
//! a global search again. `usens` and `vsens` count the periods crossed.
use super::ComputeApproxOnPolarSurface;
use super::function::nearest;
use super::initial::Sampling;
use super::start_point::WalkStart;
use crate::extrema::{ExtAlgo, ExtFlag, ExtPS, GenLocateExtPS};
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::is_equal;
use rustycad_tkmath::gp::{Point2, Point3};

/// The chain state: the last surface parameters and the crossed periods.
struct Walk {
    u0: f64,
    v0: f64,
    usens: i32,
    vsens: i32,
    /// `myDist`: the largest squared distance of the chain.
    dist: f64,
}

impl Walk {
    // Stores the new parameters and the unwrapped point of index `i`.
    fn record(&mut self, s: &mut Sampling<'_>, i: i32, uv: (f64, f64)) {
        (self.u0, self.v0) = uv;
        let u1: f64 = self.u0 + f64::from(self.usens) * s.uperiod;
        let v1: f64 = self.v0 + f64::from(self.vsens) * s.vperiod;
        s.pts2d[i] = Point2::new(u1, v1);
    }
    fn keep_distance(&mut self, sq: f64) {
        if sq > self.dist {
            self.dist = sq;
        }
    }
}

impl ComputeApproxOnPolarSurface {
    /// The extremum chain of the non-analytic case, from the first point on.
    pub(super) fn project_by_extrema(&mut self, s: &mut Sampling<'_>) -> Result<(), Failure> {
        self.dist = 0.;
        let Some(start) = self.start_point(s)? else {
            return Ok(());
        };
        let WalkStart { u, v, dist2_min } = start;
        let mut walk = Walk {
            u0: u,
            v0: v,
            usens: 0,
            vsens: 0,
            dist: dist2_min,
        };
        for i in 2..=s.nb_of_pnts {
            if !self.proj_is_done {
                break;
            }
            self.proj_is_done = walk_step(&mut walk, s, i)?;
        }
        self.dist = walk.dist;
        Ok(())
    }
}

// One point of the chain: each strategy is tried until one succeeds.
fn walk_step(w: &mut Walk, s: &mut Sampling<'_>, i: i32) -> Result<bool, Failure> {
    let point: Point3 = s.pts[i];
    if local_then_global(w, s, i, point)? {
        return Ok(true);
    }
    if s.uperiod != 0. && mirrored_search(w, s, i, point, (true, false))? {
        return Ok(true);
    }
    if s.vperiod != 0. && mirrored_search(w, s, i, point, (false, true))? {
        return Ok(true);
    }
    if s.uperiod != 0. && s.vperiod != 0. && mirrored_search(w, s, i, point, (true, true))? {
        return Ok(true);
    }
    last_global(w, s, i, point)
}

// The local search from the previous point; when its extremum is too far, the
// global search, which also detects a period jump.
// The negated comparisons keep the native positive tests, so a NaN takes the native
// branch.
#[allow(clippy::neg_cmp_op_on_partial_ord)]
fn local_then_global(
    w: &mut Walk,
    s: &mut Sampling<'_>,
    i: i32,
    point: Point3,
) -> Result<bool, Failure> {
    let mut locate = GenLocateExtPS::new(s.surf, s.tol_u, s.tol_v);
    locate.perform(point, w.u0, w.v0, false)?;
    if !locate.is_done() {
        return Ok(false);
    }
    if locate.square_distance()? < s.dist_tol3d2 {
        w.keep_distance(locate.square_distance()?);
        let uv: (f64, f64) = locate.point()?.parameter();
        w.record(s, i, uv);
        return Ok(true);
    }
    let global = ExtPS::with_surface(point, s.surf, s.tol_u, s.tol_v, ExtFlag::Min, ExtAlgo::Grad)?;
    if !global.is_done() {
        return Ok(false);
    }
    let (mut local_min, mut imin): (f64, i32) = (f64::MAX, 0);
    for isol in 1..=global.nb_ext()? {
        let sq: f64 = global.square_distance(isol)?;
        if sq < local_min {
            (local_min, imin) = (sq, isol);
        }
    }
    if !(local_min < s.dist_tol3d2) {
        return Ok(false);
    }
    w.keep_distance(local_min);
    let (local_u, local_v) = global.point(imin)?.parameter();
    if s.uperiod > 0. && (w.u0 - local_u).abs() >= s.uperiod / 2. {
        w.usens = if local_u > w.u0 { -1 } else { 1 };
    }
    if s.vperiod > 0. && (w.v0 - local_v).abs() >= s.vperiod / 2. {
        w.vsens = if local_v > w.v0 { -1 } else { 1 };
    }
    w.record(s, i, (local_u, local_v));
    if i == 2 && (!is_equal(s.uperiod, 0.0) || !is_equal(s.vperiod, 0.0)) {
        refine_first_point(s);
    }
    Ok(true)
}

// Make the first point more precise on a periodic surface: the copy of it, shifted
// by -1, 0 or +1 period in each periodic direction, nearest to the second point.
fn refine_first_point(s: &mut Sampling<'_>) {
    let p: Point2 = s.pts2d[2];
    let first: Point2 = s.pts2d[1];
    let upar: [f64; 3] = [first.x() - s.uperiod, first.x(), first.x() + s.uperiod];
    let vpar: [f64; 3] = [first.y() - s.vperiod, first.y(), first.y() + s.vperiod];
    let size_u: usize = if is_equal(s.uperiod, 0.0) { 1 } else { 3 };
    let size_v: usize = if is_equal(s.vperiod, 0.0) { 1 } else { 3 };
    let (mut best_u, mut best_v, mut sq_min): (usize, usize, f64) = (1, 1, f64::MAX);
    for (u_ind, u) in upar.iter().enumerate().take(size_u) {
        for (v_ind, v) in vpar.iter().enumerate().take(size_v) {
            let sq: f64 = p.square_distance(Point2::new(*u, *v));
            if sq < sq_min {
                (sq_min, best_u, best_v) = (sq, u_ind, v_ind);
            }
        }
    }
    s.pts2d[1] = Point2::new(upar[best_u], vpar[best_v]);
}

// The start parameter mirrored across the nearer bound, one period further, and
// the period count change that goes with it.
fn mirror(x0: f64, inf: f64, sup: f64, period: f64) -> (f64, i32) {
    if (sup - x0) > (x0 - inf) {
        (2. * inf - x0 + period, -1)
    } else {
        (2. * sup - x0 - period, 1)
    }
}

// A local search from the mirrored start in U, V or both (`dirs`). The single
// direction cases read the bounds again from the surface, as natively.
// The negated comparisons keep the native positive tests, so a NaN takes the native
// branch.
#[allow(clippy::neg_cmp_op_on_partial_ord)]
fn mirrored_search(
    w: &mut Walk,
    s: &mut Sampling<'_>,
    i: i32,
    point: Point3,
    dirs: (bool, bool),
) -> Result<bool, Failure> {
    let both: bool = dirs.0 && dirs.1;
    let (uinf, usup) = if both {
        (s.bounds[0], s.bounds[1])
    } else {
        (s.surf.first_u_parameter()?, s.surf.last_u_parameter()?)
    };
    let (vinf, vsup) = if both {
        (s.bounds[2], s.bounds[3])
    } else {
        (s.surf.first_v_parameter()?, s.surf.last_v_parameter()?)
    };
    let (uaux, du) = if dirs.0 {
        mirror(w.u0, uinf, usup, s.uperiod)
    } else {
        (w.u0, 0)
    };
    let (vaux, dv) = if dirs.1 {
        mirror(w.v0, vinf, vsup, s.vperiod)
    } else {
        (w.v0, 0)
    };
    let mut locate = GenLocateExtPS::new(s.surf, s.tol_u, s.tol_v);
    locate.perform(point, uaux, vaux, false)?;
    if !locate.is_done() || !(locate.square_distance()? < s.dist_tol3d2) {
        return Ok(false);
    }
    w.keep_distance(locate.square_distance()?);
    let uv: (f64, f64) = locate.point()?.parameter();
    w.usens += du;
    w.vsens += dv;
    w.record(s, i, uv);
    Ok(true)
}

// The last resort: the nearest global extremum, with the period counts updated by
// the jump (two thirds of a period in U, half a period in V).
// The negated comparisons keep the native positive tests, so a NaN takes the native
// branch.
#[allow(clippy::neg_cmp_op_on_partial_ord)]
fn last_global(w: &mut Walk, s: &mut Sampling<'_>, i: i32, point: Point3) -> Result<bool, Failure> {
    let ext = ExtPS::with_surface(point, s.surf, s.tol_u, s.tol_v, ExtFlag::Min, ExtAlgo::Grad)?;
    if !ext.is_done() {
        return Ok(false);
    }
    // Natively SquareDistance(1) is read even without extrema (Standard_OutOfRange).
    let good: i32 = nearest(&ext)?;
    let dist2_min: f64 = ext.square_distance(good)?;
    if !(dist2_min < s.dist_tol3d2) {
        return Ok(false);
    }
    w.keep_distance(dist2_min);
    let (u, v) = ext.point(good)?.parameter();
    if s.uperiod != 0. {
        if (w.u0 - u) > (2. * s.uperiod / 3.) {
            w.usens += 1;
        } else if (u - w.u0) > (2. * s.uperiod / 3.) {
            w.usens -= 1;
        }
    }
    if s.vperiod != 0. {
        if (w.v0 - v) > (s.vperiod / 2.) {
            w.vsens += 1;
        } else if (v - w.v0) > (s.vperiod / 2.) {
            w.vsens -= 1;
        }
    }
    w.record(s, i, (u, v));
    Ok(true)
}
