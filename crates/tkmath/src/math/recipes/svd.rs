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

//! Golub-Reinsch singular value decomposition (SVD_Decompose / SVD_Solve).
use super::{
    Failure, Matrix, STATUS_NO_CONVERGENCE, STATUS_OK, Vector, require_matrix, require_vector,
};

/// Mutable views of one decomposition; `m`/`n` are the native `a` row/column counts.
pub(super) struct SvdWork<'x, 'a, 'w, 'v, 'r> {
    pub a: &'x mut Matrix<'a>,
    pub w: &'x mut Vector<'w>,
    pub v: &'x mut Matrix<'v>,
    pub rv1: &'x mut Vector<'r>,
    pub m: i32,
    pub n: i32,
}

/// Decomposes `a = U W V^T` in place: `a` becomes U, `w` the singular values and `v`
/// the right vectors; returns a `math_Status_*` code. `svd_decompose(&mut a,&mut w,&mut v)?`.
pub fn svd_decompose(
    a: &mut Matrix<'_>,
    w: &mut Vector<'_>,
    v: &mut Matrix<'_>,
) -> Result<i32, Failure> {
    let mut rv1 = Vector::with_bounds(1, a.col_number())?;
    svd_decompose_with_work(a, w, v, &mut rv1)
}

/// The overload taking the caller's superdiagonal workspace `rv1`.
///
/// Natively a matrix with fewer rows than columns writes past its storage; that input
/// is refused with `Standard_DimensionError`. `svd_decompose_with_work(&mut a,&mut w,&mut v,&mut rv1)?`.
pub fn svd_decompose_with_work(
    a: &mut Matrix<'_>,
    w: &mut Vector<'_>,
    v: &mut Matrix<'_>,
    rv1: &mut Vector<'_>,
) -> Result<i32, Failure> {
    let (m, n) = (a.row_number(), a.col_number());
    if m < n {
        return Err(Failure::with_type(
            "Standard_DimensionError",
            &format!("SVD_Decompose: {m} rows is below the {n} columns it requires"),
            "",
        ));
    }
    require_matrix(a, m, n, "SVD_Decompose")?;
    require_matrix(v, n, n, "SVD_Decompose")?;
    require_vector(w, n, "SVD_Decompose")?;
    require_vector(rv1, n, "SVD_Decompose")?;
    let mut work = SvdWork { a, w, v, rv1, m, n };
    let norm = work.bidiagonalize();
    work.accumulate_right();
    work.accumulate_left();
    Ok(if work.diagonalize(norm) {
        STATUS_OK
    } else {
        STATUS_NO_CONVERGENCE
    })
}

/// Back-substitutes `x = V diag(1/w) U^T b`, skipping zero singular values;
/// `svd_solve(&u,&w,&v,&b,&mut x)?`.
pub fn svd_solve(
    u: &Matrix<'_>,
    w: &Vector<'_>,
    v: &Matrix<'_>,
    b: &Vector<'_>,
    x: &mut Vector<'_>,
) -> Result<(), Failure> {
    let (m, n) = (u.row_number(), u.col_number());
    require_matrix(u, m, n, "SVD_Solve")?;
    require_matrix(v, n, n, "SVD_Solve")?;
    require_vector(w, n, "SVD_Solve")?;
    require_vector(b, m, "SVD_Solve")?;
    require_vector(x, n, "SVD_Solve")?;
    let mut tmp = Vector::with_bounds(1, n)?;
    for j in 1..=n {
        tmp[j] = projected(u, w, b, j, m);
    }
    for j in 1..=n {
        let mut s = 0.0;
        for jj in 1..=n {
            s += v[(j, jj)] * tmp[jj];
        }
        x[j] = s;
    }
    Ok(())
}
fn projected(u: &Matrix<'_>, w: &Vector<'_>, b: &Vector<'_>, j: i32, m: i32) -> f64 {
    let mut s = 0.0;
    if w[j] != 0.0 {
        for i in 1..=m {
            s += u[(i, j)] * b[i];
        }
        s /= w[j];
    }
    s
}

/// Native `SIGN(a, b)` macro.
pub(super) fn sign(magnitude: f64, direction: f64) -> f64 {
    if direction >= 0.0 {
        magnitude.abs()
    } else {
        -magnitude.abs()
    }
}
/// Native overflow-avoiding hypotenuse `PYTHAG`.
pub(super) fn pythag(left: f64, right: f64) -> f64 {
    let a: f64 = left.abs();
    let b: f64 = right.abs();
    if a > b {
        let ratio: f64 = b / a;
        return a * (1.0 + ratio * ratio).sqrt();
    }
    if b != 0.0 {
        let ratio: f64 = a / b;
        return b * (1.0 + ratio * ratio).sqrt();
    }
    0.0
}
/// Rotates columns `left`/`right` over rows `1..=rows` (the native `y*c + z*s` pair).
pub(super) fn rotate_columns(
    matrix: &mut Matrix<'_>,
    rows: i32,
    [left, right]: [i32; 2],
    [cosine, sine]: [f64; 2],
) {
    for row in 1..=rows {
        let y: f64 = matrix[(row, left)];
        let z: f64 = matrix[(row, right)];
        matrix[(row, left)] = y * cosine + z * sine;
        matrix[(row, right)] = z * cosine - y * sine;
    }
}
