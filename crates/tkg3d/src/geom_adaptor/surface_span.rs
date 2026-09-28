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
// Rust translation of ModelingData/TKG3d/GeomAdaptor/GeomAdaptor_Surface.cxx (IfUVBound, Span and the boundary snapping of EvalD1..EvalDN) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The boundary handling of `GeomAdaptor_Surface`: a parameter within the adaptor
//! tolerance of a bound is snapped to it, and a B-spline is then evaluated on the
//! knot span that the bound closes (`IfUVBound`, `Span`).
use super::surface_state::SurfaceState;
use crate::geom::{GeomBSplineSurface, LocalKnots};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;

/// `PosTol` of GeomAdaptor_Surface.cxx: half the parametric confusion.
const POS_TOL: f64 = precision::p_confusion() * 0.5;

/// The snapped parameters of one evaluation: `(u, v)` and the sides (1 at the first
/// bound, -1 at the last, 0 inside).
#[derive(Clone, Copy, Debug)]
pub(super) struct Snapped {
    pub(super) u: f64,
    pub(super) v: f64,
    pub(super) u_side: i32,
    pub(super) v_side: i32,
}
impl Snapped {
    /// No snapping (EvalD0); `Snapped::raw(u, v)`.
    pub(super) fn raw(u: f64, v: f64) -> Self {
        Self {
            u,
            v,
            u_side: 0,
            v_side: 0,
        }
    }
    /// Whether a parameter was snapped to a bound; `snapped.on_bound()`.
    pub(super) fn on_bound(&self) -> bool {
        self.u_side != 0 || self.v_side != 0
    }
}

/// The snapping at the head of EvalD1..EvalDN: `|U - UFirst| <= TolU` first, then
/// `|U - ULast| <= TolU`, likewise in V; `snap(&state, u, v)`.
pub(super) fn snap(state: &SurfaceState, u: f64, v: f64) -> Snapped {
    let (u_side, snapped_u) = side(u, (state.u_first, state.u_last), state.tol_u);
    let (v_side, snapped_v) = side(v, (state.v_first, state.v_last), state.tol_v);
    Snapped {
        u: snapped_u,
        v: snapped_v,
        u_side,
        v_side,
    }
}

fn side(value: f64, (first, last): (f64, f64), tolerance: f64) -> (i32, f64) {
    if (value - first).abs() <= tolerance {
        return (1, first);
    }
    if (value - last).abs() <= tolerance {
        return (-1, last);
    }
    (0, value)
}

/// `IfUVBound`: the local knot spans of the snapped `(u, v)` and whether either lies
/// on a knot; `if_uv_bound(&bspline, snapped)?`.
pub(super) fn if_uv_bound(
    bspline: &GeomBSplineSurface,
    snapped: Snapped,
) -> Result<(bool, LocalKnots), Failure> {
    let (u_first_index, u_last_index) =
        (bspline.first_u_knot_index()?, bspline.last_u_knot_index()?);
    let (v_first_index, v_last_index) =
        (bspline.first_v_knot_index()?, bspline.last_v_knot_index()?);
    let (ideb, ifin) = bspline.locate_u(snapped.u, POS_TOL, false)?;
    let mut local: bool = ideb == ifin;
    let u_span = span(snapped.u_side, (ideb, ifin), (u_first_index, u_last_index));
    let (ivdeb, ivfin) = bspline.locate_v(snapped.v, POS_TOL, false)?;
    if ivdeb == ivfin {
        local = true;
    }
    let v_span = span(
        snapped.v_side,
        (ivdeb, ivfin),
        (v_first_index, v_last_index),
    );
    Ok((local, (u_span, v_span)))
}

/// `Span`: the knot interval given to LocalD*, from the located `(Ideb, Ifin)` and the
/// first and last knot indices; `span(1, (2, 3), (1, 5))`.
pub(super) fn span(side: i32, located: (i32, i32), range: (i32, i32)) -> (i32, i32) {
    let ((ideb, ifin), (first, last)) = (located, range);
    if ideb != ifin {
        return span_between_knots((ideb, ifin), (first, last));
    }
    if ideb <= first {
        return (first, first + 1);
    }
    if ifin >= last {
        return (last - 1, last);
    }
    if side == -1 {
        (ideb - 1, ifin)
    } else {
        (ideb, ifin + 1)
    }
}

// The "not a knot" branch of Span, in the native test order.
fn span_between_knots((ideb, ifin): (i32, i32), (first, last): (i32, i32)) -> (i32, i32) {
    if ideb < first {
        return (first, first + 1);
    }
    if ifin > last || ideb >= last - 1 {
        return (last - 1, last);
    }
    if ifin <= first + 1 {
        return (first, first + 1);
    }
    if ideb > ifin {
        return (ifin - 1, ifin);
    }
    (ideb, ifin)
}
