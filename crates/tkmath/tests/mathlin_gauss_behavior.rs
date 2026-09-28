//! Native result retention and helper validation beyond the legacy comparison suite.
use rustycad_tkmath::{
    math::{Matrix, Vector},
    math_lin::*,
    math_utils::Status,
};
#[test]
fn determinant_overflow_does_not_invalidate_solutions() {
    let mut a = Matrix::filled(0, 1, 0, 1, 0.0).unwrap();
    a[(0, 0)] = 1e200;
    a[(1, 1)] = 1e200;
    let rhs = Matrix::filled(-2, -1, 3, 4, 1e200).unwrap();
    let r = solve_multiple(&a, &rhs, 1e-15);
    assert!(r.is_done());
    assert!(r.determinant.is_none());
    assert_eq!(r.solutions.unwrap()[(0, 0)], 1.0);
    let f = lu(&a, 1e-15);
    assert!(bool::from(&f));
    assert_eq!(f.condition_estimate, Some(1.0));
    assert_eq!(determinant(&a, 1e-15).status, Status::NumericalError);
    let wrong = Vector::with_size(1);
    assert_eq!(solve(&a, &wrong, 1e-15).status, Status::InvalidInput);
    let wrong = Matrix::filled(0, 0, 0, 0, 1.0).unwrap();
    assert_eq!(
        solve_multiple(&a, &wrong, 1e-15).status,
        Status::InvalidInput
    );
    assert_eq!(lu(&a, -1.0).status, Status::InvalidInput);
}
#[test]
fn linear_norm_helpers_preserve_state_and_empty_input_rules() {
    let mut scale: f64 = 0.0;
    let mut sum: f64 = 1.0;
    for x in [0.0, 3.0, 4.0] {
        assert!(utils::accumulate_norm(x, &mut scale, &mut sum));
    }
    assert_eq!(utils::norm(scale, sum), Some(5.0));
    assert!(!utils::accumulate_norm(f64::NAN, &mut scale, &mut sum));
    assert_eq!(utils::norm(scale, sum), Some(5.0));
    assert_eq!(utils::norm(-1.0, 1.0), None);
    assert_eq!(utils::norm(f64::MAX, 4.0), None);
    assert_eq!(utils::norm(0.0, 1.0), Some(0.0));
    assert_eq!(utils::relative_tolerance(-1.0, 2), -1.0);
    assert_eq!(utils::relative_tolerance(0.0, 2), 2.0 * f64::EPSILON);
    assert!(utils::is_finite_vector(&Vector::with_size(0)));
    assert!(utils::is_finite_matrix(
        &Matrix::filled(0, 0, 0, 0, 1.0).unwrap()
    ));
}
