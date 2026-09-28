//! LU failure and validation-order contracts beyond the legacy declarations.
use rustycad_tkmath::math::{Gauss, Matrix, Vector};
#[test]
fn singular_solver_and_matrix_inverse_keep_distinct_failure_contracts() {
    let mut zero = Matrix::filled(1, 2, 1, 2, 0.0).unwrap();
    let solver = Gauss::new(&zero).unwrap();
    assert!(!solver.is_done());
    assert_eq!(
        solver.determinant().unwrap_err().exception_type(),
        "StdFail_NotDone"
    );
    assert_eq!(zero.determinant().unwrap(), 0.0);
    assert_eq!(
        zero.invert().unwrap_err().exception_type(),
        "math_SingularMatrix"
    );
    assert_eq!(zero[(1, 1)], 0.0);
}
#[test]
fn solver_checks_dimensions_and_preserves_destination_on_rejection() {
    let mut identity = Matrix::filled(1, 2, 1, 2, 0.0).unwrap();
    identity.set_diag(1.0).unwrap();
    let solver = Gauss::new(&identity).unwrap();
    let mut short = Vector::filled(1, 1, 7.0).unwrap();
    assert_eq!(
        solver.solve(&mut short).unwrap_err().exception_type(),
        "Standard_DimensionError"
    );
    assert_eq!(short[1], 7.0);
    let rhs = Vector::filled(1, 2, 3.0).unwrap();
    assert_eq!(
        solver
            .solve_into(&rhs, &mut short)
            .unwrap_err()
            .exception_type(),
        "Standard_DimensionMismatch"
    );
    assert_eq!(short[1], 7.0);
    assert!(
        solver
            .invert(&mut Matrix::filled(1, 1, 1, 1, 0.0).unwrap())
            .is_err()
    );
}
#[test]
fn minimum_pivot_and_non_square_constructor_follow_native_rules() {
    let mut matrix = Matrix::filled(1, 2, 1, 2, 0.0).unwrap();
    matrix.set_diag(1e-21).unwrap();
    assert!(!Gauss::new(&matrix).unwrap().is_done());
    assert!(Gauss::with_min_pivot(&matrix, 1e-25).unwrap().is_done());
    let rectangle = Matrix::filled(1, 2, 1, 3, 0.0).unwrap();
    assert_eq!(
        Gauss::new(&rectangle).err().unwrap().exception_type(),
        "math_NotSquare"
    );
}
#[test]
fn diagonal_setter_preserves_declared_index_behavior() {
    let mut shifted = Matrix::filled(2, 3, 3, 4, 0.0).unwrap();
    assert_eq!(
        shifted.set_diag(1.0).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
    assert_eq!(shifted[(3, 3)], 0.0);
}
