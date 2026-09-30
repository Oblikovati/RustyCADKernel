//! Direct math_EigenValuesSearcher_Test.cxx translation at pinned OCCT 3d097a0.
#![allow(non_snake_case, unused_mut)]
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkmath::math::{EigenValuesSearcher, Matrix, Vector};
fn assert_near(left: f64, right: f64, tolerance: f64) {
    assert!((left - right).abs() <= tolerance, "{left} != {right}");
}
fn create_tridiagonal_matrix(
    diagonal: &Array1<'_, f64>,
    subdiagonal: &Array1<'_, f64>,
    matrix: &mut Matrix<'_>,
) {
    matrix.init(0.0);
    for index in 1..=diagonal.len() as i32 {
        matrix[(index, index)] = diagonal[index];
    }
    for index in 2..=diagonal.len() as i32 {
        matrix[(index, index - 1)] = subdiagonal[index];
        matrix[(index - 1, index)] = subdiagonal[index];
    }
}
fn verify_eigen_pair(matrix: &Matrix<'_>, value: f64, vector: &Vector<'_>, tolerance: f64) -> bool {
    let mut result: Vector<'static> = Vector::with_bounds(1, matrix.row_number()).unwrap();
    for row in 1..=matrix.row_number() {
        let mut sum: f64 = 0.0;
        for col in 1..=matrix.row_number() {
            sum += matrix[(row, col)] * vector[col];
        }
        result[row] = sum;
    }
    for row in 1..=matrix.row_number() {
        if (result[row] - value * vector[row]).abs() > tolerance {
            return false;
        }
    }
    true
}
fn are_orthogonal(left: &Vector<'_>, right: &Vector<'_>, tolerance: f64) -> bool {
    let mut sum: f64 = 0.0;
    for index in 1..=left.length() {
        sum += left[index] * right[index];
    }
    sum.abs() < tolerance
}
fn vector_norm(vector: &Vector<'_>) -> f64 {
    let mut sum: f64 = 0.0;
    for index in 1..=vector.length() {
        sum += vector[index] * vector[index];
    }
    sum.sqrt()
}
include!("math_eigen_values_searcher/suites_0.rs");
include!("math_eigen_values_searcher/suites_1.rs");
include!("math_eigen_values_searcher/suites_2.rs");
include!("math_eigen_values_searcher/suites_3.rs");
include!("math_eigen_values_searcher/suites_4.rs");
include!("math_eigen_values_searcher/suites_5.rs");
include!("math_eigen_values_searcher/suites_6.rs");
include!("math_eigen_values_searcher/suites_7.rs");
include!("math_eigen_values_searcher/suites_8.rs");
include!("math_eigen_values_searcher/suites_9.rs");
include!("math_eigen_values_searcher/suites_10.rs");
