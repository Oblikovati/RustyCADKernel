//! Supplemental checks for the native protected `math_Matrix` bound mutators.
use rustycad_tkmath::math::Matrix;

#[test]
fn set_lower_row_preserves_positional_cells() {
    let mut matrix = Matrix::with_bounds(1, 2, 1, 2).unwrap();
    matrix[(1, 1)] = 10.0;
    matrix[(2, 1)] = 30.0;
    matrix.set_lower_row(10);
    assert_eq!((matrix.lower_row(), matrix.upper_row()), (10, 11));
    assert_eq!((matrix[(10, 1)], matrix[(11, 1)]), (10.0, 30.0));
}

#[test]
fn set_lower_col_preserves_positional_cells() {
    let mut matrix = Matrix::with_bounds(1, 2, 1, 2).unwrap();
    matrix[(1, 1)] = 10.0;
    matrix[(1, 2)] = 20.0;
    matrix.set_lower_col(-2);
    assert_eq!((matrix.lower_col(), matrix.upper_col()), (-2, -1));
    assert_eq!((matrix[(1, -2)], matrix[(1, -1)]), (10.0, 20.0));
}
