//! The failure paths that replace the removed panicking math operators: the Result
//! methods name the native exception, and indexing out of bounds is a documented
//! precondition panic that names the index.
use rustycad_tkmath::math::{DoubleTab, Matrix, Vector};

fn vector(lower: i32, upper: i32) -> Vector<'static> {
    let mut vector = Vector::with_bounds(lower, upper).unwrap();
    vector.init(2.0);
    vector
}

#[test]
fn vector_sum_and_difference_of_different_lengths_are_dimension_errors() {
    let (left, right) = (vector(1, 3), vector(1, 4));
    let sum = left.added(&right).unwrap_err();
    assert_eq!(sum.exception_type(), "Standard_DimensionError");
    let difference = left.subtracted(&right).unwrap_err();
    assert_eq!(difference.exception_type(), "Standard_DimensionError");
    let mut target = vector(1, 3);
    assert!(target.add(&right).is_err());
    assert!(target.subtract(&right).is_err());
    assert_eq!(left.added(&vector(5, 7)).unwrap()[1], 4.0);
}

#[test]
fn vector_dot_of_different_lengths_and_division_by_zero_fail() {
    let left = vector(1, 3);
    assert!(left.dot(&vector(1, 2)).is_err());
    assert_eq!(left.dot(&vector(1, 3)).unwrap(), 12.0);
    assert_eq!(
        left.divided(0.0).unwrap_err().exception_type(),
        "Standard_DivideByZero"
    );
    let mut target = vector(1, 3);
    assert!(target.divide(0.0).is_err());
    assert_eq!(left.divided(2.0).unwrap()[3], 1.0);
}

#[test]
fn row_vector_times_matrix_checks_the_row_count() {
    let row = vector(1, 3);
    let matrix = Matrix::with_bounds(1, 2, 1, 2).unwrap();
    assert!(row.multiplied_matrix(&matrix).is_err());
}

#[test]
fn matrix_operations_of_mismatched_shapes_fail() {
    let square = Matrix::with_bounds(1, 2, 1, 2).unwrap();
    let wide = Matrix::with_bounds(1, 3, 1, 4).unwrap();
    assert!(square.added(&wide).is_err());
    assert!(square.subtracted(&wide).is_err());
    assert!(square.multiplied_matrix(&wide).is_err());
    let mut target = Matrix::with_bounds(1, 2, 1, 2).unwrap();
    assert!(target.add(&wide).is_err());
    assert!(target.subtract(&wide).is_err());
    assert!(target.multiply_matrix(&wide).is_err());
    assert!(square.multiplied_vector(&vector(1, 3)).is_err());
    assert_eq!(
        square
            .divided(0.0)
            .err()
            .map(|e| e.exception_type().to_string()),
        Some("Standard_DivideByZero".to_string())
    );
    assert!(target.divide(0.0).is_err());
}

#[test]
fn unchecked_indices_use_the_result_accessors() {
    let mut row = vector(1, 3);
    assert!(row.value(4).is_err());
    assert!(row.change_value(0).is_err());
    let matrix = Matrix::with_bounds(1, 2, 1, 2).unwrap();
    assert!(matrix.value(3, 1).is_err());
    let table = DoubleTab::with_bounds(1, 2, 1, 2).unwrap();
    assert!(table.value(1, 3).is_err());
}

#[test]
#[should_panic(expected = "math_VectorBase index 4 outside [1, 3]")]
fn vector_index_outside_the_bounds_is_a_precondition_panic() {
    let row = vector(1, 3);
    let _ = row[4];
}

#[test]
#[should_panic(expected = "math_DoubleTab")]
fn matrix_index_outside_the_bounds_is_a_precondition_panic() {
    let matrix = Matrix::with_bounds(1, 2, 1, 2).unwrap();
    let _ = matrix[(0, 1)];
}
