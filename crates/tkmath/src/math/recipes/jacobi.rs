//! Cyclic Jacobi eigenanalysis of a symmetric matrix's upper triangle (`Jacobi`).
use super::jacobi_rotation::{Sweep, pivot};
use super::{
    Failure, Matrix, STATUS_NO_CONVERGENCE, STATUS_OK, Vector, require_matrix, require_vector,
};

/// Diagonalizes `a` (upper triangle, destroyed) into eigenvalues `d` and eigenvector
/// columns `v`, sorted by descending eigenvalue; `nrot` counts rotations. Returns
/// `math_Status_OK`, or `math_Status_NoConvergence` after fifty sweeps (still sorted).
/// `jacobi(&mut a,&mut d,&mut v,&mut nrot)?`.
pub fn jacobi(
    a: &mut Matrix<'_>,
    d: &mut Vector<'_>,
    v: &mut Matrix<'_>,
    nrot: &mut i32,
) -> Result<i32, Failure> {
    let n = a.row_number();
    // EigenSort walks `d.Length()` entries and the matching columns of `v`.
    let sorted = d.length().max(n);
    require_matrix(a, n, n, "Jacobi")?;
    require_vector(d, sorted, "Jacobi")?;
    require_matrix(v, sorted, sorted, "Jacobi")?;
    let mut sums = Vector::with_bounds(1, n)?;
    let mut shifts = Vector::with_bounds(1, n)?;
    initialize(a, d, v, &mut sums, &mut shifts);
    *nrot = 0;
    let status = iterate(a, d, v, &mut sums, &mut shifts, nrot);
    eigen_sort(d, v);
    Ok(status)
}
fn initialize(
    a: &Matrix<'_>,
    d: &mut Vector<'_>,
    v: &mut Matrix<'_>,
    sums: &mut Vector<'_>,
    shifts: &mut Vector<'_>,
) {
    let n = a.row_number();
    for ip in 1..=n {
        for iq in 1..=n {
            v[(ip, iq)] = 0.0;
        }
        v[(ip, ip)] = 1.0;
    }
    for ip in 1..=n {
        d[ip] = a[(ip, ip)];
        sums[ip] = d[ip];
        shifts[ip] = 0.0;
    }
}
fn iterate(
    a: &mut Matrix<'_>,
    d: &mut Vector<'_>,
    v: &mut Matrix<'_>,
    sums: &mut Vector<'_>,
    shifts: &mut Vector<'_>,
    nrot: &mut i32,
) -> i32 {
    let n = a.row_number();
    for iteration in 1..=50 {
        let sum: f64 = upper_sum(a);
        if sum == 0.0 {
            return STATUS_OK;
        }
        let threshold: f64 = if iteration < 4 {
            0.2 * sum / f64::from(n * n)
        } else {
            0.0
        };
        let sweep = Sweep {
            iteration,
            threshold,
        };
        for row in 1..n {
            for col in row + 1..=n {
                *nrot += pivot(a, d, shifts, v, [row, col], sweep);
            }
        }
        accumulate(n, d, sums, shifts);
    }
    STATUS_NO_CONVERGENCE
}
fn upper_sum(work: &Matrix<'_>) -> f64 {
    let n = work.row_number();
    let mut sum: f64 = 0.0;
    for row in 1..n {
        for col in row + 1..=n {
            sum += work[(row, col)].abs();
        }
    }
    sum
}
fn accumulate(n: i32, d: &mut Vector<'_>, sums: &mut Vector<'_>, shifts: &mut Vector<'_>) {
    for index in 1..=n {
        sums[index] += shifts[index];
        d[index] = sums[index];
        shifts[index] = 0.0;
    }
}
/// Native `EigenSort`: selection sort to descending order, the last of equal maxima
/// winning, swapping whole columns of `v`.
fn eigen_sort(d: &mut Vector<'_>, v: &mut Matrix<'_>) {
    let n = d.length();
    for i in 1..n {
        let k = last_maximum(d, i);
        if k == i {
            continue;
        }
        let p = d[k];
        d[k] = d[i];
        d[i] = p;
        for j in 1..=n {
            let p = v[(j, i)];
            v[(j, i)] = v[(j, k)];
            v[(j, k)] = p;
        }
    }
}
fn last_maximum(d: &Vector<'_>, start: i32) -> i32 {
    let mut selected: i32 = start;
    let mut p = d[start];
    for j in start + 1..=d.length() {
        if d[j] >= p {
            selected = j;
            p = d[j];
        }
    }
    selected
}
