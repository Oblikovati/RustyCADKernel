//! Supplemental Jacobi ordering, triangle selection, and failure regressions.
use rustycad_tkmath::math::{Jacobi, Matrix, Vector};
#[test]
fn repeated_eigenvalues_choose_the_last_equal_column() {
    let mut identity: Matrix<'static> = Matrix::filled(1, 3, 1, 3, 0.0).unwrap();
    identity.set_diag(1.0).unwrap();
    let solver: Jacobi = Jacobi::new(&identity).unwrap();
    let vectors: &Matrix<'static> = solver.vectors().unwrap();
    assert_eq!(vectors[(3, 1)], 1.0);
    assert_eq!(vectors[(1, 2)], 1.0);
    assert_eq!(vectors[(2, 3)], 1.0);
}
#[test]
fn lower_triangle_does_not_affect_eigenpairs() {
    let mut matrix: Matrix<'static> = Matrix::filled(1, 2, 1, 2, 1.0).unwrap();
    matrix[(2, 1)] = f64::NAN;
    let solver: Jacobi = Jacobi::new(&matrix).unwrap();
    assert!(solver.is_done());
    assert_eq!(solver.value(1).unwrap(), 2.0);
    assert_eq!(solver.value(2).unwrap(), 0.0);
}
#[test]
fn nan_upper_triangle_exhausts_sweeps_and_guards_results() {
    let mut matrix: Matrix<'static> = Matrix::filled(1, 2, 1, 2, 1.0).unwrap();
    matrix[(1, 2)] = f64::NAN;
    let solver: Jacobi = Jacobi::new(&matrix).unwrap();
    let mut output: Vector<'static> = Vector::filled(5, 6, 91.0).unwrap();
    assert!(!solver.is_done());
    assert_eq!(
        solver.values().err().unwrap().exception_type(),
        "StdFail_NotDone"
    );
    assert_eq!(
        solver.vectors().err().unwrap().exception_type(),
        "StdFail_NotDone"
    );
    assert_eq!(
        solver.value(0).unwrap_err().exception_type(),
        "StdFail_NotDone"
    );
    assert_eq!(
        solver.vector(0, &mut output).unwrap_err().exception_type(),
        "StdFail_NotDone"
    );
    assert_eq!(output[5], 91.0);
}
#[test]
fn nonsquare_input_is_rejected() {
    let matrix: Matrix<'static> = Matrix::with_bounds(1, 2, 1, 3).unwrap();
    assert_eq!(
        Jacobi::new(&matrix).err().unwrap().exception_type(),
        "math_NotSquare"
    );
}
