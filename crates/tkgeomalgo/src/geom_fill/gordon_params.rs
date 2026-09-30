// Copyright (c) 2025 OPEN CASCADE SAS
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
// Rust translation of GeomFill_GordonUtilities.pxx (the parameter helpers,
// exposePeriod, makeUnitCurves, columnMeans and rowMeans) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The parameter helpers of the Gordon network preparation and the contact tables.
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::stl;
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomCurve};
use rustycad_tkmath::bsplclib;

/// A table of network parameters indexed by (profile, guide) from 0, the native
/// `NCollection_Array2<double>` (1-based, rows = profiles) read through its `At`
/// offsets and through the `math_Matrix` views that share its storage.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct ParamTable {
    pub(crate) rows: usize,
    pub(crate) cols: usize,
    values: Vec<f64>,
}

impl ParamTable {
    /// A zero table of `rows` profiles and `cols` guides.
    pub(crate) fn new(rows: usize, cols: usize) -> Self {
        Self {
            rows,
            cols,
            values: vec![0.0; rows * cols],
        }
    }
    /// `At(row, col)` (0-based offsets).
    pub(crate) fn at(&self, row: usize, col: usize) -> f64 {
        self.values[row * self.cols + col]
    }
    /// `ChangeAt(row, col) = value`.
    pub(crate) fn set(&mut self, row: usize, col: usize, value: f64) {
        self.values[row * self.cols + col] = value;
    }
    /// `columnMeans(toMatrix(table))`: the mean of each column, summed from 0 in row
    /// order (`Transposed() * ones`) then divided by the row count.
    pub(crate) fn column_means(&self) -> Vec<f64> {
        (0..self.cols)
            .map(|col| {
                let sum: f64 = (0..self.rows).fold(0.0, |sum, row| sum + self.at(row, col) * 1.0);
                sum / self.rows as f64
            })
            .collect()
    }
    /// `rowMeans(toMatrix(table))`: the mean of each row, summed from 0 in column order.
    pub(crate) fn row_means(&self) -> Vec<f64> {
        (0..self.rows)
            .map(|row| {
                let sum: f64 = (0..self.cols).fold(0.0, |sum, col| sum + self.at(row, col) * 1.0);
                sum / self.cols as f64
            })
            .collect()
    }
}

/// `interpolationDegree()`: 3, within the B-spline degree limit.
pub(crate) fn interpolation_degree() -> i32 {
    3.min(GeomBSplineCurve::max_degree())
}

/// `parameterResolution(First, Last)`: Precision::PIntersection of the interval
/// length (at least 1).
pub(crate) fn parameter_resolution(the_first: f64, the_last: f64) -> f64 {
    precision::p_intersection_with_speed((the_last - the_first).abs().max(1.0))
}

/// `boundaryParameterResolution(First, Last, Tolerance)`: the network tolerance (at
/// least Precision::Approximation()) in parameter space, scaled by the range.
pub(crate) fn boundary_parameter_resolution(
    the_first: f64,
    the_last: f64,
    the_tolerance: f64,
) -> f64 {
    let a_range: f64 = (the_last - the_first).abs().max(1.0);
    precision::parametric(the_tolerance.max(precision::approximation())) * a_range
}

/// `appendUnitParameter`: the parameter clamped to `[0, 1]`.
pub(crate) fn append_unit_parameter(the_parameters: &mut Vec<f64>, the_parameter: f64) {
    the_parameters.push(0.0_f64.max(1.0_f64.min(the_parameter)));
}

/// `compactParameters`: sorted (libstdc++ `std::sort`), then the values within
/// parameterResolution(0, 1) of the last kept one removed.
pub(crate) fn compact_parameters(the_parameters: &mut Vec<f64>) {
    if the_parameters.is_empty() {
        return;
    }
    stl::sort_by(the_parameters, |a, b| a < b);
    let a_tol: f64 = parameter_resolution(0.0, 1.0);
    let mut a_write_idx: usize = 0;
    for a_read_idx in 1..the_parameters.len() {
        if (the_parameters[a_read_idx] - the_parameters[a_write_idx]).abs() > a_tol {
            a_write_idx += 1;
            the_parameters[a_write_idx] = the_parameters[a_read_idx];
        }
    }
    the_parameters.truncate(a_write_idx + 1);
}

/// `addIntervalMidpoints`: the midpoint of the widest interval (the first of equal
/// ones) added until `the_min_size` parameters, unless every gap is within the
/// resolution.
pub(crate) fn add_interval_midpoints(the_parameters: &mut Vec<f64>, the_min_size: usize) {
    compact_parameters(the_parameters);
    while the_parameters.len() < the_min_size && the_parameters.len() > 1 {
        let (mut a_split_idx, mut a_max_gap): (usize, f64) = (0, -1.0);
        for a_param_idx in 1..the_parameters.len() {
            let a_gap: f64 = the_parameters[a_param_idx] - the_parameters[a_param_idx - 1];
            if a_gap > a_max_gap {
                a_split_idx = a_param_idx;
                a_max_gap = a_gap;
            }
        }
        if a_max_gap <= parameter_resolution(0.0, 1.0) {
            return;
        }
        let midpoint: f64 = 0.5 * (the_parameters[a_split_idx - 1] + the_parameters[a_split_idx]);
        append_unit_parameter(the_parameters, midpoint);
        compact_parameters(the_parameters);
    }
}

/// `exposePeriod`: a periodic curve segmented on its own range and made not periodic
/// in place (false when the range is empty or an exception is raised).
pub(crate) fn expose_period(the_curve: &Handle<GeomBSplineCurve>) -> Result<bool, Failure> {
    let curve = the_curve.get().ok_or_else(null_curve)?;
    if !curve.is_periodic() {
        return Ok(true);
    }
    let segmented = || -> Result<bool, Failure> {
        let (a_first, a_last) = (curve.first_parameter(), curve.last_parameter());
        if a_last <= a_first + precision::p_confusion() {
            return Ok(false);
        }
        curve.segment(a_first, a_last, precision::p_confusion())?;
        if curve.is_periodic() {
            curve.set_not_periodic()?;
        }
        Ok(true)
    };
    // The native try/catch turns an exception into false.
    match segmented() {
        Ok(true) => Ok(!curve.is_periodic()),
        _ => Ok(false),
    }
}

/// `makeUnitCurves`: every curve not periodic and reparametrized onto `[0, 1]` (false
/// for a null curve or a period that cannot be exposed).
pub(crate) fn make_unit_curves(the_curves: &[Handle<GeomBSplineCurve>]) -> Result<bool, Failure> {
    for a_curve in the_curves {
        if a_curve.is_null() || !expose_period(a_curve)? {
            return Ok(false);
        }
        let curve = a_curve.get().ok_or_else(null_curve)?;
        let mut a_knots: Array1<'static, f64> = curve.knots();
        bsplclib::reparametrize(0.0, 1.0, &mut a_knots)?;
        curve.set_knots(&a_knots)?;
    }
    Ok(true)
}

/// The Standard_NullObject of a null working curve (a null dereference natively).
pub(crate) fn null_curve() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "GeomFill_Gordon - null working curve; expected a Geom_BSplineCurve",
        "",
    )
}
