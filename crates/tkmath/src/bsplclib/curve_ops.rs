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
// Rust translation of FoundationClasses/TKMath/BSplCLib/BSplCLib_CurveComputation.pxx (RemoveKnot, InsertKnots, InsertKnot, RaiseMultiplicity, IncreaseDegree, Unperiodize, Trimming and the Bezier IncreaseDegree) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Knot and degree operations on 2D/3D (rational) curves.
//!
//! Each operation packs the poles (homogeneous `[w*x.., w]` when weighted) into
//! reals, runs the multi-dimensional algorithm and unpacks the result.
use super::degree::increase_degree_flat;
use super::insert::insert_knots_flat;
use super::native::epsilon;
use super::periodic::{trimming_flat, unperiodize_flat};
use super::remove::remove_knot_flat;
use super::views::one_based;
use crate::plib::access::array_value;
use crate::plib::{PolePoint, get_poles, get_rational_poles, set_poles, set_rational_poles};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;

const CURVE: &str = "BSplCLib curve operation";

// Poles packed as reals, with the working dimension.
fn pack<P: PolePoint>(
    poles: &Array1<'_, P>,
    weights: Option<&Array1<'_, f64>>,
) -> Result<(Array1<'static, f64>, i32), Failure> {
    let dim: i32 = P::DIMENSION as i32 + i32::from(weights.is_some());
    let mut flat: Array1<'static, f64> = one_based(CURVE, dim * poles.len() as i32)?;
    match weights {
        Some(weights) => set_rational_poles(poles, weights, &mut flat)?,
        None => set_poles(poles, &mut flat)?,
    }
    Ok((flat, dim))
}

// Unpacks into the new poles; a rational input needs output weights.
fn unpack<P: PolePoint>(
    flat: &Array1<'_, f64>,
    rational: bool,
    new_poles: &mut Array1<'_, P>,
    new_weights: Option<&mut Array1<'_, f64>>,
) -> Result<(), Failure> {
    if !rational {
        return get_poles(flat, new_poles);
    }
    let new_weights: &mut Array1<'_, f64> = new_weights.ok_or_else(|| {
        Failure::out_of_range("BSplCLib: rational poles need NewWeights; received None")
    })?;
    get_rational_poles(flat, new_poles, new_weights)
}

fn output_flat(dim: i32, count: usize) -> Result<Array1<'static, f64>, Failure> {
    one_based(CURVE, dim * count as i32)
}

/// Removes or lowers the multiplicity of a knot of a 2D/3D curve (`BSplCLib::RemoveKnot`
/// on gp_Pnt/gp_Pnt2d); on false the new arrays are untouched;
/// `bsplclib::remove_knot(2, 0, 3, false, &p, None, &k, &m, &mut np, None, &mut nk, &mut nm, 1e-7)?`.
pub fn remove_knot<P: PolePoint>(
    index: i32,
    mult: i32,
    degree: i32,
    periodic: bool,
    poles: &Array1<'_, P>,
    weights: Option<&Array1<'_, f64>>,
    knots: &Array1<'_, f64>,
    mults: &Array1<'_, i32>,
    new_poles: &mut Array1<'_, P>,
    new_weights: Option<&mut Array1<'_, f64>>,
    new_knots: &mut Array1<'_, f64>,
    new_mults: &mut Array1<'_, i32>,
    tolerance: f64,
) -> Result<bool, Failure> {
    let (flat, dim) = pack(poles, weights)?;
    let mut new_flat: Array1<'static, f64> = output_flat(dim, new_poles.len())?;
    let removed: bool = remove_knot_flat(
        index,
        mult,
        degree,
        periodic,
        dim,
        &flat,
        knots,
        mults,
        &mut new_flat,
        new_knots,
        new_mults,
        tolerance,
    )?;
    if !removed {
        return Ok(false);
    }
    unpack(&new_flat, weights.is_some(), new_poles, new_weights)?;
    Ok(true)
}

/// Inserts knots into a 2D/3D curve (`BSplCLib::InsertKnots` on gp_Pnt/gp_Pnt2d);
/// `bsplclib::insert_knots(3, false, &p, None, &k, &m, &add, None, &mut np, None, &mut nk, &mut nm, 0.0, true)?`.
pub fn insert_knots<P: PolePoint>(
    degree: i32,
    periodic: bool,
    poles: &Array1<'_, P>,
    weights: Option<&Array1<'_, f64>>,
    knots: &Array1<'_, f64>,
    mults: &Array1<'_, i32>,
    add_knots: &Array1<'_, f64>,
    add_mults: Option<&Array1<'_, i32>>,
    new_poles: &mut Array1<'_, P>,
    new_weights: Option<&mut Array1<'_, f64>>,
    new_knots: &mut Array1<'_, f64>,
    new_mults: &mut Array1<'_, i32>,
    tolerance: f64,
    add: bool,
) -> Result<(), Failure> {
    let (flat, dim) = pack(poles, weights)?;
    let mut new_flat: Array1<'static, f64> = output_flat(dim, new_poles.len())?;
    insert_knots_flat(
        degree,
        periodic,
        dim,
        &flat,
        knots,
        mults,
        add_knots,
        add_mults,
        &mut new_flat,
        new_knots,
        new_mults,
        tolerance,
        add,
    )?;
    unpack(&new_flat, weights.is_some(), new_poles, new_weights)
}

// One knot of multiplicity `mult` inserted with the native temporary knot arrays.
fn insert_single<P: PolePoint>(
    knot: f64,
    mult: i32,
    new_knot_count: usize,
    degree: i32,
    periodic: bool,
    poles: &Array1<'_, P>,
    weights: Option<&Array1<'_, f64>>,
    knots: &Array1<'_, f64>,
    mults: &Array1<'_, i32>,
    new_poles: &mut Array1<'_, P>,
    new_weights: Option<&mut Array1<'_, f64>>,
) -> Result<(), Failure> {
    let mut add_knot: Array1<'static, f64> = one_based(CURVE, 1)?;
    add_knot[1] = knot;
    let mut add_mult: Array1<'static, i32> = one_based(CURVE, 1)?;
    add_mult[1] = mult;
    let mut new_knots: Array1<'static, f64> = one_based(CURVE, new_knot_count as i32)?;
    let mut new_mults: Array1<'static, i32> = one_based(CURVE, new_knot_count as i32)?;
    insert_knots(
        degree,
        periodic,
        poles,
        weights,
        knots,
        mults,
        &add_knot,
        Some(&add_mult),
        new_poles,
        new_weights,
        &mut new_knots,
        &mut new_mults,
        epsilon(knot),
        true,
    )
}

/// Inserts one knot `u` of multiplicity `u_mult`; only the new poles are returned,
/// and the UIndex argument is ignored natively (`BSplCLib::InsertKnot`);
/// `bsplclib::insert_knot(0, 0.5, 1, 3, false, &p, None, &k, &m, &mut np, None)?`.
pub fn insert_knot<P: PolePoint>(
    _u_index: i32,
    u: f64,
    u_mult: i32,
    degree: i32,
    periodic: bool,
    poles: &Array1<'_, P>,
    weights: Option<&Array1<'_, f64>>,
    knots: &Array1<'_, f64>,
    mults: &Array1<'_, i32>,
    new_poles: &mut Array1<'_, P>,
    new_weights: Option<&mut Array1<'_, f64>>,
) -> Result<(), Failure> {
    insert_single(
        u,
        u_mult,
        knots.len() + 1,
        degree,
        periodic,
        poles,
        weights,
        knots,
        mults,
        new_poles,
        new_weights,
    )
}

/// Raises the multiplicity of `Knots(knot_index)` to `mult`; only the new poles are
/// returned (`BSplCLib::RaiseMultiplicity`);
/// `bsplclib::raise_multiplicity(2, 3, 3, false, &p, None, &k, &m, &mut np, None)?`.
pub fn raise_multiplicity<P: PolePoint>(
    knot_index: i32,
    mult: i32,
    degree: i32,
    periodic: bool,
    poles: &Array1<'_, P>,
    weights: Option<&Array1<'_, f64>>,
    knots: &Array1<'_, f64>,
    mults: &Array1<'_, i32>,
    new_poles: &mut Array1<'_, P>,
    new_weights: Option<&mut Array1<'_, f64>>,
) -> Result<(), Failure> {
    let knot: f64 = array_value(knots, knot_index)?;
    let added: i32 = mult - array_value(mults, knot_index)?;
    insert_single(
        knot,
        added,
        knots.len(),
        degree,
        periodic,
        poles,
        weights,
        knots,
        mults,
        new_poles,
        new_weights,
    )
}

/// Elevates the degree of a 2D/3D curve (`BSplCLib::IncreaseDegree` on gp_Pnt/gp_Pnt2d);
/// `bsplclib::increase_degree(2, 3, false, &p, None, &k, &m, &mut np, None, &mut nk, &mut nm)?`.
pub fn increase_degree<P: PolePoint>(
    degree: i32,
    new_degree: i32,
    periodic: bool,
    poles: &Array1<'_, P>,
    weights: Option<&Array1<'_, f64>>,
    knots: &Array1<'_, f64>,
    mults: &Array1<'_, i32>,
    new_poles: &mut Array1<'_, P>,
    new_weights: Option<&mut Array1<'_, f64>>,
    new_knots: &mut Array1<'_, f64>,
    new_mults: &mut Array1<'_, i32>,
) -> Result<(), Failure> {
    let (flat, dim) = pack(poles, weights)?;
    let mut new_flat: Array1<'static, f64> = output_flat(dim, new_poles.len())?;
    increase_degree_flat(
        degree,
        new_degree,
        periodic,
        dim,
        &flat,
        knots,
        mults,
        &mut new_flat,
        new_knots,
        new_mults,
    )?;
    unpack(&new_flat, weights.is_some(), new_poles, new_weights)
}

/// Elevates the degree of a 2D/3D Bezier curve (`BSplCLib::IncreaseDegree(NewDegree,
/// Poles, Weights, NewPoles, NewWeights)`); the native knot arrays are both input
/// and output, which only affects scratch values;
/// `bsplclib::increase_degree_bezier(4, &poles, None, &mut new_poles, None)?`.
pub fn increase_degree_bezier<P: PolePoint>(
    new_degree: i32,
    poles: &Array1<'_, P>,
    weights: Option<&Array1<'_, f64>>,
    new_poles: &mut Array1<'_, P>,
    new_weights: Option<&mut Array1<'_, f64>>,
) -> Result<(), Failure> {
    let degree: i32 = poles.len() as i32 - 1;
    let (knots, mults) = super::bezier_knots(degree)?;
    let (mut new_knots, mut new_mults) = (knots.clone(), mults.clone());
    increase_degree(
        degree,
        new_degree,
        false,
        poles,
        weights,
        &knots,
        &mults,
        new_poles,
        new_weights,
        &mut new_knots,
        &mut new_mults,
    )
}

/// Rewrites a periodic 2D/3D curve as a non-periodic one (`BSplCLib::Unperiodize` on
/// gp_Pnt/gp_Pnt2d); `bsplclib::unperiodize(3, &m, &k, &p, None, &mut nm, &mut nk, &mut np, None)?`.
pub fn unperiodize<P: PolePoint>(
    degree: i32,
    mults: &Array1<'_, i32>,
    knots: &Array1<'_, f64>,
    poles: &Array1<'_, P>,
    weights: Option<&Array1<'_, f64>>,
    new_mults: &mut Array1<'_, i32>,
    new_knots: &mut Array1<'_, f64>,
    new_poles: &mut Array1<'_, P>,
    new_weights: Option<&mut Array1<'_, f64>>,
) -> Result<(), Failure> {
    let (flat, dim) = pack(poles, weights)?;
    let mut new_flat: Array1<'static, f64> = output_flat(dim, new_poles.len())?;
    unperiodize_flat(
        degree,
        dim,
        mults,
        knots,
        &flat,
        new_mults,
        new_knots,
        &mut new_flat,
    )?;
    unpack(&new_flat, weights.is_some(), new_poles, new_weights)
}

/// Trims a 2D/3D curve to `[u1, u2]` (`BSplCLib::Trimming` on gp_Pnt/gp_Pnt2d);
/// `bsplclib::trimming(3, false, &k, &m, &p, None, 0.2, 0.8, &mut nk, &mut nm, &mut np, None)?`.
pub fn trimming<P: PolePoint>(
    degree: i32,
    periodic: bool,
    knots: &Array1<'_, f64>,
    mults: &Array1<'_, i32>,
    poles: &Array1<'_, P>,
    weights: Option<&Array1<'_, f64>>,
    u1: f64,
    u2: f64,
    new_knots: &mut Array1<'_, f64>,
    new_mults: &mut Array1<'_, i32>,
    new_poles: &mut Array1<'_, P>,
    new_weights: Option<&mut Array1<'_, f64>>,
) -> Result<(), Failure> {
    let (flat, dim) = pack(poles, weights)?;
    let mut new_flat: Array1<'static, f64> = output_flat(dim, new_poles.len())?;
    trimming_flat(
        degree,
        periodic,
        dim,
        knots,
        mults,
        &flat,
        u1,
        u2,
        new_knots,
        new_mults,
        &mut new_flat,
    )?;
    unpack(&new_flat, weights.is_some(), new_poles, new_weights)
}
