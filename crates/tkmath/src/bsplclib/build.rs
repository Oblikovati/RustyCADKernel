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
// Rust translation of FoundationClasses/TKMath/BSplCLib/BSplCLib.cxx (BuildKnots, BuildBoor, BoorIndex, GetPole), BSplCLib_2.cxx (BuildEval) and BSplCLib_CurveComputation.pxx (BSplCLib_BuildEval) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Extraction of the local knots and poles used by the evaluation schemes.
use crate::plib::PolePoint;
use crate::plib::access::{array_cell, array_value, non_negative, require_length};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;

const BUILD_KNOTS: &str = "BSplCLib::BuildKnots";
const BUILD_EVAL: &str = "BSplCLib::BuildEval";
const BUILD_BOOR: &str = "BSplCLib::BuildBoor";
const GET_POLE: &str = "BSplCLib::GetPole";

/// Stores in `lk` the 2*Degree knots used by the Boor scheme on the span
/// `Knots(Index)..Knots(Index+1)` (`BSplCLib::BuildKnots`).
///
/// Without multiplicities `knots` are flat knots. With them, knots missing before the
/// first or after the last knot of a non-periodic curve leave `lk` untouched, as natively;
/// `bsplclib::build_knots(3, 4, false, &flat, None, &mut lk)?`.
pub fn build_knots(
    degree: i32,
    index: i32,
    periodic: bool,
    knots: &Array1<'_, f64>,
    mults: Option<&Array1<'_, i32>>,
    lk: &mut [f64],
) -> Result<(), Failure> {
    let deg: usize = non_negative(BUILD_KNOTS, "Degree", degree)?;
    require_length(BUILD_KNOTS, "LK", lk.len(), 2 * deg)?;
    let Some(mults) = mults else {
        for (i, knot) in lk[..2 * deg].iter_mut().enumerate() {
            *knot = array_value(knots, index - degree + 1 + i as i32)?;
        }
        return Ok(());
    };
    let mut walker = KnotWalker::new(index, periodic, knots, mults)?;
    for i in 0..deg {
        if let Some(knot) = walker.next_lower()? {
            lk[deg - 1 - i] = knot;
        }
        if let Some(knot) = walker.next_upper()? {
            lk[deg + i] = knot;
        }
    }
    Ok(())
}

// Walks the repeated knots below and above a span, wrapping periodic curves.
struct KnotWalker<'k, 'a> {
    knots: &'k Array1<'a, f64>,
    mults: &'k Array1<'a, i32>,
    periodic: bool,
    dknot: f64,
    ilow: i32,
    mlow: i32,
    iupp: i32,
    mupp: i32,
    loffset: f64,
    uoffset: f64,
    getlow: bool,
    getupp: bool,
}

impl<'k, 'a> KnotWalker<'k, 'a> {
    fn new(
        index: i32,
        periodic: bool,
        knots: &'k Array1<'a, f64>,
        mults: &'k Array1<'a, i32>,
    ) -> Result<Self, Failure> {
        let mut walker = Self {
            knots,
            mults,
            periodic,
            dknot: 0.,
            ilow: index,
            mlow: 0,
            iupp: index + 1,
            mupp: 0,
            loffset: 0.,
            uoffset: 0.,
            getlow: true,
            getupp: true,
        };
        if periodic {
            walker.dknot = array_value(knots, knots.upper())? - array_value(knots, knots.lower())?;
            if walker.iupp > mults.upper() {
                walker.iupp = mults.lower() + 1;
                walker.uoffset = walker.dknot;
            }
        }
        Ok(walker)
    }

    fn next_lower(&mut self) -> Result<Option<f64>, Failure> {
        if !self.getlow {
            return Ok(None);
        }
        self.mlow += 1;
        if self.mlow > array_value(self.mults, self.ilow)? {
            self.mlow = 1;
            self.ilow -= 1;
            self.getlow = self.ilow >= self.mults.lower();
            if self.periodic && !self.getlow {
                self.ilow = self.mults.upper() - 1;
                self.loffset = self.dknot;
                self.getlow = true;
            }
        }
        if !self.getlow {
            return Ok(None);
        }
        Ok(Some(array_value(self.knots, self.ilow)? - self.loffset))
    }

    fn next_upper(&mut self) -> Result<Option<f64>, Failure> {
        if !self.getupp {
            return Ok(None);
        }
        self.mupp += 1;
        if self.mupp > array_value(self.mults, self.iupp)? {
            self.mupp = 1;
            self.iupp += 1;
            self.getupp = self.iupp <= self.mults.upper();
            if self.periodic && !self.getupp {
                self.iupp = self.mults.lower() + 1;
                self.uoffset = self.dknot;
                self.getupp = true;
            }
        }
        if !self.getupp {
            return Ok(None);
        }
        Ok(Some(array_value(self.knots, self.iupp)? + self.uoffset))
    }
}

/// Copies Degree+1 scalar poles (and weights, as `[w*p, w]` pairs) from
/// `Poles.Lower()+Index`, wrapping periodically (`BSplCLib::BuildEval` on reals);
/// `bsplclib::build_eval_scalar(2, 0, &poles, None, &mut lp)?`.
pub fn build_eval_scalar(
    degree: i32,
    index: i32,
    poles: &Array1<'_, f64>,
    weights: Option<&Array1<'_, f64>>,
    lp: &mut [f64],
) -> Result<(), Failure> {
    let count: usize = non_negative(BUILD_EVAL, "Degree", degree)? + 1;
    let stride: usize = if weights.is_some() { 2 } else { 1 };
    require_length(BUILD_EVAL, "LP", lp.len(), count * stride)?;
    let mut ip: i32 = poles.lower() + index - 1;
    for i in 0..count {
        ip = next_wrapped(ip, poles.lower(), poles.upper());
        let pole: &mut [f64] = &mut lp[i * stride..(i + 1) * stride];
        match weights {
            None => pole[0] = array_value(poles, ip)?,
            Some(weights) => {
                let w: f64 = array_value(weights, ip)?;
                pole[1] = w;
                pole[0] = array_value(poles, ip)? * w;
            }
        }
    }
    Ok(())
}

/// Copies Degree+1 points (homogeneous `[w*x.., w]` rows when weighted) from
/// `Poles.Lower()+Index`, wrapping periodically (`BSplCLib::BuildEval` on gp_Pnt/gp_Pnt2d).
///
/// Non-rational coordinates are written as `x + 0.0`, as the native traits do;
/// `bsplclib::build_eval(3, 0, &poles, None, &mut lp)?`.
pub fn build_eval<P: PolePoint>(
    degree: i32,
    index: i32,
    poles: &Array1<'_, P>,
    weights: Option<&Array1<'_, f64>>,
    lp: &mut [f64],
) -> Result<(), Failure> {
    let count: usize = non_negative(BUILD_EVAL, "Degree", degree)? + 1;
    let stride: usize = P::DIMENSION + usize::from(weights.is_some());
    require_length(BUILD_EVAL, "LP", lp.len(), count * stride)?;
    let mut ip: i32 = poles.lower() + index - 1;
    for i in 0..count {
        ip = next_wrapped(ip, poles.lower(), poles.upper());
        let point: P = array_value(poles, ip)?;
        let pole: &mut [f64] = &mut lp[i * stride..(i + 1) * stride];
        match weights {
            None => {
                for (k, value) in pole.iter_mut().enumerate() {
                    *value = point.pole_coordinate(k) + 0.0;
                }
            }
            Some(weights) => {
                let w: f64 = array_value(weights, ip)?;
                pole[P::DIMENSION] = w;
                for (k, value) in pole[..P::DIMENSION].iter_mut().enumerate() {
                    *value = point.pole_coordinate(k) * w;
                }
            }
        }
    }
    Ok(())
}

fn next_wrapped(ip: i32, lower: i32, upper: i32) -> i32 {
    if ip + 1 > upper { lower } else { ip + 1 }
}

/// Copies Length+1 poles of `dimension` values from `Poles.Lower()+Index*Dimension`
/// into the even positions of a Boor array (`BSplCLib::BuildBoor`);
/// `bsplclib::build_boor(0, 2, 1, &poles, &mut lp)?`.
pub fn build_boor(
    index: i32,
    length: i32,
    dimension: i32,
    poles: &Array1<'_, f64>,
    lp: &mut [f64],
) -> Result<(), Failure> {
    let dim: usize = non_negative(BUILD_BOOR, "Dimension", dimension)?;
    let mut ip: i32 = poles.lower() + index * dimension;
    for i in 0..(length + 1).max(0) as usize {
        require_length(BUILD_BOOR, "LP", lp.len(), (2 * i + 1) * dim)?;
        for k in 0..dim {
            lp[2 * i * dim + k] = array_value(poles, ip)?;
            ip = if ip + 1 > poles.upper() {
                poles.lower()
            } else {
                ip + 1
            };
        }
    }
    Ok(())
}

/// Position of pole `index` in a Boor array computed with `length` and `depth`
/// (`BSplCLib::BoorIndex`); `bsplclib::boor_index(2, 3, 1) == 3`.
pub fn boor_index(index: i32, length: i32, depth: i32) -> i32 {
    if index <= depth {
        return index;
    }
    if index <= length {
        return 2 * index - depth;
    }
    length + index - depth
}

/// Copies pole `index` of a Boor array to `Pole(Position..)`, advancing `position`
/// and wrapping it to `Pole.Lower()` past the upper bound (`BSplCLib::GetPole`);
/// `bsplclib::get_pole(1, 2, 1, 1, &lp, &mut position, &mut pole)?`.
pub fn get_pole(
    index: i32,
    length: i32,
    depth: i32,
    dimension: i32,
    lp: &[f64],
    position: &mut i32,
    pole: &mut Array1<'_, f64>,
) -> Result<(), Failure> {
    let dim: usize = non_negative(GET_POLE, "Dimension", dimension)?;
    let start: i64 = i64::from(boor_index(index, length, depth)) * dim as i64;
    for k in 0..dim {
        let value: f64 = crate::plib::access::buffer_value(GET_POLE, lp, start + k as i64)?;
        *array_cell(pole, *position)? = value;
        *position += 1;
    }
    if *position > pole.upper() {
        *position = pole.lower();
    }
    Ok(())
}
