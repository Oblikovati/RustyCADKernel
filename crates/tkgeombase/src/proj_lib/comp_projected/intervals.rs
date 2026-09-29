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
// Rust translation of ProjLib_CompProjectedCurve::BuildIntervals
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The continuity intervals of the projected curve: the bounds of the parts, the
//! discontinuities of the curve, and the curve parameters where the projection
//! crosses a discontinuity of the surface in U or V (one order higher).
use super::derivatives::{Pair, d1};
use super::parts::Parts;
use super::{CompState, FUNC_TOL};
use crate::geom_lib::fuse_intervals;
use crate::native_failure::failure;
use crate::proj_lib::{PrjResolve, PrjResolveBox, new_array};
use rustycad_tkernel::ncollection::{Array1, LinearVector};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Point2, Point3, Vector2};

// The continuity asked of the surface for a continuity `S` of the projection.
fn surface_shape(shape: Shape) -> Result<Shape, Failure> {
    match shape {
        Shape::C0 => Ok(Shape::C1),
        Shape::C1 => Ok(Shape::C2),
        Shape::C2 => Ok(Shape::C3),
        Shape::C3 | Shape::CN => Ok(Shape::CN),
        _ => Err(failure(
            "Standard_OutOfRange",
            &format!(
                "ProjLib_CompProjectedCurve::BuildIntervals: continuity {shape:?}; expected C0, C1, C2, C3 or CN"
            ),
        )),
    }
}

fn array_of(values: &[f64]) -> Result<Array1<'static, f64>, Failure> {
    let mut array: Array1<'static, f64> = new_array(values.len() as i32)?;
    for (index, value) in values.iter().enumerate() {
        array[index as i32 + 1] = *value;
    }
    Ok(array)
}

/// A discontinuity direction: U (`fix` 2) or V (`fix` 3).
#[derive(Clone, Copy)]
struct Direction {
    fix: i32,
}

impl Direction {
    fn coord(self, p: &Point3) -> f64 {
        if self.fix == 2 { p.y() } else { p.z() }
    }
    fn other(self, p: &Point3) -> f64 {
        if self.fix == 2 { p.z() } else { p.y() }
    }
}

// The parameters where the parts cross the cut values `cuts` (indices 2..n) of
// one surface direction, duplicates within PConfusion removed.
fn discontinuities(
    state: &CompState,
    pair: Pair<'_>,
    parts: &[Vec<Point3>],
    cuts: &[f64],
    dir: Direction,
) -> Result<Vec<f64>, Failure> {
    let mut found: Vec<f64> = Vec::new();
    for cut in cuts.iter().skip(1) {
        for points in parts {
            for pair_of in points.windows(2) {
                let (left, right) = (&pair_of[0], &pair_of[1]);
                let (l, r) = (dir.coord(left), dir.coord(right));
                let tol: f64 = if dir.fix == 2 {
                    state.tol_u
                } else {
                    state.tol_v
                };
                if (l - cut).abs() <= tol {
                    found.push(left.x());
                } else if (r - cut).abs() <= tol {
                    found.push(right.x());
                } else if ((l < *cut && *cut < r) || (r < *cut && *cut < l))
                    && let Some(t) = crossing(state, pair, (left, right), *cut, dir)?
                {
                    found.push(t);
                }
            }
        }
    }
    let mut i: usize = 1;
    while i < found.len() {
        if found[i] - found[i - 1] < precision::p_confusion() {
            found.remove(i);
        } else {
            i += 1;
        }
    }
    Ok(found)
}

// The curve parameter where the projection crosses `cut` between two triples.
fn crossing(
    state: &CompState,
    pair: Pair<'_>,
    (left, right): (&Point3, &Point3),
    cut: f64,
    dir: Direction,
) -> Result<Option<f64>, Failure> {
    let other: f64 = (dir.other(left) + dir.other(right)) / 2.;
    let mut solver = PrjResolve::new(pair.curve, pair.surface, dir.fix)?;
    let d: Vector2 = d1(left.x(), left.y(), left.z(), pair)?;
    let (tl, tr) = (left.x(), right.x());
    let s = pair.surface;
    let (tol, inf, sup, u, v) = if dir.fix == 2 {
        let tol: f64 = if d.x().abs() < precision::confusion() {
            state.tol_u
        } else {
            state.tol_u.min(state.tol_u / d.x().abs())
        };
        (
            tol,
            s.first_v_parameter()?,
            s.last_v_parameter()?,
            cut,
            other,
        )
    } else {
        let tol: f64 = if d.y().abs() < precision::confusion() {
            state.tol_v
        } else {
            state.tol_v.min(state.tol_v / d.y().abs())
        };
        (
            tol,
            s.first_u_parameter()?,
            s.last_u_parameter()?,
            other,
            cut,
        )
    };
    // Both directions pass (Tol, TolV) as the tolerances, as natively.
    let bounds = PrjResolveBox {
        tol: Point2::new(tol, state.tol_v),
        inf: Point2::new(tl, inf),
        sup: Point2::new(tr, sup),
    };
    solver.perform((tl + tr) / 2., u, v, bounds, FUNC_TOL, false)?;
    if !solver.is_done() {
        return Ok(None);
    }
    Ok(Some(solver.solution()?.x()))
}

/// `BuildIntervals(S)`: the sorted interval bounds of continuity `shape`.
pub(super) fn build_intervals(state: &CompState, shape: Shape) -> Result<Vec<f64>, Failure> {
    let sfor_s: Shape = surface_shape(shape)?;
    let pair: Pair<'_> = state.pair()?;
    let nb_int_cur: i32 = pair.curve.nb_intervals(shape)?;
    let nb_int_sur_u: i32 = pair.surface.nb_u_intervals(sfor_s)?;
    let nb_int_sur_v: i32 = pair.surface.nb_v_intervals(sfor_s)?;
    let mut cut_t: Array1<'static, f64> = new_array(nb_int_cur + 1)?;
    let mut cut_u: Array1<'static, f64> = new_array(nb_int_sur_u + 1)?;
    let mut cut_v: Array1<'static, f64> = new_array(nb_int_sur_v + 1)?;
    pair.curve.intervals(&mut cut_t, shape)?;
    pair.surface.u_intervals(&mut cut_u, sfor_s)?;
    pair.surface.v_intervals(&mut cut_v, sfor_s)?;
    let parts: Vec<Vec<Point3>> = state.parts()?.all_points()?;
    let mut b_arr: Vec<f64> = Vec::with_capacity(2 * parts.len());
    for points in parts.iter().take(state.nb_curves as usize) {
        b_arr.push(points[0].x());
        b_arr.push(points[points.len() - 1].x());
    }
    let c_arr: Vec<f64> = cut_t
        .as_slice()
        .iter()
        .skip(1)
        .take((nb_int_cur - 1).max(0) as usize)
        .copied()
        .collect();
    let u_disc: Vec<f64> = discontinuities(
        state,
        pair,
        &parts,
        &cut_u.as_slice()[..nb_int_sur_u as usize],
        Direction { fix: 2 },
    )?;
    let v_disc: Vec<f64> = discontinuities(
        state,
        pair,
        &parts,
        &cut_v.as_slice()[..nb_int_sur_v as usize],
        Direction { fix: 3 },
    )?;
    for extra in [c_arr, u_disc, v_disc] {
        if extra.is_empty() {
            continue;
        }
        let mut fusion: LinearVector<f64> = LinearVector::default();
        fuse_intervals(
            &array_of(&b_arr)?,
            &array_of(&extra)?,
            &mut fusion,
            precision::p_confusion(),
            false,
        );
        b_arr = fusion.as_slice().to_vec();
    }
    Ok(b_arr)
}
