//! Crout LU with implicit row scaling and partial pivoting (LU_Decompose/Solve/Invert).
use super::{
    Failure, IntegerVector, Matrix, STATUS_OK, STATUS_SINGULAR_MATRIX, STATUS_USER_ABORTED, Vector,
    checked, require_square, require_vector,
};
use rustycad_tkernel::message::{ProgressRange, ProgressScope};

/// Decomposes `a` in place with a scratch scale vector and the default (unbound,
/// never aborting) progress range; returns a `math_Status_*` code.
///
/// `indx` receives the pivot rows and `d` the permutation sign.
/// `lu_decompose(&mut a,&mut indx,&mut d,1e-20)?`.
pub fn lu_decompose(
    a: &mut Matrix<'_>,
    indx: &mut IntegerVector<'_>,
    d: &mut f64,
    tiny: f64,
) -> Result<i32, Failure> {
    lu_decompose_with_progress(a, indx, d, tiny, &ProgressRange::default())
}

/// `LU_Decompose(a, indx, d, TINY, theProgress)`: allocates the scale workspace and
/// runs [`lu_decompose_with_scales_and_progress`];
/// `lu_decompose_with_progress(&mut a,&mut indx,&mut d,1e-20,&indicator.start())?`.
pub fn lu_decompose_with_progress(
    a: &mut Matrix<'_>,
    indx: &mut IntegerVector<'_>,
    d: &mut f64,
    tiny: f64,
    progress: &ProgressRange<'_>,
) -> Result<i32, Failure> {
    let mut scales = Vector::with_bounds(1, a.row_number())?;
    lu_decompose_with_scales_and_progress(a, indx, d, &mut scales, tiny, progress)
}

/// The overload taking the caller's scale workspace `vv` (native default TINY 1e-30)
/// with the default progress range;
/// `lu_decompose_with_scales(&mut a,&mut indx,&mut d,&mut vv,1e-30)?`.
pub fn lu_decompose_with_scales(
    a: &mut Matrix<'_>,
    indx: &mut IntegerVector<'_>,
    d: &mut f64,
    vv: &mut Vector<'_>,
    tiny: f64,
) -> Result<i32, Failure> {
    lu_decompose_with_scales_and_progress(a, indx, d, vv, tiny, &ProgressRange::default())
}

/// `LU_Decompose(a, indx, d, vv, TINY, theProgress)`: the Crout decomposition inside a
/// `"math_Gauss LU_Decompose"` progress scope of `n` steps, one per column.
///
/// The user break is polled before each column; a break stops with
/// `STATUS_USER_ABORTED` (-1) and leaves the partial decomposition in `a`. Every
/// exit, the singular ones included, closes the scope and so completes `progress`,
/// as the native scope destructor does.
/// `lu_decompose_with_scales_and_progress(&mut a,&mut indx,&mut d,&mut vv,1e-30,&range)?`.
pub fn lu_decompose_with_scales_and_progress(
    a: &mut Matrix<'_>,
    indx: &mut IntegerVector<'_>,
    d: &mut f64,
    vv: &mut Vector<'_>,
    tiny: f64,
    progress: &ProgressRange<'_>,
) -> Result<i32, Failure> {
    let n = require_square(a, indx, "LU_Decompose")?;
    require_vector(vv, n, "LU_Decompose")?;
    *d = 1.0;
    let scope = ProgressScope::with_literal(progress, "math_Gauss LU_Decompose", n.into(), false);
    if !row_scales(a, vv, n, tiny) {
        return Ok(STATUS_SINGULAR_MATRIX);
    }
    // Native `for (j = 1; j <= n && aPS.More(); j++, aPS.Next())`: each Next() range
    // is a temporary, so it advances the indicator at once.
    let mut j = 1;
    while j <= n && scope.more() {
        if !decompose_column(a, indx, d, vv, j, tiny) {
            return Ok(STATUS_SINGULAR_MATRIX);
        }
        j += 1;
        drop(scope.next(1.));
    }
    Ok(if j <= n {
        STATUS_USER_ABORTED
    } else {
        STATUS_OK
    })
}

/// One Crout column: reduce, pivot, record and scale; false when the pivot is at or
/// below `tiny` (singular).
fn decompose_column(
    a: &mut Matrix<'_>,
    indx: &mut IntegerVector<'_>,
    d: &mut f64,
    vv: &mut Vector<'_>,
    j: i32,
    tiny: f64,
) -> bool {
    let n = a.row_number();
    upper_column(a, j);
    let pivot = pivot_row(a, vv, j, n);
    if j != pivot {
        swap_rows(a, j, pivot, n);
        *d = -*d;
        vv[pivot] = vv[j];
    }
    indx[j] = pivot;
    if a[(j, j)].abs() <= tiny {
        return false;
    }
    if j != n {
        scale_column(a, j, n);
    }
    true
}

/// Stores `1 / max |a(i, j)|` per row; a row at or below `tiny` stops with the earlier
/// scales already written, as natively.
fn row_scales(a: &Matrix<'_>, vv: &mut Vector<'_>, n: i32, tiny: f64) -> bool {
    for i in 1..=n {
        let mut big = 0.0;
        for j in 1..=n {
            let temp = a[(i, j)].abs();
            if temp > big {
                big = temp;
            }
        }
        if big <= tiny {
            return false;
        }
        vv[i] = 1.0 / big;
    }
    true
}
fn upper_column(a: &mut Matrix<'_>, j: i32) {
    for i in 1..j {
        a[(i, j)] = reduced(a, i, j, i);
    }
}
fn reduced(a: &Matrix<'_>, i: i32, j: i32, end: i32) -> f64 {
    let mut sum = a[(i, j)];
    for k in 1..end {
        sum -= a[(i, k)] * a[(k, j)];
    }
    sum
}
/// Reduces the lower part of column `j` and selects the scaled largest pivot.
fn pivot_row(a: &mut Matrix<'_>, vv: &Vector<'_>, j: i32, n: i32) -> i32 {
    let mut big = 0.0;
    let mut pivot = j;
    for i in j..=n {
        let sum = reduced(a, i, j, j);
        a[(i, j)] = sum;
        let dum = vv[i] * sum.abs();
        // OCC25559: the `<` test lets NaN, Inf and IND candidates still update the pivot.
        if dum < big {
            continue;
        }
        big = dum;
        pivot = i;
    }
    pivot
}
fn swap_rows(a: &mut Matrix<'_>, j: i32, pivot: i32, n: i32) {
    for k in 1..=n {
        let dum = a[(pivot, k)];
        a[(pivot, k)] = a[(j, k)];
        a[(j, k)] = dum;
    }
}
fn scale_column(a: &mut Matrix<'_>, j: i32, n: i32) {
    let dum = 1.0 / a[(j, j)];
    for i in j + 1..=n {
        a[(i, j)] *= dum;
    }
}

/// Solves `A x = b` in place from an `lu_decompose` result; `b` may have any lower
/// bound (the native `nblow` offset). `lu_solve(&a,&indx,&mut b)?`.
pub fn lu_solve(
    a: &Matrix<'_>,
    indx: &IntegerVector<'_>,
    b: &mut Vector<'_>,
) -> Result<(), Failure> {
    let n = require_square(a, indx, "LU_Solve")?;
    if b.length() < n {
        return Err(Failure::with_type(
            "Standard_DimensionError",
            &format!(
                "LU_Solve: right-hand side length {} is below {n}",
                b.length()
            ),
            "",
        ));
    }
    let offset = b.lower() - 1;
    forward_substitute(a, indx, b, n, offset)?;
    for i in (1..=n).rev() {
        let mut sum = b[i + offset];
        for j in i + 1..=n {
            sum -= a[(i, j)] * b[j + offset];
        }
        b[i + offset] = sum / a[(i, i)];
    }
    Ok(())
}
fn forward_substitute(
    a: &Matrix<'_>,
    indx: &IntegerVector<'_>,
    b: &mut Vector<'_>,
    n: i32,
    offset: i32,
) -> Result<(), Failure> {
    let mut first = 0;
    for i in 1..=n {
        let ip = indx[i];
        let mut sum = checked(b, ip + offset)?;
        b[ip + offset] = b[i + offset];
        if first != 0 {
            for j in first..i {
                sum -= a[(i, j)] * b[j + offset];
            }
        } else if sum != 0.0 {
            first = i;
        }
        b[i + offset] = sum;
    }
    Ok(())
}

/// Replaces `a` by its inverse when `lu_decompose` succeeds; returns its status and
/// leaves the partial decomposition in `a` otherwise. `lu_invert(&mut a)?`.
pub fn lu_invert(a: &mut Matrix<'_>) -> Result<i32, Failure> {
    let n = a.row_number();
    // A row count is never negative, so the 1..n bounds are always valid.
    let mut inverse = Matrix::with_bounds(1, n, 1, n).unwrap();
    let mut column = Vector::with_bounds(1, n)?;
    let mut indx = IntegerVector::with_bounds(1, n)?;
    let mut d = 0.0;
    let error = lu_decompose(a, &mut indx, &mut d, 1.0e-20)?;
    if error != STATUS_OK {
        return Ok(error);
    }
    for j in 1..=n {
        column.init(0.0);
        column[j] = 1.0;
        lu_solve(a, &indx, &mut column)?;
        for i in 1..=n {
            inverse[(i, j)] = column[i];
        }
    }
    for j in 1..=n {
        for i in 1..=n {
            a[(i, j)] = inverse[(i, j)];
        }
    }
    Ok(error)
}
