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
// Rust translation of FoundationClasses/TKMath/PLib/PLib.cxx (HermiteCoefficients, HermiteInterpolate) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Hermite bases and Hermite interpolation in the canonical basis.
use super::access::{array_cell, failure, grid_value};
use crate::math::{Gauss, Matrix, Vector};
use rustycad_tkernel::ncollection::{Array1, Array2, ArrayError};
use rustycad_tkernel::standard::Failure;

// Derivative factors of 1, t, ..., t^5 for orders 0..2 ("portage HP" row-by-row table).
const PATTERN: [[f64; 6]; 3] = [
    [1.0, 1.0, 1.0, 1.0, 1.0, 1.0],
    [0.0, 1.0, 2.0, 3.0, 4.0, 5.0],
    [0.0, 0.0, 2.0, 6.0, 12.0, 20.0],
];

fn matrix_failure(error: ArrayError) -> Failure {
    failure("Standard_RangeError", &format!("math_Matrix: {error}"))
}

/// Builds the Hermite polynomials on [First, Last] as rows of `matrix_coefs`
/// (`PLib::HermiteCoefficients`).
///
/// Row j <= FirstOrder+1 holds H0,j-1; later rows hold H1,k. Returns false for
/// negative orders, |bounds| > 100, or bounds too close;
/// `plib::hermite_coefficients(-1.0, 1.0, 1, 1, &mut matrix)?`.
pub fn hermite_coefficients(
    first_parameter: f64,
    last_parameter: f64,
    first_order: i32,
    last_order: i32,
    matrix_coefs: &mut Matrix<'_>,
) -> Result<bool, Failure> {
    let nb_coeff: i32 = first_order + last_order + 2;
    // Native storage is allocated before the validity tests, so bad sizes raise first.
    let mut coeff: Vector<'static> = Vector::with_bounds(1, nb_coeff)?;
    let mut rhs: Vector<'static> = Vector::filled(1, nb_coeff, 0.0)?;
    let mut mat: Matrix<'static> =
        Matrix::filled(1, nb_coeff, 1, nb_coeff, 0.0).map_err(matrix_failure)?;
    if !hermite_bounds_valid(first_parameter, last_parameter, first_order, last_order) {
        return Ok(false);
    }
    let orders: [i32; 2] = [first_order + 1, last_order + 1];
    fill_hermite_system(
        &mut mat,
        &mut coeff,
        orders,
        [first_parameter, last_parameter],
    );
    let solver: Gauss = Gauss::with_min_pivot(&mat, 1.0e-10)?;
    if !solver.is_done() {
        return Ok(false);
    }
    for ii in 1..=nb_coeff {
        rhs[ii] = 1.0;
        solver.solve_into(&rhs, &mut coeff)?;
        matrix_coefs.set_row(ii, &coeff)?;
        rhs[ii] = 0.0;
    }
    Ok(true)
}

fn hermite_bounds_valid(first: f64, last: f64, first_order: i32, last_order: i32) -> bool {
    if first_order < 0 || last_order < 0 {
        return false;
    }
    let d1: f64 = first.abs();
    let mut d2: f64 = last.abs();
    if d1 > 100.0 || d2 > 100.0 {
        return false;
    }
    d2 += d1;
    if d2 < 0.01 {
        return false;
    }
    let ratio: f64 = (last - first).abs() / d2;
    // The native test rejects only `ratio < 0.01`, so a NaN ratio is accepted.
    ratio >= 0.01 || ratio.is_nan()
}

// Row pp of each side holds the (pp-1)-th derivative of 1, t, ..., t^(n-1) at that bound.
fn fill_hermite_system(
    mat: &mut Matrix<'_>,
    coeff: &mut Vector<'_>,
    orders: [i32; 2],
    bounds: [f64; 2],
) {
    let nb_coeff: i32 = coeff.length();
    let mut offset: i32 = 0;
    for side in 0..2 {
        coeff.init(1.0);
        for pp in 1..=orders[side] {
            let mut prod: f64 = 1.0;
            for jj in pp..=nb_coeff {
                mat[(pp + offset, jj)] = coeff[jj] * prod;
                coeff[jj] *= f64::from(jj - pp);
                prod *= bounds[side];
            }
        }
        offset = orders[0];
    }
}

/// Solves for canonical coefficients meeting value/derivative constraints at both ends
/// (`PLib::HermiteInterpolate`).
///
/// `first_constr(i, j)` is the j-th derivative of component i at First. Orders
/// above 2 overrun the native pattern table and are rejected. A singular system
/// raises StdFail_NotDone from the native solve before `false` can be returned;
/// `plib::hermite_interpolate(1, 0.0, 1.0, 1, 1, &first, &last, &mut coefficients)?`.
#[allow(clippy::too_many_arguments)] // Mirrors the native eight-argument signature.
pub fn hermite_interpolate(
    dimension: i32,
    first_parameter: f64,
    last_parameter: f64,
    first_order: i32,
    last_order: i32,
    first_constr: &Array2<'_, f64>,
    last_constr: &Array2<'_, f64>,
    coefficients: &mut Array1<'_, f64>,
) -> Result<bool, Failure> {
    if first_order > 2 || last_order > 2 {
        return Err(Failure::out_of_range(&format!(
            "PLib::HermiteInterpolate: FirstOrder = {first_order}, LastOrder = {last_order}; expected <= 2"
        )));
    }
    let size: i32 = first_order + last_order + 1;
    let mut system: Matrix<'static> =
        Matrix::with_bounds(0, size, 0, size).map_err(matrix_failure)?;
    fill_interpolation_rows(&mut system, 0, first_order, size, first_parameter)?;
    fill_interpolation_rows(
        &mut system,
        first_order + 1,
        last_order,
        size,
        last_parameter,
    )?;
    let equations: Gauss = Gauss::new(&system)?;
    for idim in 1..=dimension {
        let mut rhs: Vector<'static> = Vector::with_bounds(0, size)?;
        for icol in 0..=first_order {
            rhs[icol] = grid_value(first_constr, idim, icol)?;
        }
        for icol in 0..=last_order {
            rhs[first_order + 1 + icol] = grid_value(last_constr, idim, icol)?;
        }
        equations.solve(&mut rhs)?;
        for icol in 0..=size {
            *array_cell(coefficients, dimension * icol + idim - 1)? = rhs[icol];
        }
    }
    Ok(true)
}

// Row `first_row + k` holds the k-th derivative of the monomials at `parameter`.
fn fill_interpolation_rows(
    system: &mut Matrix<'_>,
    first_row: i32,
    order: i32,
    size: i32,
    parameter: f64,
) -> Result<(), Failure> {
    for irow in 0..=order {
        let mut power: f64 = 1.0;
        for icol in 0..=size {
            *system.change_value(irow + first_row, icol)? =
                PATTERN[irow as usize][icol as usize] * power;
            if irow <= icol {
                power *= parameter;
            }
        }
    }
    Ok(())
}
