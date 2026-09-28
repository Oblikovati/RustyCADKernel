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
// Rust translation of ModelingData/TKG3d/AdvApprox/AdvApprox_ApproxAFunction.cxx/.lxx (result extraction and queries) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The B-spline result of `AdvApprox_ApproxAFunction`: splitting the converted poles
//! per subspace, reducing the piece errors, and the public queries.
use super::approx_a_function::AdvApproxApproxAFunction;
use crate::geom::errors;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::arrays::{at, put};
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::convert::CompPolynomialToPoles;
use rustycad_tkmath::gp::{Point2, Point3};
use std::io;

/// The native result handles, null (`None`) until a conversion succeeds.
#[derive(Default)]
pub(super) struct ConvertedResult {
    pub poles_1d: Option<Array2<'static, f64>>,
    pub poles_2d: Option<Array2<'static, Point2>>,
    pub poles_3d: Option<Array2<'static, Point3>>,
    pub knots: Option<Array1<'static, f64>>,
    pub mults: Option<Array1<'static, i32>>,
    pub degree: i32,
    pub max_error: [Option<Array1<'static, f64>>; 3],
    pub average_error: [Option<Array1<'static, f64>>; 3],
}

// The per-piece error tables (ErrorMax, AverageError) of Approximation.
type PieceErrors<'r> = (&'r Array1<'static, f64>, &'r Array1<'static, f64>);

fn grid<T: Default + Clone>(rows: usize, cols: i32) -> Result<Array2<'static, T>, Failure> {
    Array2::with_bounds(1, rows as i32, 1, cols).map_err(errors::array_bounds)
}

fn value(poles: &Array2<'static, f64>, row: i32, col: i32) -> Result<f64, Failure> {
    poles.value(row, col).copied().map_err(errors::array_bounds)
}

impl ConvertedResult {
    /// Splits the converted poles into the 1D, 2D and 3D tables and reduces the
    /// piece errors: the maximum and the mean over the pieces of each subspace.
    pub fn extract(
        converter: &CompPolynomialToPoles,
        spaces: [i32; 3],
        num_curves: i32,
        piece_errors: PieceErrors<'_>,
    ) -> Result<Self, Failure> {
        let poles: &Array2<'static, f64> = converter.poles()?;
        let mut result = Self {
            knots: Some(converter.knots()?.clone()),
            mults: Some(converter.multiplicities()?.clone()),
            degree: converter.degree(),
            ..Self::default()
        };
        let [n1, n2, n3] = spaces;
        // Error offsets count subspaces; pole offsets count coordinates.
        let (index_2d, index_3d) = (n1, n1 + n2);
        let pole_3d: i32 = n1 + 2 * n2;
        if n1 > 0 {
            result.poles_1d = Some(split_1d(poles, n1)?);
        }
        if n2 > 0 {
            result.poles_2d = Some(split_2d(poles, n2, n1)?);
        }
        if n3 > 0 {
            result.poles_3d = Some(split_3d(poles, n3, pole_3d)?);
        }
        for (dim, (count, offset)) in [(n1, 0), (n2, index_2d), (n3, index_3d)]
            .into_iter()
            .enumerate()
        {
            if count > 0 {
                let total_num_ss: i32 = n1 + n2 + n3;
                let reduce = (count, offset, total_num_ss, num_curves);
                result.max_error[dim] = Some(reduce_max(piece_errors.0, reduce)?);
                result.average_error[dim] = Some(reduce_mean(piece_errors.1, reduce)?);
            }
        }
        Ok(result)
    }
}

fn split_1d(poles: &Array2<'static, f64>, count: i32) -> Result<Array2<'static, f64>, Failure> {
    let mut table: Array2<'static, f64> = grid(poles.nb_rows(), count)?;
    for ii in 1..=poles.nb_rows() as i32 {
        for jj in 1..=count {
            table
                .set_value(ii, jj, value(poles, ii, jj)?)
                .map_err(errors::array_bounds)?;
        }
    }
    Ok(table)
}

fn split_2d(
    poles: &Array2<'static, f64>,
    count: i32,
    index: i32,
) -> Result<Array2<'static, Point2>, Failure> {
    let mut table: Array2<'static, Point2> = grid(poles.nb_rows(), count)?;
    for ii in 1..=poles.nb_rows() as i32 {
        for jj in 1..=count {
            let base: i32 = index + (jj - 1) * 2;
            let point = Point2::new(value(poles, ii, base + 1)?, value(poles, ii, base + 2)?);
            table
                .set_value(ii, jj, point)
                .map_err(errors::array_bounds)?;
        }
    }
    Ok(table)
}

fn split_3d(
    poles: &Array2<'static, f64>,
    count: i32,
    dim_index: i32,
) -> Result<Array2<'static, Point3>, Failure> {
    let mut table: Array2<'static, Point3> = grid(poles.nb_rows(), count)?;
    for ii in 1..=poles.nb_rows() as i32 {
        for jj in 1..=count {
            let base: i32 = dim_index + (jj - 1) * 3;
            let (x, y) = (value(poles, ii, base + 1)?, value(poles, ii, base + 2)?);
            let point = Point3::new(x, y, value(poles, ii, base + 3)?);
            table
                .set_value(ii, jj, point)
                .map_err(errors::array_bounds)?;
        }
    }
    Ok(table)
}

// (subspace count, subspace offset, TotalNumSS, NumCurves).
type Reduce = (i32, i32, i32, i32);

fn reduce_max(
    errors: &Array1<'static, f64>,
    (count, offset, total, curves): Reduce,
) -> Result<Array1<'static, f64>, Failure> {
    let mut table: Array1<'static, f64> =
        Array1::with_bounds(1, count).map_err(errors::array_bounds)?;
    for jj in 1..=count {
        let mut error_value: f64 = 0.0;
        for ii in 1..=curves {
            error_value = std_max(at(errors, (ii - 1) * total + offset + jj)?, error_value);
        }
        put(&mut table, jj, error_value)?;
    }
    Ok(table)
}

fn reduce_mean(
    errors: &Array1<'static, f64>,
    (count, offset, total, curves): Reduce,
) -> Result<Array1<'static, f64>, Failure> {
    let mut table: Array1<'static, f64> =
        Array1::with_bounds(1, count).map_err(errors::array_bounds)?;
    for jj in 1..=count {
        let mut error_value: f64 = 0.0;
        for ii in 1..=curves {
            error_value += at(errors, (ii - 1) * total + offset + jj)?;
        }
        error_value /= f64::from(curves);
        put(&mut table, jj, error_value)?;
    }
    Ok(table)
}

fn null_object(member: &str) -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        &format!(
            "AdvApprox_ApproxAFunction::{member}: no result; expected a successful approximation"
        ),
        "",
    )
}

// Copies column `index` of a pole table into `target` over its own bounds.
fn copy_column<T: Copy>(
    table: Option<&Array2<'static, T>>,
    index: i32,
    target: &mut Array1<'_, T>,
    member: &str,
) -> Result<(), Failure> {
    let table: &Array2<'static, T> = table.ok_or_else(|| null_object(member))?;
    for ii in target.lower()..=target.upper() {
        let pole: T = *table.value(ii, index).map_err(errors::array_bounds)?;
        put(target, ii, pole)?;
    }
    Ok(())
}

fn dimension_slot(dimension: i32, member: &str) -> Result<usize, Failure> {
    if !(1..=3).contains(&dimension) {
        return Err(errors::out_of_range(&format!(
            "AdvApprox_ApproxAFunction::{member}: Dimension = {dimension}; expected 1, 2 or 3"
        )));
    }
    Ok(dimension as usize - 1)
}

impl AdvApproxApproxAFunction {
    /// Whether every subspace met its tolerance (`IsDone`); `approx.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }
    /// Whether a B-spline was built (`HasResult`); `approx.has_result()`.
    pub fn has_result(&self) -> bool {
        self.has_result
    }
    /// The 1D poles `(1..NbPoles, 1..Num1DSS)`, `None` when null (`Poles1d()`); `approx.poles_1d()`.
    pub fn poles_1d(&self) -> Option<&Array2<'static, f64>> {
        self.result.poles_1d.as_ref()
    }
    /// The 2D poles (`Poles2d()`); `approx.poles_2d()`.
    pub fn poles_2d(&self) -> Option<&Array2<'static, Point2>> {
        self.result.poles_2d.as_ref()
    }
    /// The 3D poles (`Poles()`); `approx.poles()`.
    pub fn poles(&self) -> Option<&Array2<'static, Point3>> {
        self.result.poles_3d.as_ref()
    }
    /// Pole count of the non-periodic B-spline, 0 without a result (`NbPoles`); `approx.nb_poles()?`.
    pub fn nb_poles(&self) -> Result<i32, Failure> {
        if !(self.done || self.has_result) {
            return Ok(0);
        }
        let mults = self
            .result
            .mults
            .as_ref()
            .ok_or_else(|| null_object("NbPoles"))?;
        bsplclib::nb_poles(self.result.degree, false, mults)
    }
    /// Copies the 1D poles of subspace `index` into `p` over its bounds (`Poles1d(Index, P)`);
    /// `approx.poles_1d_of(1, &mut p)?`.
    pub fn poles_1d_of(&self, index: i32, p: &mut Array1<'_, f64>) -> Result<(), Failure> {
        copy_column(self.poles_1d(), index, p, "Poles1d")
    }
    /// Copies the 2D poles of subspace `index` (`Poles2d(Index, P)`); `approx.poles_2d_of(1, &mut p)?`.
    pub fn poles_2d_of(&self, index: i32, p: &mut Array1<'_, Point2>) -> Result<(), Failure> {
        copy_column(self.poles_2d(), index, p, "Poles2d")
    }
    /// Copies the 3D poles of subspace `index` (`Poles(Index, P)`); `approx.poles_of(1, &mut p)?`.
    pub fn poles_of(&self, index: i32, p: &mut Array1<'_, Point3>) -> Result<(), Failure> {
        copy_column(self.poles(), index, p, "Poles")
    }
    /// The B-spline degree, 0 without a result (`Degree`); `approx.degree()`.
    pub fn degree(&self) -> i32 {
        self.result.degree
    }
    /// The knot count; Standard_NullObject without a result (`NbKnots`); `approx.nb_knots()?`.
    pub fn nb_knots(&self) -> Result<i32, Failure> {
        let knots = self.knots().ok_or_else(|| null_object("NbKnots"))?;
        Ok(knots.len() as i32)
    }
    /// Number of subspaces of dimension 1, 2 or 3; Standard_ConstructionError
    /// otherwise (`NumSubSpaces`); `approx.num_sub_spaces(3)?`.
    pub fn num_sub_spaces(&self, dimension: i32) -> Result<i32, Failure> {
        if !(1..=3).contains(&dimension) {
            return Err(errors::construction(&format!(
                "AdvApprox_ApproxAFunction::NumSubSpaces: Dimension = {dimension}; expected 1, 2 or 3"
            )));
        }
        Ok(self.num_sub_spaces[dimension as usize - 1])
    }
    /// The knots (`Knots`); `approx.knots()`.
    pub fn knots(&self) -> Option<&Array1<'static, f64>> {
        self.result.knots.as_ref()
    }
    /// The multiplicities (`Multiplicities`); `approx.multiplicities()`.
    pub fn multiplicities(&self) -> Option<&Array1<'static, i32>> {
        self.result.mults.as_ref()
    }
    /// Maximum errors of the subspaces of `dimension`, `None` when null;
    /// Standard_OutOfRange outside 1..3 (`MaxError(Dimension)`); `approx.max_errors(3)?`.
    pub fn max_errors(&self, dimension: i32) -> Result<Option<&Array1<'static, f64>>, Failure> {
        Ok(self.result.max_error[dimension_slot(dimension, "MaxError")?].as_ref())
    }
    /// Average errors of the subspaces of `dimension` (`AverageError(Dimension)`);
    /// `approx.average_errors(3)?`.
    pub fn average_errors(&self, dimension: i32) -> Result<Option<&Array1<'static, f64>>, Failure> {
        Ok(self.result.average_error[dimension_slot(dimension, "AverageError")?].as_ref())
    }
    /// Maximum error of subspace `index` of `dimension` (`MaxError(Dimension, Index)`);
    /// `approx.max_error(3, 1)?`.
    pub fn max_error(&self, dimension: i32, index: i32) -> Result<f64, Failure> {
        let table = self
            .max_errors(dimension)?
            .ok_or_else(|| null_object("MaxError"))?;
        at(table, index)
    }
    /// Average error of subspace `index` of `dimension` (`AverageError(Dimension, Index)`);
    /// `approx.average_error(3, 1)?`.
    pub fn average_error(&self, dimension: i32, index: i32) -> Result<f64, Failure> {
        let table = self
            .average_errors(dimension)?
            .ok_or_else(|| null_object("AverageError"))?;
        at(table, index)
    }
    /// Writes the native text: a title and the maximum errors of each dimension
    /// (`Dump`); `approx.dump(&mut std::io::stdout())?`.
    pub fn dump(&self, output: &mut impl io::Write) -> io::Result<()> {
        writeln!(output, "Dump of ApproxAFunction")?;
        for (dimension, label) in [(1, "1d"), (2, "2d"), (3, "3d")] {
            let count: i32 = self.num_sub_spaces[dimension as usize - 1];
            if count <= 0 {
                continue;
            }
            writeln!(output, "Error(s) {label} = ")?;
            for ii in 1..=count {
                let error: f64 = self.max_error(dimension, ii).map_err(io::Error::other)?;
                writeln!(output, "   {}", super::stream_real(error))?;
            }
        }
        Ok(())
    }
}
