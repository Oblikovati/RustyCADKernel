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
// Rust translation of FoundationClasses/TKMath/BSplSLib/BSplSLib.cxx (Iso, Reverse, IsRational) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Iso curves, reversal of a pole grid, and rationality tests.
use crate::bsplclib::native::epsilon;
use crate::bsplclib::{build_knots, eval, locate_parameter_for_eval, pole_index};
use crate::gp::Point3;
use crate::plib::access::{array_cell, grid_cell, grid_value, non_negative};
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Failure;

/// The poles of the iso curve at `param` of the U (`is_u`) or V parameter
/// (`BSplSLib::Iso`); `cweights` receive the weights, or 1 for a non-rational surface.
///
/// A weighted surface needs `cweights`, which natively is dereferenced blindly;
/// `bsplslib::iso(0.5, true, &poles, None, &knots, Some(&mults), 3, false, &mut cpoles, None)?`.
pub fn iso(
    param: f64,
    is_u: bool,
    poles: &Array2<'_, Point3>,
    weights: Option<&Array2<'_, f64>>,
    knots: &Array1<'_, f64>,
    mults: Option<&Array1<'_, i32>>,
    degree: i32,
    periodic: bool,
    cpoles: &mut Array1<'_, Point3>,
    mut cweights: Option<&mut Array1<'_, f64>>,
) -> Result<(), Failure> {
    let deg: usize = non_negative("BSplSLib::Iso", "Degree", degree)?;
    let (mut index, mut u): (i32, f64) = (0, param);
    locate_parameter_for_eval(degree, knots, mults, u, periodic, &mut index, &mut u)?;
    let mut local_knots: Vec<f64> = vec![0.0; 2 * deg];
    build_knots(degree, index, periodic, knots, mults, &mut local_knots)?;
    index = match mults {
        None => index - (knots.lower() + degree),
        Some(mults) => pole_index(degree, index, periodic, mults)?,
    };
    let rational: bool = weights.is_some();
    let dim: usize = if rational { 4 } else { 3 };
    let (f1, l1, f2, l2) = if is_u {
        (
            poles.lower_row(),
            poles.upper_row(),
            poles.lower_col(),
            poles.upper_col(),
        )
    } else {
        (
            poles.lower_col(),
            poles.upper_col(),
            poles.lower_row(),
            poles.upper_row(),
        )
    };
    let mut local: Vec<f64> = Vec::with_capacity((deg + 1) * (l2 - f2 + 1).max(0) as usize * dim);
    let mut at: i32 = index + f1;
    for _ in 0..=deg {
        for j in f2..=l2 {
            let (row, col) = if is_u { (at, j) } else { (j, at) };
            let point: Point3 = grid_value(poles, row, col)?;
            match weights {
                Some(weights) => {
                    let w: f64 = grid_value(weights, row, col)?;
                    local.extend_from_slice(&[point.x() * w, point.y() * w, point.z() * w, w]);
                }
                None => local.extend_from_slice(&[point.x(), point.y(), point.z()]),
            }
        }
        at += 1;
        if at > l1 {
            at = f1;
        }
    }
    let width: i32 = (l2 - f2 + 1) * dim as i32;
    eval(u, degree, &local_knots, width, &mut local)?;
    write_iso(&local, rational, dim, cpoles, cweights.as_deref_mut())?;
    // A non-rational surface asked for weights gets unit weights.
    if let (false, Some(cweights)) = (rational, cweights) {
        cweights.init(1.0);
    }
    Ok(())
}

fn write_iso(
    local: &[f64],
    rational: bool,
    dim: usize,
    cpoles: &mut Array1<'_, Point3>,
    mut cweights: Option<&mut Array1<'_, f64>>,
) -> Result<(), Failure> {
    let lower: i32 = cpoles.lower();
    for i in lower..=cpoles.upper() {
        let start: usize = (i - lower) as usize * dim;
        let pole: &[f64] = local.get(start..start + dim).ok_or_else(|| {
            Failure::out_of_range(&format!(
                "BSplSLib::Iso: CPoles({i}) beyond the {} iso poles",
                local.len() / dim
            ))
        })?;
        if !rational {
            *array_cell(cpoles, i)? = Point3::new(pole[0], pole[1], pole[2]);
            continue;
        }
        let weights: &mut Array1<'_, f64> = cweights.as_deref_mut().ok_or_else(|| {
            Failure::out_of_range("BSplSLib::Iso: rational poles need CWeights; received None")
        })?;
        let w: f64 = pole[3];
        *array_cell(weights, i)? = w;
        *array_cell(cpoles, i)? = Point3::new(pole[0] / w, pole[1] / w, pole[2] / w);
    }
    Ok(())
}

/// Reverses the rows (`u_direction`) or columns of a pole or weight grid so that
/// `last` becomes the first (`BSplSLib::Reverse`); `bsplslib::reverse(&mut poles, 3, true)?`.
pub fn reverse<T: Clone + Default + 'static>(
    grid: &mut Array2<'_, T>,
    last: i32,
    u_direction: bool,
) -> Result<(), Failure> {
    let (lower, count) = if u_direction {
        (grid.lower_row(), grid.nb_rows() as i32)
    } else {
        (grid.lower_col(), grid.nb_columns() as i32)
    };
    if count == 0 {
        return Err(Failure::out_of_range("BSplSLib::Reverse: empty pole grid"));
    }
    let l: i32 = lower + (last - lower) % count;
    // The native second copy loop starts at row/column l+1 and reads it.
    if l + 1 < lower {
        return Err(Failure::out_of_range(&format!(
            "BSplSLib::Reverse: Last = {last} maps to {l}; expected >= {}",
            lower - 1
        )));
    }
    let copy: Array2<'static, T> = copied(grid);
    let (rows, cols) = (
        (grid.lower_row(), grid.upper_row()),
        (grid.lower_col(), grid.upper_col()),
    );
    for i in rows.0..=rows.1 {
        for j in cols.0..=cols.1 {
            let (along, _) = if u_direction { (i, j) } else { (j, i) };
            // Source index along the reversed direction.
            let source: i32 = if along <= l {
                l - along
            } else {
                l + count - along
            };
            let source: i32 = source + lower;
            let value: T = if u_direction {
                grid_value_clone(&copy, source, j)?
            } else {
                grid_value_clone(&copy, i, source)?
            };
            *grid_cell(grid, i, j)? = value;
        }
    }
    Ok(())
}

fn copied<T: Clone>(grid: &Array2<'_, T>) -> Array2<'static, T> {
    let mut copy: Array2<'static, T> = Array2::default();
    copy.assign(grid);
    copy
}

fn grid_value_clone<T: Clone>(grid: &Array2<'_, T>, row: i32, col: i32) -> Result<T, Failure> {
    grid.value(row, col)
        .cloned()
        .map_err(|error| Failure::out_of_range(&format!("NCollection_Array2: {error}")))
}

/// Tests whether weights differ along rows I1..I2 for columns J1..J2, cyclically
/// (`BSplSLib::IsRational`). Quirk kept: the default tolerance is
/// `Epsilon(Weights(I1, I2))`, which reads column I2;
/// `bsplslib::is_rational(&weights, 1, 3, 1, 3, 0.0)?`.
pub fn is_rational(
    weights: &Array2<'_, f64>,
    i1: i32,
    i2: i32,
    j1: i32,
    j2: i32,
    epsi: f64,
) -> Result<bool, Failure> {
    let eps: f64 = if epsi > 0.0 {
        epsi
    } else {
        epsilon(grid_value(weights, i1, i2)?)
    };
    let (fi, li) = (weights.lower_row(), weights.nb_rows() as i32);
    let (fj, lj) = (weights.lower_col(), weights.nb_columns() as i32);
    if li == 0 || lj == 0 {
        return Err(Failure::out_of_range(
            "BSplSLib::IsRational: empty weight grid",
        ));
    }
    for i in i1 - fi..i2 - fi {
        for j in j1 - fj..j2 - fj {
            let col: i32 = fj + j % lj;
            let difference: f64 = grid_value(weights, fi + i % li, col)?
                - grid_value(weights, fi + (i + 1) % li, col)?;
            if difference.abs() > eps {
                return Ok(true);
            }
        }
    }
    Ok(false)
}
