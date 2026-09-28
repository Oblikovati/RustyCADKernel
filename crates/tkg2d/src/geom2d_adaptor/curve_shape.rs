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
// Rust translation of ModelingData/TKG2d/Geom2dAdaptor/Geom2dAdaptor_Curve.cxx (LocalContinuity, Continuity, NbIntervals, Intervals) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Continuity and continuity intervals of `Geom2dAdaptor_Curve`.
//!
//! A B-spline is `C(Degree - m)` on the adaptor range, `m` the highest interior
//! multiplicity; its intervals come from BSplCLib::Intervals. An offset curve has
//! the intervals of its basis one continuity order higher. Every other curve is
//! one interval.
use super::curve::Geom2dAdaptorCurve;
use super::curve_query::resolution;
use super::curve_state::{AdaptorState, BSplineData, CurveData};
use crate::adaptor2d::Adaptor2dCurve2d;
use crate::geom2d::errors::{array_bounds, domain, no_such_object};
use crate::geom2d::{Geom2dBSplineCurve, Geom2dCurve, Geom2dOffsetCurve};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::geom_abs::Shape;

/// `Continuity`; Standard_NoSuchObject for an unknown curve kind or a C0 offset basis;
/// `continuity(&state)?`.
pub(super) fn continuity(state: &AdaptorState) -> Result<Shape, Failure> {
    match &state.data {
        CurveData::BSpline(data) => local_continuity(&data.curve, state.first, state.last),
        CurveData::Offset(_) => offset_continuity(state),
        CurveData::None => Err(no_such_object("Geom2dAdaptor_Curve::Continuity")),
        _ => Ok(Shape::CN),
    }
}

// An offset loses one order: C1 becomes C0; G1 and G2 stay, C0 has no continuity.
fn offset_continuity(state: &AdaptorState) -> Result<Shape, Failure> {
    let offset_curve: Handle<Geom2dOffsetCurve> = state.curve.cast();
    Ok(match offset_curve.basis_curve_continuity() {
        Shape::CN => Shape::CN,
        Shape::C3 => Shape::C2,
        Shape::C2 => Shape::C1,
        Shape::C1 => Shape::C0,
        Shape::G1 => Shape::G1,
        Shape::G2 => Shape::G2,
        Shape::C0 => return Err(no_such_object("Geom2dAdaptor_Curve::Continuity")),
    })
}

// The knot index of `u` and the parameter brought into the knot range (LocateParameter).
fn locate(
    curve: &Geom2dBSplineCurve,
    knots: &Array1<'_, f64>,
    mults: &Array1<'_, i32>,
    u: f64,
) -> Result<(i32, f64), Failure> {
    let (mut index, mut new_u): (i32, f64) = (0, 0.0);
    let count: i32 = curve.nb_knots();
    bsplclib::locate_parameter(
        curve.degree(),
        knots,
        mults,
        u,
        curve.is_periodic(),
        1,
        count,
        &mut index,
        &mut new_u,
    )?;
    Ok((index, new_u))
}

/// `LocalContinuity`: `C(Degree - m)` with `m` the highest multiplicity of the knots
/// strictly inside `[u1, u2]`, CN between two consecutive knots of a non-periodic
/// curve; `local_continuity(&curve, 0.0, 1.0)?`.
pub(super) fn local_continuity(
    curve: &Geom2dBSplineCurve,
    u1: f64,
    u2: f64,
) -> Result<Shape, Failure> {
    let (knots, mults) = (curve.knots(), curve.multiplicities());
    let count: i32 = curve.nb_knots();
    let (mut index1, new_first) = locate(curve, &knots, &mults, u1)?;
    let (mut index2, new_last) = locate(curve, &knots, &mults, u2)?;
    if (new_first - *knots.value(index1 + 1).map_err(array_bounds)?).abs()
        < precision::p_confusion()
        && index1 < count
    {
        index1 += 1;
    }
    if (new_last - *knots.value(index2).map_err(array_bounds)?).abs() < precision::p_confusion() {
        index2 -= 1;
    }
    // Beware of periodic curves.
    if curve.is_periodic() && index1 == count {
        index1 = 1;
    }
    let mult_max: i32 = if index2 - index1 <= 0 && !curve.is_periodic() {
        100 // CN between two consecutive knots.
    } else {
        curve.degree() - max_multiplicity(&mults, index1, index2)?
    };
    Ok(match mult_max {
        i32::MIN..=0 => Shape::C0,
        1 => Shape::C1,
        2 => Shape::C2,
        3 => Shape::C3,
        _ => Shape::CN,
    })
}

// The native loop: TM(Index1 + 1), then TM(Index1 + 1..=Index2).
fn max_multiplicity(mults: &Array1<'_, i32>, index1: i32, index2: i32) -> Result<i32, Failure> {
    let mut max: i32 = *mults.value(index1 + 1).map_err(array_bounds)?;
    for i in index1 + 1..=index2 {
        max = max.max(*mults.value(i).map_err(array_bounds)?);
    }
    Ok(max)
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
    let order: i32 = match shape {
        Shape::C1 => 1,
        Shape::C2 => 2,
        Shape::C3 => 3,
        Shape::CN => data.curve.degree(),
        _ => return Err(domain(&format!("Geom2dAdaptor_Curve::{member}()"))),
    };
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
    let curve: &Geom2dBSplineCurve = &data.curve;
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

// The basis continuity of an offset's intervals; G1 and G2 have none (the native
// message names GeomAdaptor_Curve).
fn offset_basis_shape(shape: Shape) -> Result<Shape, Failure> {
    Ok(match shape {
        Shape::G1 | Shape::G2 => return Err(domain("GeomAdaptor_Curve::NbIntervals")),
        Shape::C0 => Shape::C1,
        Shape::C1 => Shape::C2,
        Shape::C2 => Shape::C3,
        Shape::C3 | Shape::CN => Shape::CN,
    })
}

// A temporary adaptor on the offset basis over the same range.
fn offset_basis_adaptor(state: &AdaptorState) -> Result<Geom2dAdaptorCurve, Failure> {
    let basis: Handle<dyn Geom2dCurve> = state.curve.cast::<Geom2dOffsetCurve>().basis_curve();
    Geom2dAdaptorCurve::with_range(&basis, state.first, state.last)
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
        CurveData::Offset(_) => {
            let basis_shape: Shape = offset_basis_shape(shape)?;
            offset_basis_adaptor(state)?.nb_intervals(basis_shape)
        }
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
            let basis_shape: Shape = offset_basis_shape(shape)?;
            let adaptor: Geom2dAdaptorCurve = offset_basis_adaptor(state)?;
            let count: i32 = adaptor.nb_intervals(basis_shape)?;
            adaptor.intervals(out, basis_shape)?;
            let lower: i32 = out.lower();
            out.set_value(lower, state.first).map_err(array_bounds)?;
            out.set_value(lower + count, state.last)
                .map_err(array_bounds)
        }
        _ => {
            let lower: i32 = out.lower();
            out.set_value(lower, state.first).map_err(array_bounds)?;
            out.set_value(lower + 1, state.last).map_err(array_bounds)
        }
    }
}
