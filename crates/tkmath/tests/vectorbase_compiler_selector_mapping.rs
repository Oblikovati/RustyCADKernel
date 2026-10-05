//! Checks the exact template overloads counted by the compiler crosswalk.
use rustycad_tkmath::math::{Matrix, Vector};

fn vector(lower: i32, values: &[f64]) -> Vector<'static> {
    let mut result = Vector::with_bounds(lower, lower + values.len() as i32 - 1).unwrap();
    for (offset, value) in values.iter().enumerate() {
        *result.change_at(offset).unwrap() = *value;
    }
    result
}

fn values(value: &Vector<'_>) -> Vec<f64> {
    (value.lower()..=value.upper())
        .map(|index| value[index])
        .collect()
}

fn assert_vector(value: &Vector<'_>, lower: i32, expected: &[f64]) {
    assert_eq!(value.lower(), lower);
    assert_eq!(value.upper(), lower + expected.len() as i32 - 1);
    assert_eq!(values(value), expected);
}

#[test]
fn compiler_mapped_vectorbase_methods_match_native_probe() {
    let left = vector(2, &[1.25, -2.0, 4.0]);
    let right = vector(-1, &[2.0, 3.0, -1.0]);
    let mut initialized = Vector::with_bounds(7, 9).unwrap();
    initialized.initialized(&right).unwrap();
    assert_vector(&initialized, 7, &[2.0, 3.0, -1.0]);
    let mut assigned = Vector::with_bounds(7, 9).unwrap();
    assigned.initialized(&left).unwrap();
    assert_vector(&assigned, 7, &[1.25, -2.0, 4.0]);
    verify_returned_arithmetic(&left, &right);
    verify_in_place_and_set(&left, &right);
    verify_matrix_products(&left);
}

fn verify_returned_arithmetic(left: &Vector<'_>, right: &Vector<'_>) {
    assert_vector(&left.added(right).unwrap(), 2, &[3.25, 1.0, 3.0]);
    assert_vector(&left.subtracted(right).unwrap(), 2, &[-0.75, -5.0, 5.0]);
    assert_eq!(left.dot(right).unwrap(), -7.5);
}

fn verify_in_place_and_set(left: &Vector<'_>, right: &Vector<'_>) {
    let mut added = left.clone();
    added.add(right).unwrap();
    assert_vector(&added, 2, &[3.25, 1.0, 3.0]);
    let mut subtracted = left.clone();
    subtracted.subtract(right).unwrap();
    assert_vector(&subtracted, 2, &[-0.75, -5.0, 5.0]);
    let mut set = vector(2, &[0.0, 0.0, 0.0]);
    set.set(2, 4, right).unwrap();
    assert_vector(&set, 2, &[2.0, 3.0, -1.0]);
    let mut sum = vector(5, &[0.0, 0.0, 0.0]);
    sum.set_sum(left, right).unwrap();
    assert_vector(&sum, 5, &[3.25, 1.0, 3.0]);
    let mut difference = vector(-3, &[0.0, 0.0, 0.0]);
    difference.set_difference(left, right).unwrap();
    assert_vector(&difference, -3, &[-0.75, -5.0, 5.0]);
}

fn verify_matrix_products(left: &Vector<'_>) {
    let matrix = sample_matrix();
    verify_matrix_vector(left, &matrix);
    verify_row_products(&matrix);
    verify_transpose_vector(left, &matrix);
}

fn sample_matrix() -> Matrix<'static> {
    let mut matrix = Matrix::with_bounds(-1, 0, 5, 7).unwrap();
    for (offset, value) in [1.0, 2.0, 3.0, 4.0, 5.0, 6.0].iter().enumerate() {
        *matrix.change_at(offset / 3, offset % 3).unwrap() = *value;
    }
    assert_eq!((matrix.lower_row(), matrix.upper_row()), (-1, 0));
    assert_eq!((matrix.lower_col(), matrix.upper_col()), (5, 7));
    matrix
}

fn verify_matrix_vector(left: &Vector<'_>, matrix: &Matrix<'_>) {
    let mut result = vector(10, &[0.0, 0.0]);
    result.set_matrix_vector(matrix, left).unwrap();
    assert_vector(&result, 10, &[9.25, 19.0]);
}

fn verify_row_products(matrix: &Matrix<'_>) {
    let row = vector(5, &[-1.0, 2.0]);
    let mut vector_matrix = vector(-4, &[0.0, 0.0, 0.0]);
    vector_matrix.set_vector_matrix(&row, matrix).unwrap();
    assert_vector(&vector_matrix, -4, &[7.0, 8.0, 9.0]);
    verify_transpose_row(&row, matrix);
}

fn verify_transpose_row(row: &Vector<'_>, matrix: &Matrix<'_>) {
    let mut result = vector(4, &[0.0, 0.0, 0.0]);
    result.set_transpose_matrix_vector(matrix, row).unwrap();
    assert_vector(&result, 4, &[7.0, 8.0, 9.0]);
}

fn verify_transpose_vector(left: &Vector<'_>, matrix: &Matrix<'_>) {
    let mut transpose_right = vector(8, &[0.0, 0.0]);
    transpose_right
        .set_vector_transpose_matrix(left, matrix)
        .unwrap();
    assert_vector(&transpose_right, 8, &[9.25, 19.0]);
}
