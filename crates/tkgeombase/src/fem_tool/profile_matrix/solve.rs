// Created on: 1997-10-31
// Created by: Roman BORISOV
// Copyright (c) 1997-1999 Matra Datavision
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
// Rust translation of ModelingData/TKGeomBase/FEmTool/FEmTool_ProfileMatrix.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The profile Cholesky factorization S S^t = A, its two triangular solves and the
//! product A X, in the native address arithmetic.
use super::layout::{ProfileLayout, slot, slot_mut};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::math::Vector;

/// The pivot threshold below which the matrix is not positive definite.
const MIN_PIVOT: f64 = 1.0e-32;

/// `Decompose()`: fills `factor` (zeroed first) and returns false at the first
/// pivot below 1e-32; `decompose(&layout, &matrix, &mut factor)?`.
pub(super) fn decompose(
    layout: &ProfileLayout,
    matrix: &[f64],
    factor: &mut [f64],
) -> Result<bool, Failure> {
    factor.fill(0.);
    for j in 1..=layout.rows() {
        let (height, diagonal) = (layout.height(j)?, layout.diagonal(j)?);
        let mut sum: f64 = 0.;
        for k in diagonal - height..diagonal {
            let s: f64 = *slot(factor, k)?;
            sum += s * s;
        }
        let a: f64 = *slot(matrix, diagonal)? - sum;
        if a < MIN_PIVOT {
            return Ok(false);
        }
        let a: f64 = a.sqrt();
        *slot_mut(factor, diagonal)? = a;
        column_below(layout, matrix, factor, j, a)?;
    }
    Ok(true)
}

// S(i, j) for every row i > j storing column j, following NextCoeff.
fn column_below(
    layout: &ProfileLayout,
    matrix: &[f64],
    factor: &mut [f64],
    j: i32,
    a: f64,
) -> Result<(), Failure> {
    let diagonal: i32 = layout.diagonal(j)?;
    let first_j: i32 = j - layout.height(j)?;
    let mut current: i32 = diagonal;
    loop {
        let i: i32 = layout.next(current)?;
        if i <= 0 {
            return Ok(());
        }
        current = layout.diagonal(i)? - (i - j);
        let k_min: i32 = (i - layout.height(i)?).max(first_j);
        let (mut ik, mut jk) = (layout.diagonal(i)? - i + k_min, diagonal - j + k_min);
        let mut sum: f64 = 0.;
        for _ in k_min..j {
            sum += *slot(factor, ik)? * *slot(factor, jk)?;
            ik += 1;
            jk += 1;
        }
        *slot_mut(factor, current)? = (*slot(matrix, current)? - sum) / a;
    }
}

// Element `i` (1-based) of `v`, read from `v.Lower()` as the native pointer walk does.
fn at(v: &Vector<'_>, i: i32) -> Result<f64, Failure> {
    v.value(v.lower() + i - 1).copied()
}

fn at_mut<'v>(v: &'v mut Vector<'_>, i: i32) -> Result<&'v mut f64, Failure> {
    let lower: i32 = v.lower();
    v.change_value(lower + i - 1)
}

/// `Solve(B, X)`: forward substitution S w = B, then backward S^t X = w;
/// `solve(&layout, &factor, &b, &mut x)?`.
pub(super) fn solve(
    layout: &ProfileLayout,
    factor: &[f64],
    b: &Vector<'_>,
    x: &mut Vector<'_>,
) -> Result<(), Failure> {
    for i in 1..=layout.rows() {
        let (height, diagonal) = (layout.height(i)?, layout.diagonal(i)?);
        let mut sum: f64 = 0.;
        for (j, jj) in (i - height..i).zip(diagonal - height..) {
            sum += *slot(factor, jj)? * at(x, j)?;
        }
        *at_mut(x, i)? = (at(b, i)? - sum) / *slot(factor, diagonal)?;
    }
    for i in (1..=layout.rows()).rev() {
        let diagonal: i32 = layout.diagonal(i)?;
        let (mut j, mut sum): (i32, f64) = (layout.next(diagonal)?, 0.);
        while j > 0 {
            let current: i32 = layout.diagonal(j)? - (j - i);
            sum += *slot(factor, current)? * at(x, j)?;
            j = layout.next(current)?;
        }
        *at_mut(x, i)? = (at(x, i)? - sum) / *slot(factor, diagonal)?;
    }
    Ok(())
}

/// `Multiplied(X, MX)`: the stored row part then the column part below the
/// diagonal, accumulated in MX(i) as natively; `multiplied(&layout, &matrix, &x, &mut mx)?`.
pub(super) fn multiplied(
    layout: &ProfileLayout,
    matrix: &[f64],
    x: &Vector<'_>,
    mx: &mut Vector<'_>,
) -> Result<(), Failure> {
    for i in 1..=layout.rows() {
        let (height, diagonal) = (layout.height(i)?, layout.diagonal(i)?);
        *at_mut(mx, i)? = 0.;
        for (j, jj) in (i - height..=i).zip(diagonal - height..) {
            *at_mut(mx, i)? += *slot(matrix, jj)? * at(x, j)?;
        }
        let mut current: i32 = diagonal;
        let mut j: i32 = layout.next(current)?;
        while j > 0 {
            current = layout.diagonal(j)? - (j - i);
            *at_mut(mx, i)? += *slot(matrix, current)? * at(x, j)?;
            j = layout.next(current)?;
        }
    }
    Ok(())
}
