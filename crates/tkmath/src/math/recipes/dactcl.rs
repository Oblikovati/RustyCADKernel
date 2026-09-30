//! Skyline (profile) `L D L^T` factorization of a symmetric matrix (DACTCL_Decompose/Solve).
//!
//! `a` stores each column's profile down to its diagonal consecutively and `indx(j)`
//! is the position of diagonal `j` in `a`, as in the native storage. Every access is
//! checked because the profile, not the dimensions, decides which indices are read.
use super::{
    Failure, IntegerVector, STATUS_OK, STATUS_SINGULAR_MATRIX, Vector, checked, checked_mut,
};

/// Factors the profile matrix in place; returns `math_Status_SingularMatrix` when a
/// pivot magnitude is at or below `min_pivot` (native default 1e-20).
/// `dactcl_decompose(&mut a,&indx,1e-20)?`.
pub fn dactcl_decompose(
    a: &mut Vector<'_>,
    indx: &IntegerVector<'_>,
    min_pivot: f64,
) -> Result<i32, Failure> {
    let mut previous = 0;
    for j in 1..=indx.length() {
        let diagonal = checked(indx, j)?;
        let height = diagonal - previous;
        if height - 2 > 0 {
            reduce_off_diagonal(a, indx, j, [previous, height])?;
        }
        if height - 2 >= 0 && !reduce_diagonal(a, indx, j, [previous, diagonal], min_pivot)? {
            return Ok(STATUS_SINGULAR_MATRIX);
        }
        previous = diagonal;
    }
    Ok(STATUS_OK)
}
/// Subtracts the profile dot products from the non-diagonal entries of column `j`.
fn reduce_off_diagonal(
    a: &mut Vector<'_>,
    indx: &IntegerVector<'_>,
    j: i32,
    [previous, height]: [i32; 2],
) -> Result<(), Failure> {
    let first = j - height + 2;
    let mut id = checked(indx, first - 1)?;
    // `k` walks the column's stored entries from `jr + 2` in step with `i`.
    for (k, i) in (previous + 2..).zip(first..j) {
        let ir = id;
        id = checked(indx, i)?;
        let length = (id - ir - 1).min(i - first + 1);
        if length > 0 {
            let dot = profile_dot(a, a, [k - length - 1, id - length - 1], length)?;
            *checked_mut(a, k)? = checked(a, k)? - dot;
        }
    }
    Ok(())
}
/// Divides column `j` by its diagonal pivots and updates `a(jd)`; false when singular.
fn reduce_diagonal(
    a: &mut Vector<'_>,
    indx: &IntegerVector<'_>,
    j: i32,
    [previous, diagonal]: [i32; 2],
    min_pivot: f64,
) -> Result<bool, Failure> {
    let k = j - diagonal;
    for i in previous + 1..diagonal {
        let id = checked(indx, k + i)?;
        let mut aa = checked(a, id)?;
        if aa < 0.0 {
            aa = -aa;
        }
        if aa <= min_pivot {
            return Ok(false);
        }
        let d = checked(a, i)?;
        *checked_mut(a, i)? = d / checked(a, id)?;
        *checked_mut(a, diagonal)? = checked(a, diagonal)? - d * checked(a, i)?;
    }
    Ok(true)
}
/// Native `dot = dot + a(idot1 + idot) * b(idot2 + idot)` for `idot = 1..=length`.
fn profile_dot(
    left: &Vector<'_>,
    right: &Vector<'_>,
    [left_base, right_base]: [i32; 2],
    length: i32,
) -> Result<f64, Failure> {
    let mut dot = 0.0;
    for idot in 1..=length {
        // `dot += x` is the same IEEE operation as the native `dot = dot + x`.
        dot += checked(left, left_base + idot)? * checked(right, right_base + idot)?;
    }
    Ok(dot)
}

/// Solves with a `dactcl_decompose` result, overwriting `b`; returns
/// `math_Status_SingularMatrix` on a pivot at or below `min_pivot` (the reduction of
/// `b` already happened). `dactcl_solve(&a,&mut b,&indx,1e-20)?`.
pub fn dactcl_solve(
    a: &Vector<'_>,
    b: &mut Vector<'_>,
    indx: &IntegerVector<'_>,
    min_pivot: f64,
) -> Result<i32, Failure> {
    let equations = indx.length();
    reduce_right_hand_side(a, b, indx)?;
    for i in 1..=equations {
        let id = checked(indx, i)?;
        let mut aa = checked(a, id)?;
        if aa < 0.0 {
            aa = -aa;
        }
        if aa <= min_pivot {
            return Ok(STATUS_SINGULAR_MATRIX);
        }
        *checked_mut(b, i)? = checked(b, i)? / checked(a, id)?;
    }
    back_substitute(a, b, indx)?;
    Ok(STATUS_OK)
}
fn reduce_right_hand_side(
    a: &Vector<'_>,
    b: &mut Vector<'_>,
    indx: &IntegerVector<'_>,
) -> Result<(), Failure> {
    let mut previous = 0;
    for j in 1..=indx.length() {
        let diagonal = checked(indx, j)?;
        let height = diagonal - previous;
        let first = j - height + 2;
        let dot = profile_dot(a, b, [previous, first - 2], height - 1)?;
        *checked_mut(b, j)? = checked(b, j)? - dot;
        previous = diagonal;
    }
    Ok(())
}
fn back_substitute(
    a: &Vector<'_>,
    b: &mut Vector<'_>,
    indx: &IntegerVector<'_>,
) -> Result<(), Failure> {
    let equations = indx.length();
    if equations < 1 {
        return Ok(());
    }
    let mut diagonal = checked(indx, equations)?;
    for j in (1..equations).rev() {
        let d = checked(b, j + 1)?;
        let previous = checked(indx, j)?;
        if diagonal - previous > 1 {
            let first = j - diagonal + previous + 2;
            let k = previous - first + 1;
            for i in first..=j {
                *checked_mut(b, i)? = checked(b, i)? - checked(a, i + k)? * d;
            }
        }
        diagonal = previous;
    }
    Ok(())
}
