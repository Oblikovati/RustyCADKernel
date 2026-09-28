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
// Rust translation of FoundationClasses/TKMath/BSplCLib/BSplCLib_2.cxx (BuildBSpMatrix, FactorBandedMatrix), BSplCLib.cxx (SolveBandedSystem) and BSplCLib_CurveComputation.pxx (point overloads) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Banded collocation matrices of B-spline bases and their LU solution.
//!
//! A band row `i` stores columns `i-LowerBandWidth..i+UpperBandWidth` of the full
//! matrix in columns `1..UpperBandWidth+LowerBandWidth+1`.
use super::basis::eval_bspline_basis;
use super::native::real_small;
use super::{MAX_DEGREE, flat_points, store_flat_points};
use crate::math::Matrix;
use crate::plib::PolePoint;
use crate::plib::access::{array_value, require_length};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;

const BUILD: &str = "BSplCLib::BuildBSpMatrix";
const FACTOR: &str = "BSplCLib::FactorBandedMatrix";
const SOLVE: &str = "BSplCLib::SolveBandedSystem";

/// Builds the banded matrix of basis derivatives `B_j^(n_i)(t_i)` for interpolation
/// (`BSplCLib::BuildBSpMatrix`); returns `(status, UpperBandWidth, LowerBandWidth)`.
///
/// Status 1: the matrix is not `Parameters bounds x 1..2*Degree+1`; 2: the basis could
/// not be evaluated. Contact orders outside `0..=Degree`, whose rows the native code
/// reads from an unwritten buffer, are rejected;
/// `bsplclib::build_bsp_matrix(&params, &orders, &flat, 3, &mut matrix)?`.
pub fn build_bsp_matrix(
    parameters: &Array1<'_, f64>,
    order_array: &Array1<'_, i32>,
    flat_knots: &Array1<'_, f64>,
    degree: i32,
    matrix: &mut Matrix<'_>,
) -> Result<(i32, i32, i32), Failure> {
    let order: i32 = degree + 1;
    let band_width: i32 = 2 * degree + 1;
    if matrix.lower_row() != parameters.lower()
        || matrix.upper_row() != parameters.upper()
        || matrix.lower_col() != 1
        || matrix.upper_col() != band_width
    {
        return Ok((1, degree, degree));
    }
    if order > MAX_DEGREE + 1 {
        return Err(Failure::out_of_range(&format!(
            "{BUILD}: Degree = {degree}; expected <= {MAX_DEGREE}"
        )));
    }
    let max_order: i32 = MAX_DEGREE + 1;
    let mut basis: Matrix<'static> = Matrix::filled(1, max_order, 1, max_order, 0.0)
        .map_err(|error| Failure::out_of_range(&error.to_string()))?;
    matrix.init(0.0);
    let columns: usize = matrix.col_number() as usize;
    for i in parameters.lower()..=parameters.upper() {
        let contact: i32 = array_value(order_array, i)?;
        if !(0..order).contains(&contact) {
            return Err(Failure::out_of_range(&format!(
                "{BUILD}: ContactOrderArray({i}) = {contact}; expected 0..={degree}"
            )));
        }
        let mut first: i32 = 0;
        let parameter: f64 = array_value(parameters, i)?;
        if eval_bspline_basis(
            contact, order, flat_knots, parameter, &mut first, &mut basis, false,
        )? != 0
        {
            return Ok((2, degree, degree));
        }
        let index: i32 = degree + 1 + first - i;
        let row: usize = (i - matrix.lower_row()) as usize * columns;
        let source: usize = contact as usize * max_order as usize;
        for j in 0..order {
            let column: i32 = index + j - 1;
            if !(0..band_width).contains(&column) {
                return Err(Failure::out_of_range(&format!(
                    "{BUILD}: basis column {column} of row {i} outside the band 0..{band_width}"
                )));
            }
            matrix.cells_mut()[row + column as usize] = basis.cells()[source + j as usize];
        }
    }
    Ok((0, degree, degree))
}

/// Factors a banded matrix in place into LU form, the multipliers of L stored in
/// the band (`BSplCLib::FactorBandedMatrix`); returns `(status, PivotIndexProblem)`.
///
/// Status 1 reports a pivot not above RealSmall() at row PivotIndexProblem. Do not use
/// it on totally positive matrices such as Schoenberg collocation. Quirk kept: the first
/// band column is computed from the absolute row index, as if rows started at 1;
/// `bsplclib::factor_banded_matrix(&mut matrix, 3, 3)?`.
pub fn factor_banded_matrix(
    matrix: &mut Matrix<'_>,
    upper_band_width: i32,
    lower_band_width: i32,
) -> Result<(i32, i32), Failure> {
    let band = Band::new(FACTOR, matrix, upper_band_width, lower_band_width)?;
    let cells: &mut [f64] = matrix.cells_mut();
    for i in band.lower_row + 1..=band.upper_row {
        let row_i: i64 = band.row_start(i);
        for j in 1.max(lower_band_width - i + 2)..=lower_band_width {
            let index: i32 = i - lower_band_width + j - 1;
            let row_index: i64 = band.row_start(index);
            let pivot: f64 = band.get(cells, row_index + i64::from(lower_band_width))?;
            if pivot.abs() <= real_small() {
                return Ok((1, index));
            }
            let inverse: f64 = -1.0 / pivot;
            let multiplier: f64 = band.get(cells, row_i + i64::from(j - 1))? * inverse;
            band.set(cells, row_i + i64::from(j - 1), multiplier)?;
            let max_index: i32 = upper_band_width + lower_band_width + 1 + index - i;
            for k in j + 1..=max_index {
                let target: i64 = row_i + i64::from(k - 1);
                let term: f64 =
                    multiplier * band.get(cells, row_index + i64::from(k + i - index - 1))?;
                let value: f64 = band.get(cells, target)? + term;
                band.set(cells, target, value)?;
            }
        }
    }
    Ok((0, 0))
}

// Row-major band storage addressed like the native `&Matrix(LowerRow, 1)` pointer.
struct Band {
    lower_row: i32,
    upper_row: i32,
    columns: i64,
}

impl Band {
    fn new(function: &str, matrix: &Matrix<'_>, upper: i32, lower: i32) -> Result<Self, Failure> {
        if matrix.lower_col() != 1 || matrix.row_number() == 0 {
            return Err(Failure::out_of_range(&format!(
                "{function}: the band must start at column 1 (found {}) and hold rows; \
                 band widths {upper}/{lower}",
                matrix.lower_col()
            )));
        }
        Ok(Self {
            lower_row: matrix.lower_row(),
            upper_row: matrix.upper_row(),
            columns: i64::from(matrix.col_number()),
        })
    }

    fn row_start(&self, row: i32) -> i64 {
        i64::from(row - self.lower_row) * self.columns
    }

    fn get(&self, cells: &[f64], offset: i64) -> Result<f64, Failure> {
        crate::plib::access::buffer_value("BSplCLib banded matrix", cells, offset)
    }

    fn set(&self, cells: &mut [f64], offset: i64, value: f64) -> Result<(), Failure> {
        let slot: &mut f64 = usize::try_from(offset)
            .ok()
            .and_then(|offset| cells.get_mut(offset))
            .ok_or_else(|| {
                Failure::out_of_range(&format!(
                    "BSplCLib banded matrix: cell {offset} outside the matrix"
                ))
            })?;
        *slot = value;
        Ok(())
    }
}

/// Solves `Matrix.X = B` for a factored band matrix, B and X being rows of
/// `array_dimension` values in `array` (`BSplCLib::SolveBandedSystem`); returns 0, or 1
/// on a bad band shape or a diagonal not above 1e-16 (PRO18577).
///
/// Quirk kept: right-hand side rows are addressed from the absolute row index minus 1;
/// `bsplclib::solve_banded_system(&matrix, 3, 3, 2, &mut values)?`.
pub fn solve_banded_system(
    matrix: &Matrix<'_>,
    upper_band_width: i32,
    lower_band_width: i32,
    array_dimension: i32,
    array: &mut [f64],
) -> Result<i32, Failure> {
    if matrix.lower_col() != 1 || matrix.upper_col() != upper_band_width + lower_band_width + 1 {
        return Ok(1);
    }
    let band = Band::new(SOLVE, matrix, upper_band_width, lower_band_width)?;
    let rhs = RightHandSide {
        dimension: array_dimension,
    };
    let cells: &[f64] = matrix.cells();
    for ii in band.lower_row + 1..=band.upper_row {
        let row: i64 = band.row_start(ii);
        for jj in (ii - lower_band_width).max(band.lower_row)..ii {
            let coeff: f64 = band.get(cells, row + i64::from(jj - ii + lower_band_width))?;
            rhs.update(array, ii, jj, coeff, 1.0)?;
        }
    }
    for ii in (band.lower_row..=band.upper_row).rev() {
        let row: i64 = band.row_start(ii);
        for jj in (ii + 1..=(ii + upper_band_width).min(band.upper_row)).rev() {
            let coeff: f64 = band.get(cells, row + i64::from(jj - ii + lower_band_width))?;
            rhs.update(array, ii, jj, coeff, -1.0)?;
        }
        // Fixing a bug PRO18577 to avoid division by zero.
        let divisor: f64 = band.get(cells, row + i64::from(lower_band_width))?;
        if divisor.abs() <= 1.0e-16 {
            return Ok(1);
        }
        rhs.scale(array, ii, 1.0 / divisor)?;
    }
    Ok(0)
}

struct RightHandSide {
    dimension: i32,
}

impl RightHandSide {
    fn range(&self, array: &[f64], row: i32) -> Result<std::ops::Range<usize>, Failure> {
        let start: i64 = i64::from(row - 1) * i64::from(self.dimension);
        let start: usize = usize::try_from(start).map_err(|_| {
            Failure::out_of_range(&format!("{SOLVE}: row {row} precedes the Array buffer"))
        })?;
        let end: usize = start + self.dimension.max(0) as usize;
        require_length(SOLVE, "Array", array.len(), end)?;
        Ok(start..end)
    }

    // Row ii += sign * coeff * row jj; the sign selects the native += or -= form.
    fn update(
        &self,
        array: &mut [f64],
        ii: i32,
        jj: i32,
        coeff: f64,
        sign: f64,
    ) -> Result<(), Failure> {
        let target = self.range(array, ii)?;
        let source = self.range(array, jj)?;
        for (t, s) in target.zip(source) {
            let term: f64 = array[s] * coeff;
            if sign > 0.0 {
                array[t] += term;
            } else {
                array[t] -= term;
            }
        }
        Ok(())
    }

    fn scale(&self, array: &mut [f64], ii: i32, inverse: f64) -> Result<(), Failure> {
        for t in self.range(array, ii)? {
            array[t] *= inverse;
        }
        Ok(())
    }
}

/// Solves the band system for weighted rows (`BSplCLib::SolveBandedSystem` with
/// HomogenousFlag, Poles and Weights); returns 0, 1 (bad band), 2 (poles) or 3 (weights).
///
/// Without `homogeneous_flag` the poles are multiplied by the weights first and divided
/// by the solved weights afterwards;
/// `bsplclib::solve_banded_system_rational(&m, 3, 3, false, 2, &mut poles, &mut weights)?`.
pub fn solve_banded_system_rational(
    matrix: &Matrix<'_>,
    upper_band_width: i32,
    lower_band_width: i32,
    homogeneous_flag: bool,
    array_dimension: i32,
    poles: &mut [f64],
    weights: &mut [f64],
) -> Result<i32, Failure> {
    if matrix.lower_col() != 1 || matrix.upper_col() != upper_band_width + lower_band_width + 1 {
        return Ok(1);
    }
    let rows: usize = matrix.row_number().max(0) as usize;
    let dim: usize = array_dimension.max(0) as usize;
    require_length(SOLVE, "Poles", poles.len(), rows * dim)?;
    require_length(SOLVE, "Weights", weights.len(), rows)?;
    if !homogeneous_flag {
        for ii in 0..rows {
            for kk in 0..dim {
                poles[ii * dim + kk] *= weights[ii];
            }
        }
    }
    if solve_banded_system(
        matrix,
        upper_band_width,
        lower_band_width,
        array_dimension,
        poles,
    )? != 0
    {
        return Ok(2);
    }
    if solve_banded_system(matrix, upper_band_width, lower_band_width, 1, weights)? != 0 {
        return Ok(3);
    }
    if !homogeneous_flag {
        for ii in 0..rows {
            let inverse: f64 = 1.0 / weights[ii];
            for kk in 0..dim {
                poles[ii * dim + kk] *= inverse;
            }
        }
    }
    Ok(0)
}

/// Solves the band system for 2D or 3D points (`BSplCLib::SolveBandedSystem` on
/// gp_Pnt/gp_Pnt2d arrays); `bsplclib::solve_banded_system_points(&m, 3, 3, &mut points)?`.
pub fn solve_banded_system_points<P: PolePoint>(
    matrix: &Matrix<'_>,
    upper_band_width: i32,
    lower_band_width: i32,
    array: &mut Array1<'_, P>,
) -> Result<i32, Failure> {
    let mut flat: Vec<f64> = flat_points(array);
    let dimension: i32 = P::DIMENSION as i32;
    let status: i32 = solve_banded_system(
        matrix,
        upper_band_width,
        lower_band_width,
        dimension,
        &mut flat,
    )?;
    store_flat_points(&flat, array);
    Ok(status)
}

/// Solves the band system for weighted 2D or 3D points (`BSplCLib::SolveBandedSystem`
/// with HomogeneousFlag on gp_Pnt/gp_Pnt2d arrays);
/// `bsplclib::solve_banded_system_points_rational(&m, 3, 3, false, &mut points, &mut weights)?`.
pub fn solve_banded_system_points_rational<P: PolePoint>(
    matrix: &Matrix<'_>,
    upper_band_width: i32,
    lower_band_width: i32,
    homogeneous_flag: bool,
    array: &mut Array1<'_, P>,
    weights: &mut Array1<'_, f64>,
) -> Result<i32, Failure> {
    let mut flat: Vec<f64> = flat_points(array);
    let status: i32 = solve_banded_system_rational(
        matrix,
        upper_band_width,
        lower_band_width,
        homogeneous_flag,
        P::DIMENSION as i32,
        &mut flat,
        weights.as_mut_slice(),
    )?;
    store_flat_points(&flat, array);
    Ok(status)
}
