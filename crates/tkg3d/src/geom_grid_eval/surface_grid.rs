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
// Rust translation of the U x V loops shared by ModelingData/TKG3d/GeomGridEval/*.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The U x V loops of the surface evaluators: a grid of bounds `1..NbU x 1..NbV`
//! filled row by row (U outer, V inner), with an optional per-row context that the
//! elementary surfaces use for their U-dependent terms.
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Failure;

/// An empty grid (`NCollection_Array2<T>()`).
pub(super) fn empty_grid<T>() -> Array2<'static, T> {
    Array2::default()
}

/// A default-filled grid of bounds `1..nb_u x 1..nb_v` (`NCollection_Array2<T>(1, NbU, 1, NbV)`).
pub(super) fn grid_array<T: Default>(nb_u: usize, nb_v: usize) -> Array2<'static, T> {
    let mut grid: Array2<'static, T> = Array2::with_size(nb_u, nb_v);
    grid.update_lower_row(1);
    grid.update_lower_col(1);
    grid
}

/// Whether either parameter array is empty (the `aNbU == 0 || aNbV == 0` tests).
pub(super) fn any_empty(u: &Array1<'_, f64>, v: &Array1<'_, f64>) -> bool {
    u.is_empty() || v.is_empty()
}

/// The `theNU < 0 || theNV < 0 || (theNU + theNV) < 1` rejection of every DN.
pub(super) fn invalid_orders(nu: i32, nv: i32) -> bool {
    nu < 0 || nv < 0 || (nu + nv) < 1
}

/// Fills the grid row by row: `row(u)` once per U value (the `UContext`), then
/// `cell(&context, v)` for every V value; the first failure aborts the grid.
pub(super) fn try_map_rows<C, T: Default>(
    u_params: &Array1<'_, f64>,
    v_params: &Array1<'_, f64>,
    mut row: impl FnMut(f64) -> Result<C, Failure>,
    mut cell: impl FnMut(&C, f64) -> Result<T, Failure>,
) -> Result<Array2<'static, T>, Failure> {
    let nb_v: usize = v_params.len();
    let mut grid: Array2<'static, T> = grid_array(u_params.len(), nb_v);
    let slots = grid.as_mut_slice().chunks_mut(nb_v.max(1));
    for (slots, &u) in slots.zip(u_params.iter()) {
        let context: C = row(u)?;
        for (slot, &v) in slots.iter_mut().zip(v_params.iter()) {
            *slot = cell(&context, v)?;
        }
    }
    Ok(grid)
}

/// `try_map_rows` for evaluators that cannot raise.
pub(super) fn map_rows<C, T: Default>(
    u_params: &Array1<'_, f64>,
    v_params: &Array1<'_, f64>,
    mut row: impl FnMut(f64) -> C,
    mut cell: impl FnMut(&C, f64) -> T,
) -> Array2<'static, T> {
    let filled: Result<Array2<'static, T>, Failure> =
        try_map_rows(u_params, v_params, |u| Ok(row(u)), |c, v| Ok(cell(c, v)));
    filled.unwrap_or_default()
}

/// Fills cell `(i, j)` with `cell(i, u_i, j, v_j)`, with zero-based indices (the loops
/// that read a basis curve grid by row or by column); the first failure aborts.
pub(super) fn try_map_indexed<T: Default>(
    u_params: &Array1<'_, f64>,
    v_params: &Array1<'_, f64>,
    mut cell: impl FnMut((usize, f64), (usize, f64)) -> Result<T, Failure>,
) -> Result<Array2<'static, T>, Failure> {
    let nb_v: usize = v_params.len();
    let mut grid: Array2<'static, T> = grid_array(u_params.len(), nb_v);
    let rows = grid.as_mut_slice().chunks_mut(nb_v.max(1));
    for (i, (slots, &u)) in rows.zip(u_params.iter()).enumerate() {
        for (j, (slot, &v)) in slots.iter_mut().zip(v_params.iter()).enumerate() {
            *slot = cell((i, u), (j, v))?;
        }
    }
    Ok(grid)
}

/// A grid of bounds `1..nb_u x 1..nb_v` whose cells all hold `value` (the constant
/// derivative loops).
pub(super) fn constant_surface_grid<T: Default + Copy>(
    nb_u: usize,
    nb_v: usize,
    value: T,
) -> Array2<'static, T> {
    let mut grid: Array2<'static, T> = grid_array(nb_u, nb_v);
    grid.init(value);
    grid
}
