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
// Rust translation of FoundationClasses/TKMath/PLib/PLib.cxx (CoefficientsPoles on surfaces, PRO5852) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Conversion of a bivariate coefficient grid to Bezier surface poles.
//!
//! Rows (U) are converted first from the coefficients, then columns (V) in place;
//! since the 1996 PRO5852 change the caller is responsible for the U/V order.
use super::access::{failure, grid_cell, grid_value};
use super::binomial::bin;
use crate::gp::Point3;
use rustycad_tkernel::ncollection::Array2;
use rustycad_tkernel::standard::Failure;

type WeightGrids<'r, 'a, 'b> = Option<(&'r Array2<'a, f64>, &'r mut Array2<'b, f64>)>;

/// Converts a coefficient grid to poles over the poles' bounds (`PLib::CoefficientsPoles` on Array2);
/// `plib::coefficients_poles_grid(&coefs, None, &mut poles, None)?`.
pub fn coefficients_poles_grid(
    coefs: &Array2<'_, Point3>,
    wcoefs: Option<&Array2<'_, f64>>,
    poles: &mut Array2<'_, Point3>,
    weights: Option<&mut Array2<'_, f64>>,
) -> Result<(), Failure> {
    let mut pair: WeightGrids = match (wcoefs, weights) {
        (None, _) => None,
        (Some(source), Some(target)) => Some((source, target)),
        (Some(_), None) => {
            return Err(failure(
                "Standard_Failure",
                "PLib::CoefficientsPoles: weighted grid without output weights",
            ));
        }
    };
    for row in poles.lower_row()..=poles.upper_row() {
        convert_row(row, coefs, poles, &mut pair)?;
    }
    for col in poles.lower_col()..=poles.upper_col() {
        convert_column(col, poles, &mut pair)?;
    }
    let Some((_, target)) = pair else {
        return Ok(());
    };
    for row in poles.lower_row()..=poles.upper_row() {
        for col in poles.lower_col()..=poles.upper_col() {
            let w: f64 = grid_value(target, row, col)?;
            let pole: &mut Point3 = grid_cell(poles, row, col)?;
            *pole = divided(*pole, w);
        }
    }
    Ok(())
}

fn divided(point: Point3, divisor: f64) -> Point3 {
    Point3::new(
        point.x() / divisor,
        point.y() / divisor,
        point.z() / divisor,
    )
}

fn summed(point: Point3, other: Point3) -> Point3 {
    Point3::new(
        point.x() + other.x(),
        point.y() + other.y(),
        point.z() + other.z(),
    )
}

fn convert_row(
    row: i32,
    coefs: &Array2<'_, Point3>,
    poles: &mut Array2<'_, Point3>,
    pair: &mut WeightGrids,
) -> Result<(), Failure> {
    let (lc, uc) = (poles.lower_col(), poles.upper_col());
    let row_length: i32 = poles.nb_columns() as i32;
    *grid_cell(poles, row, lc)? = grid_value(coefs, row, lc)?;
    if let Some((source, target)) = pair {
        *grid_cell(target, row, lc)? = grid_value(source, row, lc)?;
    }
    for col in lc + 1..uc {
        let cnp: f64 = bin(row_length - 1, col - lc)?;
        *grid_cell(poles, row, col)? = divided(grid_value(coefs, row, col)?, cnp);
        if let Some((source, target)) = pair {
            *grid_cell(target, row, col)? = grid_value(source, row, col)? / cnp;
        }
    }
    *grid_cell(poles, row, uc)? = grid_value(coefs, row, uc)?;
    if let Some((source, target)) = pair {
        *grid_cell(target, row, uc)? = grid_value(source, row, uc)?;
    }
    for i1 in 1..row_length {
        for i2 in (lc + i1..=uc).rev() {
            let cells: [(i32, i32); 2] = [(row, i2), (row, i2 - 1)];
            add_previous(poles, pair, cells)?;
        }
    }
    Ok(())
}

fn convert_column(
    col: i32,
    poles: &mut Array2<'_, Point3>,
    pair: &mut WeightGrids,
) -> Result<(), Failure> {
    let (lr, ur) = (poles.lower_row(), poles.upper_row());
    let col_length: i32 = poles.nb_rows() as i32;
    for row in lr + 1..ur {
        let cnp: f64 = bin(col_length - 1, row - lr)?;
        let pole: &mut Point3 = grid_cell(poles, row, col)?;
        *pole = divided(*pole, cnp);
        if let Some((_, target)) = pair {
            *grid_cell(target, row, col)? /= cnp;
        }
    }
    for i1 in 1..col_length {
        for i2 in (lr + i1..=ur).rev() {
            add_previous(poles, pair, [(i2, col), (i2 - 1, col)])?;
        }
    }
    Ok(())
}

// cells[0] += cells[1] for the pole and, when rational, for its weight.
fn add_previous(
    poles: &mut Array2<'_, Point3>,
    pair: &mut WeightGrids,
    cells: [(i32, i32); 2],
) -> Result<(), Failure> {
    let [(row, col), (previous_row, previous_col)] = cells;
    let previous: Point3 = grid_value(poles, previous_row, previous_col)?;
    let pole: &mut Point3 = grid_cell(poles, row, col)?;
    *pole = summed(*pole, previous);
    if let Some((_, target)) = pair {
        let previous_weight: f64 = grid_value(target, previous_row, previous_col)?;
        *grid_cell(target, row, col)? += previous_weight;
    }
    Ok(())
}
