//! Native dual visibility, count ordering, and independent convergence tolerances.
use rustycad_tkmath::math::{Matrix, Uzawa, Vector};
#[test]
fn failed_iteration_still_exposes_initialized_dual_variables() {
    let matrix: Matrix<'static> = Matrix::filled(1, 1, 1, 1, 1.0).unwrap();
    let rhs: Vector<'static> = Vector::filled(1, 1, -10.0).unwrap();
    let start: Vector<'static> = Vector::filled(1, 1, 0.0).unwrap();
    let solver: Uzawa =
        Uzawa::with_constraints(&matrix, &rhs, &start, 0, 1, [1e-6, 1e-6], 0).unwrap();
    assert!(!solver.is_done());
    assert!(solver.value().is_err());
    assert!(solver.error().is_err());
    assert!(solver.initial_error().is_err());
    assert!(solver.iterations().is_err());
    assert!(solver.inverse_constraints().is_err());
    let mut dual: Vector<'static> = Vector::filled(-3, -3, 71.0).unwrap();
    solver.copy_dual(&mut dual).unwrap();
    assert_eq!(dual[-3], 1.0);
}
#[test]
fn primal_convergence_with_unconverged_dual_exits_without_retrying() {
    let matrix: Matrix<'static> = Matrix::filled(1, 1, 1, 1, 1.0).unwrap();
    let rhs: Vector<'static> = Vector::filled(1, 1, -10.0).unwrap();
    let start: Vector<'static> = Vector::filled(1, 1, 0.0).unwrap();
    let solver: Uzawa =
        Uzawa::with_constraints(&matrix, &rhs, &start, 0, 1, [100.0, 0.0], 500).unwrap();
    assert!(!solver.is_done());
    let mut dual: Vector<'static> = Vector::filled(1, 1, 0.0).unwrap();
    solver.copy_dual(&mut dual).unwrap();
    assert!(dual[1] > 1.0);
}
#[test]
fn singular_equality_projection_propagates_crout_not_done() {
    let matrix: Matrix<'static> = Matrix::filled(1, 1, 1, 1, 0.0).unwrap();
    let vector: Vector<'static> = Vector::filled(1, 1, 0.0).unwrap();
    let error = Uzawa::new(&matrix, &vector, &vector).err().unwrap();
    assert_eq!(error.exception_type(), "StdFail_NotDone");
}
#[test]
fn unsafe_native_tall_storage_shape_is_rejected_with_dimensions() {
    let matrix: Matrix<'static> = Matrix::filled(1, 2, 1, 1, 1.0).unwrap();
    let rhs: Vector<'static> = Vector::filled(1, 2, 0.0).unwrap();
    let start: Vector<'static> = Vector::filled(1, 1, 0.0).unwrap();
    let error = Uzawa::new(&matrix, &rhs, &start).err().unwrap();
    assert_eq!(error.exception_type(), "Standard_DimensionError");
}
