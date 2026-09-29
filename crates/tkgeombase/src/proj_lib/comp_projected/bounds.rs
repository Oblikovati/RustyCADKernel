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
// Rust translation of ExactBound, DichExactBound and InitialPoint
// (ProjLib_CompProjectedCurve.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The bounds of a continuous part of the projection: the exact point where it
//! leaves the surface bounds (a projection with a fixed u or v), a dichotomy when
//! that fails, and the initial point of a part (the nearest normal projection).
use super::FUNC_TOL;
use super::derivatives::{Pair, d1};
use crate::extrema::{ExtAlgo, ExtFlag, ExtPS};
use crate::proj_lib::{PrjResolve, PrjResolveBox};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{self, Point2, Point3, Vector2};

/// The surface bounds `(FirstU, LastU, FirstV, LastV)`.
pub(super) fn surface_box(pair: Pair<'_>) -> Result<[f64; 4], Failure> {
    let s = pair.surface;
    Ok([s.first_u_parameter()?, s.last_u_parameter()?, s.first_v_parameter()?, s.last_v_parameter()?])
}

/// A bound candidate: the bound value, its distance along the projection
/// direction, and 2 (a U bound, fixed u) or 3 (a V bound, fixed v).
#[derive(Clone, Copy)]
struct Candidate {
    value: f64,
    distance: f64,
    fix: i32,
}

// The distances from (U0, V0) to each bound along the direction of the projection.
fn bound_distances(u0: f64, v0: f64, d2d: Vector2, b: [f64; 4]) -> [f64; 4] {
    let [first_u, last_u, first_v, last_v] = b;
    if d2d.x().abs() < gp::resolution() {
        let inf: f64 = precision::infinite();
        return [inf, inf, v0 - first_v, last_v - v0];
    }
    if d2d.y().abs() < gp::resolution() {
        let inf: f64 = precision::infinite();
        return [u0 - first_u, last_u - u0, inf, inf];
    }
    let origin: Point2 = Point2::new(u0, v0);
    let at_u = |u: f64| origin.distance(Point2::new(u, v0 + (u - u0) * d2d.y() / d2d.x()));
    let at_v = |v: f64| origin.distance(Point2::new(u0 + (v - v0) * d2d.x() / d2d.y(), v));
    [at_u(first_u), at_u(last_u), at_v(first_v), at_v(last_v)]
}

/// `ExactBound(Sol, NotSol, Tol, TolU, TolV, Curve, Surface)`: the point where the
/// projection crosses the nearest surface bound between `Sol.X` and `not_sol`; the
/// bounds are tried from the nearest; false when none is found.
pub(super) fn exact_bound(sol: &mut Point3, not_sol: f64, tol: f64, tol_uv: (f64, f64), pair: Pair<'_>) -> Result<bool, Failure> {
    let (u0, v0) = (sol.y(), sol.z());
    let b: [f64; 4] = surface_box(pair)?;
    let d2d: Vector2 = d1(sol.x(), u0, v0, pair)?;
    let r: [f64; 4] = bound_distances(u0, v0, d2d, b);
    let mut seq: Vec<Candidate> = vec![
        Candidate { value: b[0], distance: r[0], fix: 2 },
        Candidate { value: b[1], distance: r[1], fix: 2 },
        Candidate { value: b[2], distance: r[2], fix: 3 },
        Candidate { value: b[3], distance: r[3], fix: 3 },
    ];
    // The native bubble sort, largest distance first.
    for i in 1..=3 {
        for j in 0..(4 - i) {
            if seq[j].distance < seq[j + 1].distance {
                seq.swap(j, j + 1);
            }
        }
    }
    let t: f64 = sol.x();
    let (t1, t2) = (sol.x().min(not_sol), sol.x().max(not_sol));
    while let Some(p) = seq.pop() {
        let mut resolve = PrjResolve::new(pair.curve, pair.surface, p.fix)?;
        if p.fix == 2 {
            let bounds = PrjResolveBox { tol: Point2::new(tol, tol_uv.1), inf: Point2::new(t1, b[2]), sup: Point2::new(t2, b[3]) };
            resolve.perform(t, p.value, v0, bounds, FUNC_TOL, false)?;
            if !resolve.is_done() {
                continue;
            }
            let pons: Point2 = resolve.solution()?;
            *sol = Point3::new(pons.x(), p.value, pons.y());
            return Ok(true);
        }
        let bounds = PrjResolveBox { tol: Point2::new(tol, tol_uv.0), inf: Point2::new(t1, b[0]), sup: Point2::new(t2, b[1]) };
        resolve.perform(t, u0, p.value, bounds, FUNC_TOL, false)?;
        if !resolve.is_done() {
            continue;
        }
        let pons: Point2 = resolve.solution()?;
        *sol = Point3::new(pons.x(), pons.y(), p.value);
        return Ok(true);
    }
    Ok(false)
}

/// `DichExactBound(Sol, NotSol, Tol, TolU, TolV, Curve, Surface)`: halves the
/// parameter interval towards `not_sol` while the projection exists.
pub(super) fn dich_exact_bound(sol: &mut Point3, not_sol: f64, tol: f64, tol_uv: (f64, f64), pair: Pair<'_>) -> Result<(), Failure> {
    let (mut u0, mut v0) = (sol.y(), sol.z());
    let mut resolve = PrjResolve::new(pair.curve, pair.surface, 1)?;
    let b: [f64; 4] = surface_box(pair)?;
    let bounds = PrjResolveBox {
        tol: Point2::new(tol_uv.0, tol_uv.1),
        inf: Point2::new(b[0], b[2]),
        sup: Point2::new(b[1], b[3]),
    };
    let mut a_not_sol: f64 = not_sol;
    while (sol.x() - a_not_sol).abs() > tol {
        let t: f64 = (sol.x() + a_not_sol) / 2.;
        resolve.perform(t, u0, v0, bounds, FUNC_TOL, true)?;
        if resolve.is_done() {
            let pons: Point2 = resolve.solution()?;
            *sol = Point3::new(t, pons.x(), pons.y());
            (u0, v0) = (sol.y(), sol.z());
        } else {
            a_not_sol = t;
        }
    }
    Ok(())
}

/// `InitialPoint(Point, t, C, S, TolU, TolV, U, V, theMaxDist)`: the nearest
/// ExtPS minimum (within `max_dist` when positive) from which the orthogonal
/// projection of `C(t)` converges; None when there is none.
pub(super) fn initial_point(point: Point3, t: f64, pair: Pair<'_>, tol_uv: (f64, f64), max_dist: f64) -> Result<Option<(f64, f64)>, Failure> {
    let mut resolve = PrjResolve::new(pair.curve, pair.surface, 1)?;
    let b: [f64; 4] = surface_box(pair)?;
    let ext = ExtPS::with_range(point, pair.surface, (b[0], b[1]), (b[2], b[3]), tol_uv, (ExtFlag::Min, ExtAlgo::Grad))?;
    let mut sq_max: f64 = max_dist;
    if sq_max > 0. {
        sq_max *= sq_max;
    }
    let bounds = PrjResolveBox {
        tol: Point2::new(tol_uv.0, tol_uv.1),
        inf: Point2::new(b[0], b[2]),
        sup: Point2::new(b[1], b[3]),
    };
    let mut argmin: i32 = 0;
    if ext.is_done() && ext.nb_ext()? != 0 {
        for i in 1..=ext.nb_ext()? {
            if sq_max > 0. && sq_max < ext.square_distance(i)? {
                continue;
            }
            let (par_u, par_v) = ext.point(i)?.parameter();
            resolve.perform(t, par_u, par_v, bounds, FUNC_TOL, true)?;
            if resolve.is_done() && (argmin == 0 || ext.square_distance(i)? < ext.square_distance(argmin)?) {
                argmin = i;
            }
        }
    }
    if argmin == 0 {
        return Ok(None);
    }
    Ok(Some(ext.point(argmin)?.parameter()))
}
