//! Validation and default-state branches supplement the unchanged legacy assertions.
use rustycad_tkmath::{
    math::{Matrix, Vector},
    math_lin::*,
    math_utils::Status,
};
fn system() -> (Matrix<'static>, Vector<'static>) {
    let mut a: Matrix<'static> = Matrix::filled(0, 1, 0, 0, 1.0).unwrap();
    a[(1, 0)] = 2.0;
    (a, Vector::filled(0, 1, 1.0).unwrap())
}
#[test]
fn native_method_fallthrough_and_result_boolean() {
    let (a, b) = system();
    assert!(!bool::from(&LeastSquaresResult::default()));
    let method: LeastSquaresMethod = LeastSquaresMethod::Invalid(4);
    assert_eq!(
        least_squares(&a, &b, method, 1e-15).status,
        Status::NotConverged
    );
    assert!(bool::from(&least_squares(
        &a,
        &b,
        LeastSquaresMethod::default(),
        1e-15
    )));
}
#[test]
fn weighting_rejects_wrong_shape_and_nonpositive_weights() {
    let (a, b) = system();
    let mut w: Vector<'static> = Vector::filled(0, 1, 0.0).unwrap();
    assert_eq!(
        weighted_least_squares(&a, &b, &w, LeastSquaresMethod::Qr, 1e-15).status,
        Status::InvalidInput
    );
    w[0] = f64::NAN;
    assert_eq!(
        weighted_least_squares(&a, &b, &w, LeastSquaresMethod::Qr, 1e-15).status,
        Status::InvalidInput
    );
    let wrong: Vector<'static> = Vector::with_size(1);
    assert_eq!(
        weighted_least_squares(&a, &wrong, &w, LeastSquaresMethod::Qr, 1e-15).status,
        Status::InvalidInput
    );
    assert_eq!(
        weighted_least_squares(&a, &b, &wrong, LeastSquaresMethod::Qr, 1e-15).status,
        Status::InvalidInput
    );
}
#[test]
fn regularization_rejects_invalid_grid_and_parameters() {
    let (a, b) = system();
    for lambda in [-1.0, f64::NAN, f64::INFINITY] {
        assert_eq!(
            regularized_least_squares(&a, &b, lambda, 1e-15).status,
            Status::InvalidInput
        );
    }
    for (lo, hi, n) in [
        (0.0, 1.0, 20),
        (2.0, 1.0, 20),
        (1.0, 2.0, 1),
        (f64::NAN, 1.0, 20),
        (1.0, f64::INFINITY, 20),
    ] {
        assert!(optimal_regularization(&a, &b, lo, hi, n).is_none());
    }
    assert_eq!(
        regularized_least_squares(&a, &b, 0.0, -1.0).status,
        Status::InvalidInput
    );
}
#[test]
fn zero_rhs_gcv_keeps_first_grid_point() {
    let (a, mut b) = system();
    b.init(0.0);
    assert_eq!(optimal_regularization(&a, &b, 0.01, 1.0, 20), Some(0.01));
    let wrong: Vector<'static> = Vector::with_size(1);
    assert!(optimal_regularization(&a, &wrong, 0.01, 1.0, 20).is_none());
    assert_eq!(
        least_squares(&a, &wrong, LeastSquaresMethod::Qr, 1e-15).status,
        Status::InvalidInput
    );
}
