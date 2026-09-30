//! MathLin_Jacobi.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use crate::math::{Jacobi, Matrix, Vector};
use crate::math_utils::{EigenResult, Status};
fn symmetric_scale(a: &Matrix<'_>) -> Option<f64> {
    let n: usize = a.row_number() as usize;
    let mut scale: f64 = 0.0;
    if n == 0 || a.col_number() as usize != n {
        return None;
    }
    for i in 0..n {
        for j in 0..n {
            let x: f64 = *a.at(i, j).unwrap();
            let y: f64 = *a.at(j, i).unwrap();
            if !x.is_finite() || (x - y).abs() > f64::EPSILON * n as f64 * x.abs().max(y.abs()) {
                return None;
            }
            scale = scale.max(x.abs());
        }
    }
    Some(scale)
}
fn normalized(a: &Matrix<'_>, scale: f64) -> Matrix<'static> {
    let n: i32 = a.row_number();
    let mut work: Matrix<'static> = Matrix::with_bounds(1, n, 1, n).unwrap();
    for i in 0..n as usize {
        for j in 0..n as usize {
            *work.change_at(i, j).unwrap() = if scale == 0.0 {
                0.0
            } else {
                a.at(i, j).unwrap() / scale
            };
        }
    }
    work
}
fn descending(values: &mut Vector<'_>, vectors: &mut Matrix<'_>) {
    for i in 0..values.length() - 1 {
        let mut selected: i32 = i;
        for j in i + 1..values.length() {
            if values[j] > values[selected] {
                selected = j;
            }
        }
        if selected != i {
            let value: f64 = values[i];
            values[i] = values[selected];
            values[selected] = value;
            vectors.swap_col(i, selected).unwrap();
        }
    }
}
/// Diagonalizes with native scaling and sorting; `jacobi(&matrix,true)`.
pub fn jacobi(a: &Matrix<'_>, sort_descending: bool) -> EigenResult {
    let Some(scale) = symmetric_scale(a) else {
        return EigenResult {
            status: Status::InvalidInput,
            ..Default::default()
        };
    };
    if a.row_number() == 1 {
        return EigenResult {
            status: Status::Ok,
            eigen_values: Some(Vector::filled(0, 0, *a.at(0, 0).unwrap()).unwrap()),
            eigen_vectors: Some(Matrix::filled(0, 0, 0, 0, 1.0).unwrap()),
            ..Default::default()
        };
    }
    let mut work: Matrix<'static> = normalized(a, scale);
    let (done, rotations, values, vectors) = Jacobi::recipe(&mut work);
    if !done {
        return EigenResult {
            status: Status::NotConverged,
            ..Default::default()
        };
    }
    let n: i32 = a.row_number();
    let mut out_values: Vector<'static> = Vector::with_size(n as usize);
    let mut out_vectors: Matrix<'static> = Matrix::with_bounds(0, n - 1, 0, n - 1).unwrap();
    out_values.initialized(&values).unwrap();
    out_vectors.initialized(&vectors).unwrap();
    for i in 0..n {
        out_values[i] *= scale;
    }
    if sort_descending {
        descending(&mut out_values, &mut out_vectors);
    }
    let status: Status =
        if out_values.values().iter().all(|x| x.is_finite()) && finite_matrix(&out_vectors) {
            Status::Ok
        } else {
            Status::NumericalError
        };
    EigenResult {
        status,
        iterations: rotations,
        eigen_values: Some(out_values),
        eigen_vectors: Some(out_vectors),
    }
}
pub(super) fn finite_matrix(matrix: &Matrix<'_>) -> bool {
    for i in 0..matrix.row_number() as usize {
        for j in 0..matrix.col_number() as usize {
            if !matrix.at(i, j).unwrap().is_finite() {
                return false;
            }
        }
    }
    true
}
/// Keeps native eigenvectors even for the eigenvalue-only entry point; `eigen_values(&a,true)`.
pub fn eigen_values(a: &Matrix<'_>, sort_descending: bool) -> EigenResult {
    jacobi(a, sort_descending)
}
/// Preserves the unsorted native rotation order; `spectral_decomposition(&a)`.
pub fn spectral_decomposition(a: &Matrix<'_>) -> EigenResult {
    jacobi(a, false)
}
fn powers(values: &Vector<'_>, power: f64) -> Option<Vector<'static>> {
    let mut result: Vector<'static> = Vector::with_size(values.length() as usize);
    for i in 0..values.length() {
        if values[i] < 0.0 && power != power.floor() {
            return None;
        }
        result[i] = if values[i] >= 0.0 {
            values[i].powf(power)
        } else {
            (-values[i]).powf(power)
        };
        if values[i] < 0.0 && power.abs() % 2.0 == 1.0 {
            result[i] = -result[i];
        }
        if !result[i].is_finite() {
            return None;
        }
    }
    Some(result)
}
fn reconstruct(v: &Matrix<'_>, d: &Vector<'_>) -> Option<Matrix<'static>> {
    let n: i32 = d.length();
    let mut out: Matrix<'static> = Matrix::with_bounds(0, n - 1, 0, n - 1).unwrap();
    for i in 0..n {
        for j in 0..n {
            let mut sum: f64 = 0.0;
            for k in 0..n {
                sum += v[(i, k)] * d[k] * v[(j, k)];
            }
            if !sum.is_finite() {
                return None;
            }
            out[(i, j)] = sum;
        }
    }
    Some(out)
}
/// Raises eigenvalues using native signed-integer handling; `matrix_power(&a,2.0)`.
pub fn matrix_power(a: &Matrix<'_>, power: f64) -> Option<Matrix<'static>> {
    if !power.is_finite() {
        return None;
    }
    let eigen: EigenResult = jacobi(a, false);
    if !eigen.is_done() {
        return None;
    }
    let d: Vector<'static> = powers(eigen.eigen_values.as_ref().unwrap(), power)?;
    reconstruct(eigen.eigen_vectors.as_ref().unwrap(), &d)
}
/// Computes the native spectral square root; `matrix_sqrt(&a)`.
pub fn matrix_sqrt(a: &Matrix<'_>) -> Option<Matrix<'static>> {
    matrix_power(a, 0.5)
}
/// Computes the native inverse spectral square root; `matrix_inv_sqrt(&a)`.
pub fn matrix_inv_sqrt(a: &Matrix<'_>) -> Option<Matrix<'static>> {
    matrix_power(a, -0.5)
}
