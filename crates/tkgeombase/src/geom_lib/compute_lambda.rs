// Created on: 1993-07-07
// Created by: Jean Claude VAUTHIER
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
// Rust translation of ModelingData/TKGeomBase/GeomLib/GeomLib.cxx (ComputeLambda)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ComputeLambda`: the tangent scale of a curve extension that keeps the speed
//! along the extension closest to the speed at the junction.
use super::log_sample::LogSample;
use super::poly_func::PolyFunc;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::math::{self, FunctionAllRoots, Matrix, Vector};
use rustycad_tkmath::plib;

// Curve extension and lambda optimization.
const THE_LAMBDA_MIN_FACTOR: f64 = 1.0e-3;
const THE_LAMBDA_MAX_FACTOR: f64 = 50.0;
const THE_LAMBDA_SAMPLE_COUNT: i32 = 100;
const THE_LAMBDA_ROOT_VALUE_TOLERANCE: f64 = 1.0e-15;

/// `ComputeLambda(Constraint, Hermit, Length, Lambda)`: minimizes over Lambda the
/// integral of `(|C'(t)|^2 - |C'(0)|^2)^2` for the Hermite extension of the
/// `Constraint` columns (point, derivatives, target) scaled by powers of Lambda,
/// starting from `lambda`.
///
/// The polynomial in Lambda is integrated with `4 + 4 Continuity` Gauss points; when
/// its value at `lambda` exceeds Precision::Confusion(), the roots of its derivative
/// on a logarithmic sampling of `[1e-3 lambda, 50 lambda]` are tried and the best
/// one kept. A failed root search raises its Standard_NumericError, as natively.
pub(super) fn compute_lambda(
    constraint: &Matrix<'_>,
    hermit: &Matrix<'_>,
    length: f64,
    lambda: &mut f64,
) -> Result<(), Failure> {
    let hermite_order: usize = hermit.row_size();
    let continuity: i32 = hermite_order as i32 - 2;
    let hder: Vec<f64> = derivative_coefficients(hermit, hermite_order)?;
    let pol4: Vector<'static> = energy_polynomial(constraint, &hder, continuity, length)?;
    let pol4_values: Vec<f64> = pol4.array1().iter().copied().collect();
    let degree: i32 = pol4.length() - 1;
    let mut e_min: f64 = eval(&pol4_values, degree, *lambda)?;
    if e_min <= precision::confusion() {
        return Ok(());
    }
    // Search for extrema of the function.
    let mut ff = PolyFunc::new(&pol4)?;
    let s = LogSample::new(
        *lambda * THE_LAMBDA_MIN_FACTOR,
        *lambda * THE_LAMBDA_MAX_FACTOR,
        THE_LAMBDA_SAMPLE_COUNT,
    );
    let eps = [
        precision::confusion(),
        precision::confusion() * (length + 1.0),
        THE_LAMBDA_ROOT_VALUE_TOLERANCE,
    ];
    let solve = FunctionAllRoots::new(&mut ff, &s, eps)?;
    if !solve.is_done() {
        return Ok(());
    }
    for point_index in 1..=solve.number_of_points()? {
        let t: f64 = solve.point(point_index)?;
        let e: f64 = eval(&pol4_values, degree, t)?;
        if e < e_min {
            *lambda = t;
            e_min = e;
        }
    }
    Ok(())
}

// PLib::NoDerivativeEvalPolynomial of a scalar polynomial of `degree`.
fn eval(coefficients: &[f64], degree: i32, t: f64) -> Result<f64, Failure> {
    let mut value: [f64; 1] = [0.0];
    plib::no_derivative_eval_polynomial(t, degree, 1, degree, coefficients, &mut value)?;
    Ok(value[0])
}

// HDer, row-major: HDer(row, column) = (row + 1) Hermit(column, row + 1), the
// coefficients of the derivatives of the Hermite polynomials.
fn derivative_coefficients(hermit: &Matrix<'_>, order: usize) -> Result<Vec<f64>, Failure> {
    let mut hder: Vec<f64> = vec![0.0; (order - 1) * order];
    for column in 0..order {
        for row in 0..order - 1 {
            let value: f64 = *hermit.at(column, row + 1).map_err(matrix_error)?;
            hder[row * order + column] = (row + 1) as f64 * value;
        }
    }
    Ok(hder)
}

fn matrix_error(error: rustycad_tkernel::ncollection::ArrayError) -> Failure {
    crate::native_failure::array_failure(error, "ComputeLambda")
}

// The polynomial pol4 of `(|C'(t)|^2 - |C'(0)|^2)^2` integrated over the Gauss points.
fn energy_polynomial(
    constraint: &Matrix<'_>,
    hder: &[f64],
    continuity: i32,
    length: f64,
) -> Result<Vector<'static>, Failure> {
    let order: i32 = continuity + 2;
    let mut vec2: Vector<'static> = constraint.col(2)?;
    vec2.divide(length)?;
    let squared1: f64 = vec2.norm2()?;
    let g_ordre: i32 = 4 + 4 * continuity;
    let d_dim: i32 = continuity * (continuity + 2);
    let mut gauss_p: Vector<'static> = Vector::with_bounds(1, g_ordre)?;
    let mut gauss_w: Vector<'static> = Vector::with_bounds(1, g_ordre)?;
    math::gauss_points(g_ordre, &mut gauss_p);
    math::gauss_weights(g_ordre, &mut gauss_w);
    let mut pol4: Vector<'static> = Vector::filled(1, 4 * continuity + 1, 0.0)?;
    for ip in 1..=g_ordre {
        let t: f64 = (*gauss_p.value(ip)? + 1.0) / 2.0;
        let gw: f64 = *gauss_w.value(ip)?;
        let mut valhder: Vec<f64> = vec![0.0; order as usize];
        plib::no_derivative_eval_polynomial(t, continuity, order, d_dim, hder, &mut valhder)?;
        let mut v: Vector<'_> = Vector::from_buffer(&mut valhder, 1, order)?;
        // Normalisation.
        v.divide(length)?;
        let pol2: Vec<f64> = speed_polynomial(constraint, &v, continuity, squared1)?;
        accumulate(&mut pol4, &pol2, gw)?;
    }
    Ok(pol4)
}

// pol2 = |C'(t)|^2 - |C'(0)|^2 as a polynomial in Lambda, where
// C'(t) = SUM Vi * Lambda^i.
fn speed_polynomial(
    constraint: &Matrix<'_>,
    v: &Vector<'_>,
    continuity: i32,
    squared1: f64,
) -> Result<Vec<f64>, Failure> {
    let order: i32 = continuity + 2;
    let scaled_col = |col: i32, factor: f64| -> Result<Vector<'static>, Failure> {
        let mut column: Vector<'static> = constraint.col(col)?;
        column.multiply(factor);
        Ok(column)
    };
    let mut vec1: Vector<'static> = scaled_col(1, *v.value(1)?)?;
    let mut last: Vector<'static> = constraint.col(constraint.upper_col())?;
    last = last.multiplied(*v.value(order)?);
    vec1.add(&last)?;
    let vec2: Vector<'static> = scaled_col(2, *v.value(2)?)?;
    let mut pol2: Vec<f64> = vec![0.0; (2 * continuity + 1) as usize];
    pol2[0] = vec1.norm2()?;
    pol2[1] = 2.0 * vec1.dot(&vec2)?;
    pol2[2] = vec2.norm2()? - squared1;
    if continuity > 1 {
        let vec3: Vector<'static> = scaled_col(3, *v.value(3)?)?;
        pol2[2] += 2.0 * vec1.dot(&vec3)?;
        pol2[3] = 2.0 * vec2.dot(&vec3)?;
        pol2[4] = vec3.norm2()?;
        if continuity > 2 {
            let vec4: Vector<'static> = scaled_col(4, *v.value(4)?)?;
            pol2[3] += 2.0 * vec1.dot(&vec4)?;
            pol2[4] += 2.0 * vec2.dot(&vec4)?;
            pol2[5] = 2.0 * vec3.dot(&vec4)?;
            pol2[6] = vec4.norm2()?;
        }
    }
    Ok(pol2)
}

// Integrale de (C'(t)^2 - C'(0)^2)^2: pol4 += GW pol2^2, the cross products first.
fn accumulate(pol4: &mut Vector<'static>, pol2: &[f64], gw: f64) -> Result<(), Failure> {
    for first_index in 0..pol2.len() {
        for second_index in 0..first_index {
            *pol4.change_at(first_index + second_index)? +=
                2.0 * gw * pol2[first_index] * pol2[second_index];
        }
        // std::pow(x, 2) is the libm pow, as powf.
        *pol4.change_at(2 * first_index)? += gw * pol2[first_index].powf(2.0);
    }
    Ok(())
}
