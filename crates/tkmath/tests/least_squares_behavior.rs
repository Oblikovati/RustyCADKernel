//! Normal-equation solver storage behavior beyond the legacy declarations.
use rustycad_tkmath::math::{GaussLeastSquares, Matrix, Vector};
fn coefficient_matrix() -> Matrix<'static> {
    let mut matrix = Matrix::filled(1, 2, 1, 2, 1.0).unwrap();
    matrix[(1, 1)] = 2.0;
    matrix[(2, 2)] = 2.0;
    matrix
}
#[test]
fn oversized_buffers_keep_their_unprocessed_tail() {
    let solver = GaussLeastSquares::new(&coefficient_matrix()).unwrap();
    let mut rhs = Vector::filled(-1, 2, 3.0).unwrap();
    rhs[2] = 57.0;
    let mut output = Vector::filled(5, 8, 91.0).unwrap();
    solver.solve_into(&rhs, &mut output).unwrap();
    assert!((output[5] - 1.0).abs() < 1e-12);
    assert!((output[6] - 1.0).abs() < 1e-12);
    assert_eq!((output[7], output[8]), (91.0, 91.0));
}
#[test]
fn aliased_rhs_retains_sequential_transpose_product_writes() {
    let solver = GaussLeastSquares::new(&coefficient_matrix()).unwrap();
    let mut rhs = Vector::filled(1, 2, 3.0).unwrap();
    solver.solve_in_place(&mut rhs).unwrap();
    assert!((rhs[1] + 5.0 / 3.0).abs() < 1e-12);
    assert!((rhs[2] - 13.0 / 3.0).abs() < 1e-12);
}
#[test]
fn singular_normal_equations_reject_solving_before_mutation() {
    let solver = GaussLeastSquares::new(&Matrix::filled(1, 2, 1, 2, 0.0).unwrap()).unwrap();
    let rhs = Vector::filled(1, 2, 3.0).unwrap();
    let mut output = Vector::filled(1, 2, 91.0).unwrap();
    assert!(!solver.is_done());
    assert_eq!(
        solver
            .solve_into(&rhs, &mut output)
            .unwrap_err()
            .exception_type(),
        "StdFail_NotDone"
    );
    assert_eq!(output[1], 91.0);
}
