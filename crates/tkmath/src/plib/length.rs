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
// Rust translation of FoundationClasses/TKMath/PLib/PLib.cxx (EvalLength) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Arc length of a polynomial curve by symmetric Gauss-Legendre quadrature.
use super::access::{non_negative, require_length};
use crate::math::{Vector, gauss_points, gauss_weights};
use rustycad_tkernel::standard::Failure;

const NAME: &str = "PLib::EvalLength";

/// Integrates |P'(u)| over [U1, U2] with 4*min(Degree/4+1, 10) Gauss points
/// (`PLib::EvalLength`); `plib::eval_length(1, 1, &[0.0, 2.0], 0.0, 1.0)? == 2.0`.
pub fn eval_length(
    degree: i32,
    dimension: i32,
    polynomial_coeff: &[f64],
    u1: f64,
    u2: f64,
) -> Result<f64, Failure> {
    let order: usize = non_negative(NAME, "Degree", degree)?;
    let dim: usize = non_negative(NAME, "Dimension", dimension)?;
    require_length(
        NAME,
        "PolynomialCoeff",
        polynomial_coeff.len(),
        (order + 1) * dim,
    )?;
    let nb_gauss_points: i32 = 4 * (degree / 4 + 1).min(10);
    let mut points: Vector<'static> = Vector::with_bounds(1, nb_gauss_points)?;
    gauss_points(nb_gauss_points, &mut points);
    let mut weights: Vector<'static> = Vector::with_bounds(1, nb_gauss_points)?;
    gauss_weights(nb_gauss_points, &mut weights);
    let c1: f64 = (u2 + u1) / 2.0;
    let c2: f64 = (u2 - u1) / 2.0;
    let mut sum: f64 = 0.0;
    for j in 1..=nb_gauss_points / 2 {
        // Each positive node covers its mirrored node, as in the native symmetric loop.
        let tran: f64 = c2 * points[j];
        let (speed1, speed2) = squared_speeds(order, dim, polynomial_coeff, c1 + tran, c1 - tran);
        sum += weights[j] * c2 * (speed1.sqrt() + speed2.sqrt());
    }
    Ok(sum)
}

// Squared norms of P'(x1) and P'(x2), summed over components in native order.
fn squared_speeds(order: usize, dim: usize, coefficients: &[f64], x1: f64, x2: f64) -> (f64, f64) {
    let (mut der1, mut der2) = (0.0, 0.0);
    for idim in 0..dim {
        let leading: f64 = order as f64 * coefficients[idim + order * dim];
        let (mut d1, mut d2) = (leading, leading);
        for i in (1..order).rev() {
            let dd: f64 = i as f64 * coefficients[idim + i * dim];
            d1 = d1 * x1 + dd;
            d2 = d2 * x2 + dd;
        }
        der1 += d1 * d1;
        der2 += d2 * d2;
    }
    (der1, der2)
}

/// Doubles the subinterval count until successive lengths differ by at most Tol,
/// for at most 13 evaluations (`PLib::EvalLength` with tolerance).
///
/// Returns `(length, error)`; `plib::eval_length_with_tolerance(1, 1, &c, 0.0, 1.0, 1e-9)?`.
pub fn eval_length_with_tolerance(
    degree: i32,
    dimension: i32,
    polynomial_coeff: &[f64],
    u1: f64,
    u2: f64,
    tol: f64,
) -> Result<(f64, f64), Failure> {
    const MAX_NB_ITER: i32 = 13;
    let mut length: f64 = eval_length(degree, dimension, polynomial_coeff, u1, u2)?;
    let (mut nb_sub_int, mut nb_iter) = (1, 1);
    loop {
        let old_length: f64 = length;
        length = 0.0;
        nb_sub_int *= 2;
        let du: f64 = (u2 - u1) / f64::from(nb_sub_int);
        for i in 1..=nb_sub_int {
            let (first, last) = (u1 + f64::from(i - 1) * du, u1 + f64::from(i) * du);
            length += eval_length(degree, dimension, polynomial_coeff, first, last)?;
        }
        nb_iter += 1;
        let error: f64 = (old_length - length).abs();
        if !(error > tol && nb_iter <= MAX_NB_ITER) {
            return Ok((length, error));
        }
    }
}
