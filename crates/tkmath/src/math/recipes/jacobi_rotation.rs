//! One Jacobi pivot: the skip test, the rotation angle and the native `ROTATE` macro.
use super::{Matrix, Vector};

/// Sweep number and off-diagonal threshold of the current Jacobi pass.
#[derive(Clone, Copy)]
pub(super) struct Sweep {
    pub iteration: i32,
    pub threshold: f64,
}

/// Annihilates `a(row, col)`; returns 1 when a rotation was applied, else 0.
pub(super) fn pivot(
    a: &mut Matrix<'_>,
    d: &mut Vector<'_>,
    shifts: &mut Vector<'_>,
    v: &mut Matrix<'_>,
    [row, col]: [i32; 2],
    sweep: Sweep,
) -> i32 {
    let magnitude: f64 = 100.0 * a[(row, col)].abs();
    let left: f64 = d[row].abs();
    let right: f64 = d[col].abs();
    if sweep.iteration > 4 && left + magnitude == left && right + magnitude == right {
        a[(row, col)] = 0.0;
        return 0;
    }
    // Native `else if (fabs(a) > tresh)`: a NaN entry is skipped, so negate `>`.
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    if !(a[(row, col)].abs() > sweep.threshold) {
        return 0;
    }
    let t: f64 = tangent(a[(row, col)], d[col] - d[row], magnitude);
    let cosine: f64 = 1.0 / (1.0 + t * t).sqrt();
    let sine: f64 = t * cosine;
    let tau: f64 = sine / (1.0 + cosine);
    let shift: f64 = t * a[(row, col)];
    shifts[row] -= shift;
    shifts[col] += shift;
    d[row] -= shift;
    d[col] += shift;
    a[(row, col)] = 0.0;
    rotate_upper(a, row, col, [sine, tau]);
    for index in 1..=a.row_number() {
        rotate(v, [(index, row), (index, col)], [sine, tau]);
    }
    1
}
fn tangent(pivot: f64, difference: f64, magnitude: f64) -> f64 {
    if difference.abs() + magnitude == difference.abs() {
        return pivot / difference;
    }
    let theta: f64 = 0.5 * difference / pivot;
    let result: f64 = 1.0 / (theta.abs() + (1.0 + theta * theta).sqrt());
    if theta < 0.0 { -result } else { result }
}
fn rotate_upper(work: &mut Matrix<'_>, row: i32, col: i32, angle: [f64; 2]) {
    for index in 1..row {
        rotate(work, [(index, row), (index, col)], angle);
    }
    for index in row + 1..col {
        rotate(work, [(row, index), (index, col)], angle);
    }
    for index in col + 1..=work.row_number() {
        rotate(work, [(row, index), (col, index)], angle);
    }
}
/// Native `ROTATE(a, i, j, k, l)`: `g = a(i,j); h = a(k,l)` then the tau-stabilized update.
fn rotate(work: &mut Matrix<'_>, [left, right]: [(i32, i32); 2], [sine, tau]: [f64; 2]) {
    let g: f64 = work[left];
    let h: f64 = work[right];
    work[left] = g - sine * (h + g * tau);
    work[right] = h + sine * (g - h * tau);
}
