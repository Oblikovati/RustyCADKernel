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
// Rust translation of the result assembly of EvalD1..EvalDN in ModelingData/TKG3d/GeomEval/GeomEval_TBezierSurface.cxx and GeomEval_AHTBezierSurface.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The Bezier eval surfaces compute the point and the derivatives of one evaluator
//! order as ten coordinates in `ORDER_PAIRS` order; this module turns them into the
//! Geom_Surface result structures and picks the low-order EvalDN component.
use super::bezier_grid::ORDER_PAIRS;
use crate::geom::{GeomSurfaceD1, GeomSurfaceD2, GeomSurfaceD3, errors};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Point3, Vector3, Xyz};

/// S, DU, DV, D2U, D2V, D2UV, D3U, D3V, D3UUV, D3UVV; unused orders stay zero.
pub(crate) type SurfaceDerivs = [Xyz; 10];

/// `Geom_Surface::ResD1` from the first three values; `to_d1(values)`.
pub(crate) fn to_d1(values: SurfaceDerivs) -> GeomSurfaceD1 {
    let [s, d1u, d1v, ..] = values;
    GeomSurfaceD1 {
        point: Point3::from(s),
        d1u: Vector3::from(d1u),
        d1v: Vector3::from(d1v),
    }
}

/// `Geom_Surface::ResD2` from the first six values; `to_d2(values)`.
pub(crate) fn to_d2(values: SurfaceDerivs) -> GeomSurfaceD2 {
    let [s, d @ ..] = values;
    let [d1u, d1v, d2u, d2v, d2uv, ..] = d.map(Vector3::from);
    GeomSurfaceD2 {
        point: Point3::from(s),
        d1u,
        d1v,
        d2u,
        d2v,
        d2uv,
    }
}

/// `Geom_Surface::ResD3` from the ten values; `to_d3(values)`.
pub(crate) fn to_d3(values: SurfaceDerivs) -> GeomSurfaceD3 {
    let [s, d @ ..] = values;
    let [d1u, d1v, d2u, d2v, d2uv, d3u, d3v, d3uuv, d3uvv] = d.map(Vector3::from);
    GeomSurfaceD3 {
        point: Point3::from(s),
        d1u,
        d1v,
        d2u,
        d2v,
        d2uv,
        d3u,
        d3v,
        d3uuv,
        d3uvv,
    }
}

/// The native `Nu + Nv < 1 || Nu < 0 || Nv < 0` check of both EvalDN members
/// (Standard_RangeError); `check_dn_range(CLASS, nu, nv)?`.
pub(crate) fn check_dn_range(class: &str, nu: i32, nv: i32) -> Result<(), Failure> {
    if nu + nv < 1 || nu < 0 || nv < 0 {
        return Err(errors::range(&format!(
            "{class}::EvalDN: Nu = {nu}, Nv = {nv}; expected Nu >= 0, Nv >= 0 and Nu + Nv >= 1"
        )));
    }
    Ok(())
}

/// The position in `SurfaceDerivs` of the `(nu, nv)` derivative when `nu + nv <= 3`,
/// the orders that EvalDN takes from EvalD1..EvalD3; `low_order_index(2, 1) == Some(8)`.
pub(crate) fn low_order_index(nu: i32, nv: i32) -> Option<usize> {
    let pair: (usize, usize) = (usize::try_from(nu).ok()?, usize::try_from(nv).ok()?);
    ORDER_PAIRS
        .iter()
        .skip(1)
        .position(|&p| p == pair)
        .map(|k| k + 1)
}
