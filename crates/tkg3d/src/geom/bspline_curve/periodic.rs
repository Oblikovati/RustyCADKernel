// Copyright (c) 1993-1999 Matra Datavision
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
// Rust translation of ModelingData/TKG3d/Geom/Geom_BSplineCurve.cxx (SetOrigin(U, Tol)) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `SetOrigin(U, Tol)`, which only Geom_BSplineCurve declares.
use crate::geom::errors;
use crate::geom::spline_kind::BSplineState;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::arrays::{at, one_based, put};

/// `SetOrigin(U, Tol)`: shift the knots by whole periods so that `u` lies in the
/// range, then make the knot at `u` (inserted when none is within `tol`) the origin.
///
/// Like the native loops, an infinite `u` or period never terminates; callers pass
/// finite values.
pub(super) fn set_origin_parameter(
    state: &mut BSplineState,
    u: f64,
    tol: f64,
) -> Result<(), Failure> {
    if !state.periodic {
        return Err(errors::no_such_object(
            "Geom_BSplineCurve::SetOrigin: the curve is not periodic",
        ));
    }
    let (uf, ul) = shift_range_to(state, u, tol)?;
    if (u - uf).abs() < tol || (u - ul).abs() < tol {
        return Ok(());
    }
    let (mut ik, delta) = nearest_knot(state, u)?;
    if delta.abs() > tol {
        insert_knot_default(state, u)?;
        if delta < 0. {
            ik += 1;
        }
    }
    state.set_origin(ik)
}

// The reparametrization step of SetOrigin(U, Tol): returns the shifted (uf, ul).
fn shift_range_to(state: &mut BSplineState, u: f64, tol: f64) -> Result<(f64, f64), Failure> {
    let (mut uf, mut ul) = (state.first_parameter(), state.last_parameter());
    let (mut shifted, period) = (u, ul - uf);
    while tol < (uf - shifted) {
        shifted += period;
    }
    while tol > (ul - shifted) {
        shifted -= period;
    }
    if (u - shifted).abs() > tol {
        state.clear_eval_rep_on_edit();
        let delta: f64 = u - shifted;
        uf += delta;
        ul += delta;
        for knot in state.knots.as_mut_slice() {
            *knot += delta;
        }
        state.update_knots()?;
    }
    Ok((uf, ul))
}

// The first knot closest to `u` and its signed distance (`delta = RealLast()` start).
fn nearest_knot(state: &BSplineState, u: f64) -> Result<(i32, f64), Failure> {
    let (mut ik, mut delta): (i32, f64) = (0, f64::MAX);
    for i in state.knots.lower()..=state.knots.upper() {
        let dki: f64 = at(&state.knots, i)? - u;
        if dki.abs() < delta.abs() {
            ik = i;
            delta = dki;
        }
    }
    Ok((ik, delta))
}

// `InsertKnot(U)` with its default arguments (M = 1, tolerance 0, Add = true).
fn insert_knot_default(state: &mut BSplineState, u: f64) -> Result<(), Failure> {
    let mut knots: Array1<'static, f64> = one_based(1)?;
    put(&mut knots, 1, u)?;
    let mut mults: Array1<'static, i32> = one_based(1)?;
    put(&mut mults, 1, 1)?;
    state.insert_knots(&knots, &mults, 0.0, true)
}
