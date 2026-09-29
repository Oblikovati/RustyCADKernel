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
// Rust translation of ModelingData/TKG3d/GeomAdaptor/GeomAdaptor_Curve.cxx (LocalContinuity, Continuity, NbIntervals, Intervals) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Continuity and continuity intervals of `GeomAdaptor_Curve`.
//!
//! A B-spline is `C(Degree - m)` on the adaptor range, `m` the highest interior
//! multiplicity; its intervals come from BSplCLib::Intervals. An offset curve has
//! the interval bounds of its basis, one continuity order higher, that lie strictly
//! inside the adaptor range. Every other curve is one interval.
use super::curve::GeomAdaptorCurve;
use super::curve_query::resolution;
use super::curve_state::{AdaptorState, BSplineData, CurveData};
use crate::adaptor3d::Adaptor3dCurve;
use crate::geom::errors::{array_bounds, no_such_object};
use crate::geom::{GeomBSplineCurve, GeomCurve, GeomOffsetCurve};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d_adaptor::knot_continuity::{
    self, KnotSpans, interval_order, offset_basis_shape,
};
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::geom_abs::Shape;

/// `Continuity`; Standard_NoSuchObject for an unknown curve kind or a C0 offset basis;
/// `continuity(&state)?`.
pub(super) fn continuity(state: &AdaptorState) -> Result<Shape, Failure> {
    match &state.data {
        CurveData::BSpline(data) => bspline_continuity(&data.curve, state.first, state.last),
        CurveData::Offset(_) => offset_continuity(state),
        // The native message misspells "Contunuity".
        CurveData::None => Err(no_such_object("GeomAdaptor_Curve::Contunuity")),
        _ => Ok(Shape::CN),
    }
}

// An offset loses one order: C1 becomes C0; G1 and G2 stay, C0 has no continuity.
fn offset_continuity(state: &AdaptorState) -> Result<Shape, Failure> {
    let offset_curve: Handle<GeomOffsetCurve> = state.curve.cast();
    knot_continuity::offset_continuity(offset_curve.basis_curve_continuity(), "GeomAdaptor_Curve")
}

/// `LocalContinuity` over the knots of `curve`; `bspline_continuity(&curve, 0.0, 1.0)?`.
fn bspline_continuity(curve: &GeomBSplineCurve, u1: f64, u2: f64) -> Result<Shape, Failure> {
    let (knots, mults) = (curve.knots(), curve.multiplicities());
    let spans = KnotSpans {
        knots: &knots,
        mults: &mults,
        degree: curve.degree(),
        periodic: curve.is_periodic(),
    };
    knot_continuity::local_continuity(&spans, u1, u2)
}

// Whether a B-spline answers one interval: non-periodic and already `shape` over the
// range, or `shape` is C0.
fn bspline_single(state: &AdaptorState, data: &BSplineData, shape: Shape) -> Result<bool, Failure> {
    Ok((!data.curve.is_periodic() && shape <= continuity(state)?) || shape == Shape::C0)
}

// The BSplCLib::Intervals request of a B-spline: continuity order and knot tolerance.
fn bspline_order(
    state: &AdaptorState,
    data: &BSplineData,
    shape: Shape,
    member: &str,
) -> Result<(i32, f64), Failure> {
    let order: i32 = interval_order(shape, data.curve.degree(), "GeomAdaptor_Curve", member)?;
    let tolerance: f64 = resolution(state, precision::confusion())?.min(precision::p_confusion());
    Ok((order, tolerance))
}

fn bspline_intervals(
    state: &AdaptorState,
    data: &BSplineData,
    shape: Shape,
    out: Option<&mut Array1<'_, f64>>,
    member: &str,
) -> Result<i32, Failure> {
    let (order, tolerance) = bspline_order(state, data, shape, member)?;
    let curve: &GeomBSplineCurve = &data.curve;
    let (knots, mults) = (curve.knots(), curve.multiplicities());
    bsplclib::intervals(
        &knots,
        &mults,
        curve.degree(),
        curve.is_periodic(),
        order,
        state.first,
        state.last,
        tolerance,
        out,
    )
}

// The interval bounds of the offset basis (a temporary adaptor over the same range,
// one continuity order higher) strictly inside the range: akm 05/04/02 (OCC278)
// recounts the basis intervals so that a trimmed offset reflects its bounds.
fn offset_interior_breaks(state: &AdaptorState, shape: Shape) -> Result<Vec<f64>, Failure> {
    let basis_shape: Shape = offset_basis_shape(shape)?;
    let basis: Handle<dyn GeomCurve> = state.curve.cast::<GeomOffsetCurve>().basis_curve();
    let adaptor = GeomAdaptorCurve::with_range(&basis, state.first, state.last)?;
    let count: i32 = adaptor.nb_intervals(basis_shape)?;
    if count <= 1 {
        return Ok(Vec::new());
    }
    let mut bounds: Array1<'static, f64> =
        Array1::with_bounds(1, 1 + count).map_err(array_bounds)?;
    adaptor.intervals(&mut bounds, basis_shape)?;
    let inside = |value: &f64| *value > state.first && *value < state.last;
    Ok(bounds.as_slice()[..count as usize]
        .iter()
        .copied()
        .filter(inside)
        .collect())
}

/// `NbIntervals`; Standard_DomainError for G1/G2 on a B-spline or an offset;
/// `nb_intervals(&state, Shape::C2)?`.
pub(super) fn nb_intervals(state: &AdaptorState, shape: Shape) -> Result<i32, Failure> {
    match &state.data {
        CurveData::BSpline(data) => {
            if bspline_single(state, data, shape)? {
                return Ok(1);
            }
            bspline_intervals(state, data, shape, None, "NbIntervals")
        }
        CurveData::Offset(_) => Ok(offset_interior_breaks(state, shape)?.len() as i32 + 1),
        _ => Ok(1),
    }
}

/// `Intervals`: `out` gets `NbIntervals + 1` bounds from its lower index, the first
/// and last being the adaptor bounds; `intervals(&state, &mut out, Shape::C2)?`.
pub(super) fn intervals(
    state: &AdaptorState,
    out: &mut Array1<'_, f64>,
    shape: Shape,
) -> Result<(), Failure> {
    match &state.data {
        CurveData::BSpline(data) if !bspline_single(state, data, shape)? => {
            bspline_intervals(state, data, shape, Some(out), "Intervals").map(|_| ())
        }
        CurveData::Offset(_) => {
            let breaks: Vec<f64> = offset_interior_breaks(state, shape)?;
            // Natively `T(++myNbIntervals)`: absolute indices from 2, whatever T.Lower().
            for (index, value) in (2..).zip(breaks.iter()) {
                out.set_value(index, *value).map_err(array_bounds)?;
            }
            let lower: i32 = out.lower();
            out.set_value(lower, state.first).map_err(array_bounds)?;
            out.set_value(lower + breaks.len() as i32 + 1, state.last)
                .map_err(array_bounds)
        }
        _ => {
            let lower: i32 = out.lower();
            out.set_value(lower, state.first).map_err(array_bounds)?;
            out.set_value(lower + 1, state.last).map_err(array_bounds)
        }
    }
}
