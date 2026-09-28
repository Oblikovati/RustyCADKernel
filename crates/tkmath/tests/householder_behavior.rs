//! Solution extraction and checked state contracts supplement the original declarations.
use rustycad_tkmath::math::{Householder, Matrix, Vector};
#[test]
fn range_constructor_selects_a_real_submatrix() {
    let mut padded = Matrix::filled(-2, 2, -2, 2, 91.0).unwrap();
    padded[(-1, -1)] = 1.0;
    padded[(-1, 0)] = 0.0;
    padded[(0, -1)] = 0.0;
    padded[(0, 0)] = 1.0;
    let mut rhs = Matrix::filled(3, 4, 1, 1, 0.0).unwrap();
    rhs[(3, 1)] = 2.0;
    rhs[(4, 1)] = 3.0;
    let solver = Householder::from_range(&padded, &rhs, [-1, 0, -1, 0], 1e-20).unwrap();
    assert!(solver.is_done());
    assert_eq!(solver.all_values().unwrap()[(1, 1)], 2.0);
    assert_eq!(solver.all_values().unwrap()[(2, 1)], 3.0);
}
#[test]
fn extraction_rejects_invalid_column_after_not_done_check() {
    let zero = Matrix::filled(1, 2, 1, 2, 0.0).unwrap();
    let rhs = Vector::filled(1, 2, 1.0).unwrap();
    let failed = Householder::from_vector(&zero, &rhs, 1e-20).unwrap();
    let mut destination = Vector::filled(5, 6, 91.0).unwrap();
    assert_eq!(
        failed
            .value(&mut destination, 0)
            .unwrap_err()
            .exception_type(),
        "StdFail_NotDone"
    );
    let mut identity = zero;
    identity.set_diag(1.0).unwrap();
    let solver = Householder::from_vector(&identity, &rhs, 1e-20).unwrap();
    assert_eq!(
        solver
            .value(&mut destination, 2)
            .unwrap_err()
            .exception_type(),
        "Standard_OutOfRange"
    );
    assert_eq!(destination[5], 91.0);
    solver.value(&mut destination, 1).unwrap();
    assert_eq!(destination[5], 1.0);
}
