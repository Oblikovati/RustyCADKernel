//! Persistent truncation is part of the native SVD solver's observable state.
use rustycad_tkmath::math::{Matrix, Svd, Vector};
#[test]
fn lowering_epsilon_does_not_restore_discarded_singular_values() {
    let mut diagonal: Matrix<'static> = Matrix::filled(1, 2, 1, 2, 0.0).unwrap();
    diagonal[(1, 1)] = 1.0;
    diagonal[(2, 2)] = 1e-8;
    let mut solver: Svd = Svd::new(&diagonal).unwrap();
    let rhs: Vector<'static> = Vector::filled(1, 2, 1.0).unwrap();
    let mut output: Vector<'static> = Vector::with_bounds(4, 5).unwrap();
    solver.solve(&rhs, &mut output, 1e-6).unwrap();
    assert_eq!(output[4], 1.0);
    assert_eq!(output[5], 0.0);
    solver.solve(&rhs, &mut output, 0.0).unwrap();
    assert_eq!(output[5], 0.0);
    let mut fresh: Svd = Svd::new(&diagonal).unwrap();
    fresh.solve(&rhs, &mut output, 0.0).unwrap();
    assert_eq!(output[5], 1e8);
}
#[test]
fn wide_zero_matrix_has_zero_minimum_norm_solution() {
    let zero: Matrix<'static> = Matrix::filled(-2, -1, 4, 7, 0.0).unwrap();
    let mut solver: Svd = Svd::new(&zero).unwrap();
    let rhs: Vector<'static> = Vector::filled(4, 5, 1.0).unwrap();
    let mut output: Vector<'static> = Vector::filled(-3, 0, 91.0).unwrap();
    solver.solve(&rhs, &mut output, 1e-6).unwrap();
    assert!(solver.is_done());
    for index in -3..=0 {
        assert_eq!(output[index], 0.0);
    }
}
