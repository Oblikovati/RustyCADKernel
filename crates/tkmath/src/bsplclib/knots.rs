// Copyright (c) 1991-1999 Matra Datavision
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
// Rust translation of FoundationClasses/TKMath/BSplCLib/BSplCLib.cxx (knot and multiplicity management) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Knot vectors, multiplicities and their flat (repeated) sequences.
use crate::plib::access::{array_cell, array_value};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;

/// Bisection search of `x` in an increasing array (`BSplCLib::Hunt`).
///
/// Values before the first element give `Lower() - 1`, values after the last give
/// `Upper() + 1`; otherwise `array(pos) < x <= array(pos + 1)` except that an exact
/// hit on the first element returns `Lower()`; `bsplclib::hunt(&knots, 0.5, &mut pos)?`.
pub fn hunt(array: &Array1<'_, f64>, x: f64, x_pos: &mut i32) -> Result<(), Failure> {
    if array_value(array, array.lower())? > x {
        *x_pos = array.lower() - 1;
        return Ok(());
    }
    if array_value(array, array.upper())? < x {
        *x_pos = array.upper() + 1;
        return Ok(());
    }
    *x_pos = array.lower();
    if array.len() <= 1 {
        return Ok(());
    }
    let mut hi: i32 = array.upper();
    while hi - *x_pos != 1 {
        let mid: i32 = (hi + *x_pos) / 2;
        if array_value(array, mid)? < x {
            *x_pos = mid;
        } else {
            hi = mid;
        }
    }
    Ok(())
}

/// Index of the knot giving the start point of the curve (`BSplCLib::FirstUKnotIndex`);
/// `bsplclib::first_u_knot_index(3, &mults)? == 1` for end multiplicities 4.
pub fn first_u_knot_index(degree: i32, mults: &Array1<'_, i32>) -> Result<i32, Failure> {
    let mut index: i32 = mults.lower();
    let mut sigma_mult: i32 = array_value(mults, index)?;
    while sigma_mult <= degree {
        index += 1;
        sigma_mult += array_value(mults, index)?;
    }
    Ok(index)
}

/// Index of the knot giving the end point of the curve (`BSplCLib::LastUKnotIndex`);
/// `bsplclib::last_u_knot_index(3, &mults)? == mults.upper()` for end multiplicities 4.
pub fn last_u_knot_index(degree: i32, mults: &Array1<'_, i32>) -> Result<i32, Failure> {
    let mut index: i32 = mults.upper();
    let mut sigma_mult: i32 = array_value(mults, index)?;
    while sigma_mult <= degree {
        index -= 1;
        sigma_mult += array_value(mults, index)?;
    }
    Ok(index)
}

/// Index in the flat knot sequence matching knot `index` (`BSplCLib::FlatIndex`);
/// `bsplclib::flat_index(2, 2, &mults, false)?`.
pub fn flat_index(
    degree: i32,
    index: i32,
    mults: &Array1<'_, i32>,
    periodic: bool,
) -> Result<i32, Failure> {
    let lower: i32 = mults.lower();
    let mut result: i32 = index;
    for i in lower + 1..=index {
        result += array_value(mults, i)? - 1;
    }
    if periodic {
        result += degree;
    } else {
        result += array_value(mults, lower)? - 1;
    }
    Ok(result)
}

/// Greatest multiplicity between K1 and K2 (`BSplCLib::MaxKnotMult`);
/// `bsplclib::max_knot_mult(&mults, 1, 3)?`.
pub fn max_knot_mult(mults: &Array1<'_, i32>, from_k1: i32, to_k2: i32) -> Result<i32, Failure> {
    let mut result: i32 = array_value(mults, from_k1)?;
    for index in from_k1 + 1..=to_k2 {
        result = result.max(array_value(mults, index)?);
    }
    Ok(result)
}

/// Lowest multiplicity between K1 and K2 (`BSplCLib::MinKnotMult`);
/// `bsplclib::min_knot_mult(&mults, 1, 3)?`.
pub fn min_knot_mult(mults: &Array1<'_, i32>, from_k1: i32, to_k2: i32) -> Result<i32, Failure> {
    let mut result: i32 = array_value(mults, from_k1)?;
    for index in from_k1 + 1..=to_k2 {
        result = result.min(array_value(mults, index)?);
    }
    Ok(result)
}

/// Number of poles, or 0 when a multiplicity is invalid (`BSplCLib::NbPoles`);
/// `bsplclib::nb_poles(3, false, &mults)?`.
pub fn nb_poles(degree: i32, periodic: bool, mults: &Array1<'_, i32>) -> Result<i32, Failure> {
    let (f, l): (i32, i32) = (mults.lower(), mults.upper());
    let mf: i32 = array_value(mults, f)?;
    let ml: i32 = array_value(mults, l)?;
    if mf <= 0 || ml <= 0 {
        return Ok(0);
    }
    let Some(mut sigma) = end_sigma(degree, periodic, mf, ml) else {
        return Ok(0);
    };
    for i in f + 1..l {
        let mult: i32 = array_value(mults, i)?;
        if mult <= 0 || mult > degree {
            return Ok(0);
        }
        sigma += mult;
    }
    Ok(sigma)
}

// The contribution of the end multiplicities, or None when they are invalid.
fn end_sigma(degree: i32, periodic: bool, mf: i32, ml: i32) -> Option<i32> {
    if periodic {
        if mf > degree || ml > degree || mf != ml {
            return None;
        }
        return Some(mf);
    }
    let deg1: i32 = degree + 1;
    if mf > deg1 || ml > deg1 {
        return None;
    }
    Some(mf + ml - deg1)
}

/// Length of the flat knot sequence (`BSplCLib::KnotSequenceLength`);
/// `bsplclib::knot_sequence_length(&mults, 3, false)?`.
pub fn knot_sequence_length(
    mults: &Array1<'_, i32>,
    degree: i32,
    periodic: bool,
) -> Result<i32, Failure> {
    let mut length: i32 = 0;
    for i in mults.lower()..=mults.upper() {
        length += array_value(mults, i)?;
    }
    if periodic {
        length += 2 * (degree + 1 - array_value(mults, mults.lower())?);
    }
    Ok(length)
}

/// Flat knots of degree 0 (`BSplCLib::KnotSequence(Knots, Mults, KnotSeq, Periodic)`);
/// `bsplclib::knot_sequence(&knots, &mults, &mut flat, false)?`.
pub fn knot_sequence(
    knots: &Array1<'_, f64>,
    mults: &Array1<'_, i32>,
    knot_seq: &mut Array1<'_, f64>,
    periodic: bool,
) -> Result<(), Failure> {
    knot_sequence_with_degree(knots, mults, 0, periodic, knot_seq)
}

/// Repeats every knot by its multiplicity, extending periodic sequences by one period
/// on each side (`BSplCLib::KnotSequence` with Degree).
///
/// The sequence is written from index 1, whatever the lower bound of `knot_seq`;
/// `bsplclib::knot_sequence_with_degree(&knots, &mults, 3, false, &mut flat)?`.
pub fn knot_sequence_with_degree(
    knots: &Array1<'_, f64>,
    mults: &Array1<'_, i32>,
    degree: i32,
    periodic: bool,
    knot_seq: &mut Array1<'_, f64>,
) -> Result<(), Failure> {
    let m1: i32 = degree + 1 - array_value(mults, mults.lower())?;
    let mut index: i32 = if periodic { m1 + 1 } else { 1 };
    for i in knots.lower()..=knots.upper() {
        let knot: f64 = array_value(knots, i)?;
        // Multiplicities are read with the knot index, as the native raw pointer does.
        for _ in 1..=array_value(mults, i)? {
            *array_cell(knot_seq, index)? = knot;
            index += 1;
        }
    }
    if periodic {
        periodic_prefix(knots, mults, m1, knot_seq)?;
        periodic_suffix(knots, mults, index, knot_seq)?;
    }
    Ok(())
}

// Fills KnotSeq(M1..1) backwards with the knots before the last one, shifted by -period.
fn periodic_prefix(
    knots: &Array1<'_, f64>,
    mults: &Array1<'_, i32>,
    m1: i32,
    knot_seq: &mut Array1<'_, f64>,
) -> Result<(), Failure> {
    let period: f64 = array_value(knots, knots.upper())? - array_value(knots, knots.lower())?;
    let (mut m, mut j): (i32, i32) = (1, knots.upper() - 1);
    for i in (1..=m1).rev() {
        *array_cell(knot_seq, i)? = array_value(knots, j)? - period;
        m += 1;
        if m > array_value(mults, j)? {
            j -= 1;
            m = 1;
        }
    }
    Ok(())
}

// Fills KnotSeq(index..Upper) with the knots after the first one, shifted by +period.
fn periodic_suffix(
    knots: &Array1<'_, f64>,
    mults: &Array1<'_, i32>,
    index: i32,
    knot_seq: &mut Array1<'_, f64>,
) -> Result<(), Failure> {
    let period: f64 = array_value(knots, knots.upper())? - array_value(knots, knots.lower())?;
    let (mut m, mut j): (i32, i32) = (1, knots.lower() + 1);
    for i in index..=knot_seq.upper() {
        *array_cell(knot_seq, i)? = array_value(knots, j)? + period;
        m += 1;
        if m > array_value(mults, j)? {
            j += 1;
            m = 1;
        }
    }
    Ok(())
}

/// Number of distinct values of a flat sequence read from index 1 (`BSplCLib::KnotsLength`);
/// the periodic flag is ignored natively; `bsplclib::knots_length(&flat, false)? == 2`.
pub fn knots_length(knot_seq: &Array1<'_, f64>, _periodic: bool) -> Result<i32, Failure> {
    let mut size_mult: i32 = 1;
    let mut val: f64 = array_value(knot_seq, 1)?;
    for jj in 2..=knot_seq.len() as i32 {
        let knot: f64 = array_value(knot_seq, jj)?;
        // Strict equality on nodes, as the native test.
        if knot != val {
            val = knot;
            size_mult += 1;
        }
    }
    Ok(size_mult)
}

/// Splits a flat sequence read from index 1 into knots and multiplicities written from
/// index 1 (`BSplCLib::Knots`); `bsplclib::knots(&flat, &mut knots, &mut mults, false)?`.
pub fn knots(
    knot_seq: &Array1<'_, f64>,
    knots: &mut Array1<'_, f64>,
    mults: &mut Array1<'_, i32>,
    _periodic: bool,
) -> Result<(), Failure> {
    let mut val: f64 = array_value(knot_seq, 1)?;
    let mut kk: i32 = 1;
    *array_cell(knots, kk)? = val;
    *array_cell(mults, kk)? = 1;
    for jj in 2..=knot_seq.len() as i32 {
        let knot: f64 = array_value(knot_seq, jj)?;
        if knot != val {
            val = knot;
            kk += 1;
            *array_cell(knots, kk)? = val;
            *array_cell(mults, kk)? = 1;
        } else {
            *array_cell(mults, kk)? += 1;
        }
    }
    Ok(())
}

/// First pole used on the span starting at knot `index` (`BSplCLib::PoleIndex`);
/// the result is an offset to add to `Poles.Lower()`; `bsplclib::pole_index(3, 1, false, &mults)?`.
pub fn pole_index(
    degree: i32,
    index: i32,
    periodic: bool,
    mults: &Array1<'_, i32>,
) -> Result<i32, Failure> {
    let mut pindex: i32 = 0;
    for i in mults.lower()..=index {
        pindex += array_value(mults, i)?;
    }
    if periodic {
        pindex -= array_value(mults, mults.lower())?;
    } else {
        pindex -= degree + 1;
    }
    Ok(pindex)
}
