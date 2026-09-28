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
// Rust translation of FoundationClasses/TKMath/BSplCLib/BSplCLib.cxx (BuildSchoenbergPoints, Interpolate) and BSplCLib_CurveComputation.pxx (BSplCLib_Interpolate) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Interpolation of values and derivatives through the banded collocation system.
use super::banded::{
    build_bsp_matrix, factor_banded_matrix, solve_banded_system, solve_banded_system_rational,
};
use super::{flat_points, store_flat_points};
use crate::math::Matrix;
use crate::plib::PolePoint;
use crate::plib::access::{array_cell, array_value};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;

const INTERPOLATE: &str = "BSplCLib::Interpolate";

/// Averages of Degree consecutive flat knots, the Schoenberg points
/// (`BSplCLib::BuildSchoenbergPoints`): `Parameters(i) = sum FlatKnots(i+1..i+Degree) / Degree`;
/// `bsplclib::build_schoenberg_points(3, &flat, &mut parameters)?`.
pub fn build_schoenberg_points(
    degree: i32,
    flat_knots: &Array1<'_, f64>,
    parameters: &mut Array1<'_, f64>,
) -> Result<(), Failure> {
    let inverse: f64 = 1.0e0 / f64::from(degree);
    for ii in parameters.lower()..=parameters.upper() {
        let mut sum: f64 = 0.0e0;
        for jj in 1..=degree {
            sum += array_value(flat_knots, jj + ii)?;
        }
        *array_cell(parameters, ii)? = sum * inverse;
    }
    Ok(())
}

// The collocation matrix of the parameters, factored; fails as natively on any error.
fn factored_matrix(
    degree: i32,
    flat_knots: &Array1<'_, f64>,
    parameters: &Array1<'_, f64>,
    contact_order_array: &Array1<'_, i32>,
    inversion_problem: &mut i32,
) -> Result<(Matrix<'static>, i32, i32), Failure> {
    let mut matrix: Matrix<'static> =
        Matrix::with_bounds(1, parameters.len() as i32, 1, 2 * degree + 1)
            .map_err(|error| Failure::out_of_range(&format!("{INTERPOLATE}: {error}")))?;
    let (status, upper, lower) = build_bsp_matrix(
        parameters,
        contact_order_array,
        flat_knots,
        degree,
        &mut matrix,
    )?;
    if status != 0 {
        return Err(Failure::out_of_range(INTERPOLATE));
    }
    let (status, pivot) = factor_banded_matrix(&mut matrix, upper, lower)?;
    *inversion_problem = pivot;
    if status != 0 {
        return Err(Failure::out_of_range(INTERPOLATE));
    }
    Ok((matrix, upper, lower))
}

/// Replaces the values/derivatives in `poles` (rows of `array_dimension`) by the poles of
/// the interpolating B-spline (`BSplCLib::Interpolate` on reals).
///
/// Fails with Standard_OutOfRange when the system cannot be built, factored or solved;
/// `inversion_problem` then holds the failing pivot index;
/// `bsplclib::interpolate_flat(3, &flat, &params, &orders, 1, &mut values, &mut problem)?`.
pub fn interpolate_flat(
    degree: i32,
    flat_knots: &Array1<'_, f64>,
    parameters: &Array1<'_, f64>,
    contact_order_array: &Array1<'_, i32>,
    array_dimension: i32,
    poles: &mut [f64],
    inversion_problem: &mut i32,
) -> Result<(), Failure> {
    let (matrix, upper, lower) = factored_matrix(
        degree,
        flat_knots,
        parameters,
        contact_order_array,
        inversion_problem,
    )?;
    if solve_banded_system(&matrix, upper, lower, array_dimension, poles)? != 0 {
        return Err(Failure::out_of_range(INTERPOLATE));
    }
    Ok(())
}

/// Rational interpolation: poles are multiplied by the weights, interpolated with them,
/// then divided by the interpolated weights (`BSplCLib::Interpolate` on reals with Weights);
/// `bsplclib::interpolate_flat_rational(3, &flat, &params, &orders, 1, &mut v, &mut w, &mut problem)?`.
#[allow(clippy::too_many_arguments)] // Mirrors the native eight-argument signature.
pub fn interpolate_flat_rational(
    degree: i32,
    flat_knots: &Array1<'_, f64>,
    parameters: &Array1<'_, f64>,
    contact_order_array: &Array1<'_, i32>,
    array_dimension: i32,
    poles: &mut [f64],
    weights: &mut [f64],
    inversion_problem: &mut i32,
) -> Result<(), Failure> {
    let (matrix, upper, lower) = factored_matrix(
        degree,
        flat_knots,
        parameters,
        contact_order_array,
        inversion_problem,
    )?;
    let status: i32 = solve_banded_system_rational(
        &matrix,
        upper,
        lower,
        false,
        array_dimension,
        poles,
        weights,
    )?;
    if status != 0 {
        return Err(Failure::out_of_range(INTERPOLATE));
    }
    Ok(())
}

/// Interpolates 2D or 3D points in place (`BSplCLib::Interpolate` on gp_Pnt/gp_Pnt2d);
/// `bsplclib::interpolate(3, &flat, &params, &orders, &mut points, &mut problem)?`.
pub fn interpolate<P: PolePoint>(
    degree: i32,
    flat_knots: &Array1<'_, f64>,
    parameters: &Array1<'_, f64>,
    contact_order_array: &Array1<'_, i32>,
    poles: &mut Array1<'_, P>,
    inversion_problem: &mut i32,
) -> Result<(), Failure> {
    let mut flat: Vec<f64> = flat_points(poles);
    let dimension: i32 = P::DIMENSION as i32;
    let result = interpolate_flat(
        degree,
        flat_knots,
        parameters,
        contact_order_array,
        dimension,
        &mut flat,
        inversion_problem,
    );
    store_flat_points(&flat, poles);
    result
}

/// Interpolates weighted 2D or 3D points in place (`BSplCLib::Interpolate` on
/// gp_Pnt/gp_Pnt2d with Weights);
/// `bsplclib::interpolate_rational(3, &flat, &params, &orders, &mut points, &mut weights, &mut problem)?`.
pub fn interpolate_rational<P: PolePoint>(
    degree: i32,
    flat_knots: &Array1<'_, f64>,
    parameters: &Array1<'_, f64>,
    contact_order_array: &Array1<'_, i32>,
    poles: &mut Array1<'_, P>,
    weights: &mut Array1<'_, f64>,
    inversion_problem: &mut i32,
) -> Result<(), Failure> {
    let mut flat: Vec<f64> = flat_points(poles);
    let dimension: i32 = P::DIMENSION as i32;
    let result = interpolate_flat_rational(
        degree,
        flat_knots,
        parameters,
        contact_order_array,
        dimension,
        &mut flat,
        weights.as_mut_slice(),
        inversion_problem,
    );
    store_flat_points(&flat, poles);
    result
}
