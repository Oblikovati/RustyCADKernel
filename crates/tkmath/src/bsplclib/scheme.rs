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
// Rust translation of FoundationClasses/TKMath/BSplCLib/BSplCLib.cxx (Eval, BoorScheme, AntiBoorScheme, Derivative) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! De Boor schemes on local pole and knot buffers.
//!
//! The native dimension 1..4 specialisations perform the same per-component
//! operations as the general loop, so one loop reproduces them bit for bit.
use crate::plib::access::{non_negative, require_length};
use rustycad_tkernel::standard::Failure;

const EVAL: &str = "BSplCLib::Eval";
const BOOR: &str = "BSplCLib::BoorScheme";
const ANTI_BOOR: &str = "BSplCLib::AntiBoorScheme";
const DERIVATIVE: &str = "BSplCLib::Derivative";

/// Evaluates in place the de Boor scheme of one span (`BSplCLib::Eval(U, Degree,
/// Knots, Dimension, Poles)`).
///
/// `knots` holds the 2*Degree knots around the span, `poles` the Degree+1 poles of
/// `dimension` values; the point is left in the first pole;
/// `bsplclib::eval(0.5, 1, &[0.0, 1.0], 1, &mut [0.0, 2.0])?`.
pub fn eval(
    u: f64,
    degree: i32,
    knots: &[f64],
    dimension: i32,
    poles: &mut [f64],
) -> Result<(), Failure> {
    let deg: usize = non_negative(EVAL, "Degree", degree)?;
    let dim: usize = non_negative(EVAL, "Dimension", dimension)?;
    require_length(EVAL, "Knots", knots.len(), 2 * deg)?;
    require_length(EVAL, "Poles", poles.len(), (deg + 1) * dim)?;
    // Level `level` combines Degree-level poles; knot pairs are (Degree-1+i+1, level+i).
    for level in 0..deg {
        let count: usize = deg - level;
        for i in 0..count {
            let upper: f64 = knots[deg + i];
            let x: f64 = (upper - u) / (upper - knots[level + i]);
            let y: f64 = 1.0 - x;
            for k in 0..dim {
                let p: usize = i * dim + k;
                poles[p] *= x;
                poles[p] += y * poles[p + dim];
            }
        }
    }
    Ok(())
}

/// Runs the Boor scheme to `depth` on `length+1` interleaved poles
/// (`BSplCLib::BoorScheme`).
///
/// Poles sit at even pole positions of `poles`, which holds (2*Length+1)*Dimension
/// values; intermediate points fill the odd positions (see [`boor_index`](super::boor_index));
/// `bsplclib::boor_scheme(0.5, 1, &knots, 1, &mut poles, 1, 1)?`.
pub fn boor_scheme(
    u: f64,
    degree: i32,
    knots: &[f64],
    dimension: i32,
    poles: &mut [f64],
    depth: i32,
    length: i32,
) -> Result<(), Failure> {
    let dim: usize = non_negative(BOOR, "Dimension", dimension)?;
    for step in 0..depth.max(0) {
        for i in step..length {
            let x: f64 = boor_ratio(BOOR, knots, u, degree, step, i)?;
            let y: f64 = 1. - x;
            let pole: usize = scheme_offset(dim, i, step);
            require_length(BOOR, "Poles", poles.len(), pole + 2 * dim)?;
            for k in 0..dim {
                poles[pole + k] = x * poles[pole + k - dim] + y * poles[pole + k + dim];
            }
        }
    }
    Ok(())
}

/// Reverses the Boor scheme to recover the poles before an insertion
/// (`BSplCLib::AntiBoorScheme`).
///
/// Returns false as soon as the two candidate poles differ by more than `tolerance`;
/// `bsplclib::anti_boor_scheme(0.5, 2, &knots, 1, &mut poles, 1, 2, 1e-7)?`.
#[allow(clippy::too_many_arguments)] // Mirrors the native eight-argument signature.
pub fn anti_boor_scheme(
    u: f64,
    degree: i32,
    knots: &[f64],
    dimension: i32,
    poles: &mut [f64],
    depth: i32,
    length: i32,
    tolerance: f64,
) -> Result<bool, Failure> {
    let dim: usize = non_negative(ANTI_BOOR, "Dimension", dimension)?;
    let scheme = AntiBoor {
        u,
        degree,
        knots,
        dim,
        tolerance,
    };
    if length == 1 {
        return scheme.central_point(poles, depth);
    }
    for step in (0..depth).rev() {
        scheme.left_to_right(poles, step, length)?;
        if !scheme.right_to_left(poles, step, length)? {
            return Ok(false);
        }
    }
    Ok(true)
}

struct AntiBoor<'k> {
    u: f64,
    degree: i32,
    knots: &'k [f64],
    dim: usize,
    tolerance: f64,
}

impl AntiBoor<'_> {
    // Length == 1: only the central point is verified.
    fn central_point(&self, poles: &[f64], depth: i32) -> Result<bool, Failure> {
        let x: f64 = boor_ratio(ANTI_BOOR, self.knots, self.u, self.degree, 0, 0)?;
        let y: f64 = 1. - x;
        let first: usize = pole_offset(ANTI_BOOR, i64::from(depth - 1) * self.dim as i64)?;
        require_length(ANTI_BOOR, "Poles", poles.len(), first + 3 * self.dim)?;
        for k in 0..self.dim {
            let z: f64 = x * poles[first + k] + y * poles[first + k + 2 * self.dim];
            if (z - poles[first + k + self.dim]).abs() > self.tolerance {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn left_to_right(&self, poles: &mut [f64], step: i32, length: i32) -> Result<(), Failure> {
        let dim: usize = self.dim;
        for i in step..length - 1 {
            let x: f64 = boor_ratio(ANTI_BOOR, self.knots, self.u, self.degree, step, i)?;
            let y: f64 = 1. - x;
            let pole: usize = scheme_offset(dim, i, step);
            require_length(ANTI_BOOR, "Poles", poles.len(), pole + 2 * dim)?;
            for k in 0..dim {
                poles[pole + k + dim] = (poles[pole + k] - x * poles[pole + k - dim]) / y;
            }
        }
        Ok(())
    }

    // Only half of the way is walked back: going further degenerates through overflows.
    fn right_to_left(&self, poles: &mut [f64], step: i32, length: i32) -> Result<bool, Failure> {
        let dim: usize = self.dim;
        let half_length: i32 = (length - 1 + step) / 2;
        for i in (half_length + 1..length).rev() {
            let x: f64 = boor_ratio(ANTI_BOOR, self.knots, self.u, self.degree, step, i)?;
            let y: f64 = 1. - x;
            let pole: usize = scheme_offset(dim, i, step);
            require_length(ANTI_BOOR, "Poles", poles.len(), pole + 2 * dim)?;
            for k in 0..dim {
                let z: f64 = (poles[pole + k] - y * poles[pole + k + dim]) / x;
                if (z - poles[pole + k - dim]).abs() > self.tolerance {
                    return Ok(false);
                }
                poles[pole + k - dim] += z;
                poles[pole + k - dim] /= 2.;
            }
        }
        Ok(true)
    }
}

// X = (knot(i+Degree-step) - U) / (knot(i+Degree-step) - knot(i)).
fn boor_ratio(
    function: &str,
    knots: &[f64],
    u: f64,
    degree: i32,
    step: i32,
    i: i32,
) -> Result<f64, Failure> {
    let upper: f64 = knot_at(function, knots, i64::from(i) + i64::from(degree - step))?;
    Ok((upper - u) / (upper - knot_at(function, knots, i64::from(i))?))
}

fn knot_at(function: &str, knots: &[f64], index: i64) -> Result<f64, Failure> {
    crate::plib::access::buffer_value(function, knots, index)
}

// Position of P(step+1, i-step) in the interleaved Boor array: Dimension*(2i-step+1).
fn scheme_offset(dim: usize, i: i32, step: i32) -> usize {
    dim * (2 * i - step + 1) as usize
}

fn pole_offset(function: &str, offset: i64) -> Result<usize, Failure> {
    usize::try_from(offset).map_err(|_| {
        Failure::out_of_range(&format!(
            "{function}: pole offset {offset} precedes the Poles buffer; expected >= 0"
        ))
    })
}

/// Replaces poles by the poles of the derivatives up to `order`
/// (`BSplCLib::Derivative`); `bsplclib::derivative(2, &knots, 1, 3, 1, &mut poles)?`.
pub fn derivative(
    degree: i32,
    knots: &[f64],
    dimension: i32,
    length: i32,
    order: i32,
    poles: &mut [f64],
) -> Result<(), Failure> {
    let dim: usize = non_negative(DERIVATIVE, "Dimension", dimension)?;
    let mut span: i32 = degree;
    for step in 1..=order {
        for i in step..length {
            let index: i64 = i64::from(i);
            let delta: f64 = knot_at(DERIVATIVE, knots, index + i64::from(span))?
                - knot_at(DERIVATIVE, knots, index)?;
            // The native negation happens on the integer span, so span 0 gives +0.0.
            let coef: f64 = f64::from(-span) / delta;
            let pole: usize = (i - step) as usize * dim;
            require_length(DERIVATIVE, "Poles", poles.len(), pole + 2 * dim)?;
            for k in 0..dim {
                poles[pole + k] -= poles[pole + k + dim];
                poles[pole + k] *= coef;
            }
        }
        span -= 1;
    }
    Ok(())
}
