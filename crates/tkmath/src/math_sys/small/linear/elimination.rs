//! `Solve3x3`/`Solve4x4` of MathSys_Newton3D/4D.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! Both are row-scaled partial-pivot eliminations, but they round differently: 3x3 keeps
//! `F / scale` and negates during back substitution, divides by the pivot for each factor and
//! refuses a NaN pivot; 4x4 stores `-F / scale`, multiplies by the pivot inverse, zeroes the
//! eliminated entry and lets a NaN pivot through (`aMaxVal <= tol`). One routine keyed on `N`
//! keeps both orders.
use crate::math_sys::small::checks::{NEWTON_PIVOT_TOL, list_max};

/// Solves `J delta = -F` by scaled Gaussian elimination with partial pivoting (`Solve3x3`).
/// `delta` receives every back-substituted value even when one is not finite;
/// `solve_3x3(&j, &f, &mut delta)`.
pub fn solve_3x3(j: &[[f64; 3]; 3], f: &[f64; 3], delta: &mut [f64; 3]) -> bool {
    solve_n(j, f, delta)
}
/// Solves `J delta = -F` on the augmented 4x5 matrix (`Solve4x4`); `solve_4x4(&j, &f, &mut delta)`.
pub fn solve_4x4(j: &[[f64; 4]; 4], f: &[f64; 4], delta: &mut [f64; 4]) -> bool {
    solve_n(j, f, delta)
}
pub(super) fn solve_n<const N: usize>(
    j: &[[f64; N]; N],
    f: &[f64; N],
    delta: &mut [f64; N],
) -> bool {
    let Some((mut a, mut b)) = scaled(j, f) else {
        return false;
    };
    for pivot in 0..N {
        let row: usize = (pivot + 1..N).fold(pivot, |best, r| {
            if a[r][pivot].abs() > a[best][pivot].abs() {
                r
            } else {
                best
            }
        });
        let magnitude: f64 = a[row][pivot].abs();
        if (N == 4 && magnitude <= NEWTON_PIVOT_TOL)
            || (N != 4
                && magnitude.partial_cmp(&NEWTON_PIVOT_TOL) != Some(std::cmp::Ordering::Greater))
        {
            return false;
        }
        swap(&mut a, &mut b, row, pivot);
        eliminate(&mut a, &mut b, pivot);
    }
    for row in (0..N).rev() {
        let initial: f64 = if N == 4 { b[row] } else { -b[row] };
        let sum: f64 = (row + 1..N).fold(initial, |v, col| v - a[row][col] * delta[col]);
        delta[row] = sum / a[row][row];
    }
    delta.iter().all(|v| v.is_finite())
}
fn scaled<const N: usize>(j: &[[f64; N]; N], f: &[f64; N]) -> Option<([[f64; N]; N], [f64; N])> {
    let mut a: [[f64; N]; N] = [[0.0; N]; N];
    let mut b: [f64; N] = [0.0; N];
    for row in 0..N {
        let scale: f64 = list_max(&j[row].map(f64::abs));
        if scale.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater) {
            return None;
        }
        a[row] = j[row].map(|v| v / scale);
        b[row] = if N == 4 {
            -f[row] / scale
        } else {
            f[row] / scale
        };
        if !b[row].is_finite() {
            return None;
        }
    }
    Some((a, b))
}
// Native row/column indices retain the elimination update order.
#[allow(clippy::needless_range_loop)]
fn swap<const N: usize>(a: &mut [[f64; N]; N], b: &mut [f64; N], row: usize, pivot: usize) {
    for col in pivot..N {
        let saved: f64 = a[pivot][col];
        a[pivot][col] = a[row][col];
        a[row][col] = saved;
    }
    b.swap(row, pivot);
}
#[allow(clippy::needless_range_loop)]
fn eliminate<const N: usize>(a: &mut [[f64; N]; N], b: &mut [f64; N], pivot: usize) {
    let inverse: f64 = 1.0 / a[pivot][pivot];
    for row in pivot + 1..N {
        let factor: f64 = if N == 4 {
            a[row][pivot] * inverse
        } else {
            a[row][pivot] / a[pivot][pivot]
        };
        for col in pivot + 1..N {
            a[row][col] -= factor * a[pivot][col];
        }
        b[row] -= factor * b[pivot];
        if N == 4 {
            a[row][pivot] = 0.0;
        }
    }
}
