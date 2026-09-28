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
// Rust translation of FoundationClasses/TKMath/PLib/PLib.cxx (RationalDerivative[s]) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Derivatives of a ratio u/w by the Leibniz recurrence
//! f^(q) = (u^(q) - sum_{p<q} C(q,p) f^(p) w^(q-p)) / w.
//!
//! The native code carries an unrolled copy of each loop for Dimension == 3. It
//! performs the same operations in the same order as the general loop (same
//! element updates, same `jj` order), so one loop reproduces both bit for bit.
use super::access::{non_negative, require_length};
use rustycad_tkernel::standard::Failure;

const DERIVATIVE: &str = "PLib::RationalDerivative";
const DERIVATIVES: &str = "PLib::RationalDerivatives";

/// Differentiates homogeneous rows `x(1..Dimension), w` N times (`PLib::RationalDerivative`).
///
/// Input rows above `Degree` are treated as zero. With `all` the result holds
/// Dimension*(N+1) values, otherwise only the N-th derivative;
/// `plib::rational_derivative(2, 1, 1, &ders, &mut out, true)?`.
pub fn rational_derivative(
    degree: i32,
    derivative_request: i32,
    dimension: i32,
    ders: &[f64],
    rders: &mut [f64],
    all: bool,
) -> Result<(), Failure> {
    let request: usize = non_negative(DERIVATIVE, "DerivativeRequest", derivative_request)?;
    let dim: usize = non_negative(DERIVATIVE, "Dimension", dimension)?;
    let known: i64 = i64::from(derivative_request.min(degree));
    // Row 0 is always read for the leading weight, even when Degree < 0.
    let rows: usize = (known + 1).max(1) as usize;
    require_length(DERIVATIVE, "Ders", ders.len(), rows * (dim + 1))?;
    let written: usize = if all { (request + 1) * dim } else { dim };
    require_length(DERIVATIVE, "RDers", rders.len(), written)?;
    if all {
        homogeneous_rows(ders, dim, request, known, rders);
        return Ok(());
    }
    let mut storage: Vec<f64> = vec![0.0; (request + 1) * dim];
    homogeneous_rows(ders, dim, request, known, &mut storage);
    rders[..dim].copy_from_slice(&storage[request * dim..]);
    Ok(())
}

fn homogeneous_rows(ders: &[f64], dim: usize, request: usize, known: i64, out: &mut [f64]) {
    let stride: usize = dim + 1;
    let inverse: f64 = 1.0e0 / ders[dim];
    let mut binomial: Vec<f64> = vec![1.0e0; request + 1];
    for ii in 0..=request {
        let is_known: bool = ii as i64 <= known;
        for kk in 0..dim {
            out[ii * dim + kk] = if is_known {
                ders[ii * stride + kk]
            } else {
                0.0e0
            };
        }
        // Rows beyond Degree only combine the last `known` lower orders (jj >= ii - known).
        let lowest: usize = if is_known {
            0
        } else {
            (ii as i64 - known) as usize
        };
        let weight = |order: usize| ders[order * stride + dim];
        finish_row(out, dim, ii, lowest, &mut binomial, weight, inverse);
    }
}

/// Differentiates poles and weights stored separately (`PLib::RationalDerivatives`).
///
/// Poles hold Dimension*(N+1) values, weights N+1 values;
/// `plib::rational_derivatives(1, 3, &poles, &weights, &mut out)?`.
pub fn rational_derivatives(
    derivative_request: i32,
    dimension: i32,
    poles_derivatives: &[f64],
    weights_derivatives: &[f64],
    rational_derivatives: &mut [f64],
) -> Result<(), Failure> {
    let request: usize = non_negative(DERIVATIVES, "DerivativesRequest", derivative_request)?;
    let dim: usize = non_negative(DERIVATIVES, "Dimension", dimension)?;
    let values: usize = (request + 1) * dim;
    require_length(
        DERIVATIVES,
        "PolesDerivatives",
        poles_derivatives.len(),
        values,
    )?;
    require_length(
        DERIVATIVES,
        "WeightsDerivatives",
        weights_derivatives.len(),
        request + 1,
    )?;
    require_length(
        DERIVATIVES,
        "RationalDerivates",
        rational_derivatives.len(),
        values,
    )?;
    let inverse: f64 = 1.0e0 / weights_derivatives[0];
    let mut binomial: Vec<f64> = vec![1.0e0; request + 1];
    for ii in 0..=request {
        let row: std::ops::Range<usize> = ii * dim..(ii + 1) * dim;
        rational_derivatives[row.clone()].copy_from_slice(&poles_derivatives[row]);
        let weight = |order: usize| weights_derivatives[order];
        finish_row(
            rational_derivatives,
            dim,
            ii,
            0,
            &mut binomial,
            weight,
            inverse,
        );
    }
    Ok(())
}

// Subtracts C(ii,jj) w^(ii-jj) f^(jj) for jj descending, advances the binomial row, divides by w.
fn finish_row(
    out: &mut [f64],
    dim: usize,
    ii: usize,
    lowest: usize,
    binomial: &mut [f64],
    weight: impl Fn(usize) -> f64,
    inverse: f64,
) {
    for jj in (lowest..ii).rev() {
        let factor: f64 = binomial[jj] * weight(ii - jj);
        for kk in 0..dim {
            out[ii * dim + kk] -= factor * out[jj * dim + kk];
        }
    }
    for jj in (1..=ii).rev() {
        binomial[jj] += binomial[jj - 1];
    }
    for kk in 0..dim {
        out[ii * dim + kk] *= inverse;
    }
}
