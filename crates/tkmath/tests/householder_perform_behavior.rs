//! Protected `math_Householder::Perform` reuse and status transitions.
use rustycad_tkmath::math::{Householder, Matrix};

fn identity() -> Matrix<'static> {
    let mut matrix = Matrix::filled(1, 2, 1, 2, 0.0).unwrap();
    matrix[(1, 1)] = 1.0;
    matrix[(2, 2)] = 1.0;
    matrix
}

fn rhs(first: f64, second: f64) -> Matrix<'static> {
    let mut matrix = Matrix::filled(1, 2, 1, 1, 0.0).unwrap();
    matrix[(1, 1)] = first;
    matrix[(2, 1)] = second;
    matrix
}

#[test]
fn perform_reuses_storage_after_success_small_pivot_and_recovery() {
    let identity = identity();
    let initial_rhs = rhs(1.0, 2.0);
    let mut solver = Householder::from_matrix(&identity, &initial_rhs, 1e-20).unwrap();
    let zero = Matrix::filled(1, 2, 1, 2, 0.0).unwrap();
    solver.perform(&zero, &initial_rhs, 1e-20).unwrap();
    assert!(!solver.is_done());
    assert_eq!(
        solver.all_values().err().unwrap().exception_type(),
        "StdFail_NotDone"
    );
    let updated_rhs = rhs(3.0, 4.0);
    solver.perform(&identity, &updated_rhs, 1e-20).unwrap();
    assert_eq!(solver.all_values().unwrap()[(1, 1)], 3.0);
    assert_eq!(solver.all_values().unwrap()[(2, 1)], 4.0);
}
