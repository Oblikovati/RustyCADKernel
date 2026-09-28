//! Additional API and failure-path coverage for modern eigenanalysis.
use rustycad_tkmath::math::{Matrix, Vector};
use rustycad_tkmath::math_lin::*;
use rustycad_tkmath::math_utils::Status;
#[test]
fn spectral_aliases_and_powers_preserve_native_outputs() {
    let mut a = Matrix::filled(-2, -1, 3, 4, 0.0).unwrap();
    a[(-2, 3)] = 4.0;
    a[(-1, 4)] = 9.0;
    let j = jacobi(&a, false);
    let e = eigen_values(&a, false);
    let s = spectral_decomposition(&a);
    assert!(j.is_done());
    assert!(e.is_done());
    assert!(s.is_done());
    assert_eq!(j.iterations, e.iterations);
    assert_eq!(*s.eigen_values.unwrap().at(0).unwrap(), 9.0);
    let root = matrix_sqrt(&a).unwrap();
    let inv = matrix_inv_sqrt(&a).unwrap();
    assert_eq!(root[(0, 0)], 2.0);
    assert_eq!(root[(1, 1)], 3.0);
    assert_eq!(inv[(0, 0)], 0.5);
    assert_eq!(matrix_power(&a, 2.0).unwrap()[(1, 1)], 81.0);
    a[(-2, 3)] = -4.0;
    assert!(matrix_sqrt(&a).is_none());
    assert_eq!(matrix_power(&a, 3.0).unwrap()[(0, 0)], -64.0);
    assert!(matrix_power(&a, f64::NAN).is_none());
}
#[test]
fn singleton_signed_zero_and_absent_vectors() {
    let a = Matrix::filled(0, 0, 0, 0, -0.0).unwrap();
    let result = jacobi(&a, true);
    assert_eq!(
        result.eigen_values.unwrap()[0].to_bits(),
        (-0.0_f64).to_bits()
    );
    let empty = TridiagonalResult::default();
    assert!(!bool::from(&empty));
    let v = get_eigen_vector(&empty, 0).unwrap();
    assert_eq!(v.length(), 1);
    assert_eq!(v[0], 0.0);
    let d = Vector::filled(0, 0, 7.0).unwrap();
    let r = eigen_tridiagonal(&d, &Vector::with_size(0), 30);
    assert!(bool::from(&r));
    assert!(get_eigen_vector(&r, 1).is_err());
}
#[test]
fn invalid_matrices_and_nonfinite_couplings() {
    let rectangle = Matrix::filled(0, 1, 0, 2, 0.0).unwrap();
    assert_eq!(jacobi(&rectangle, true).status, Status::InvalidInput);
    let mut a = Matrix::filled(0, 1, 0, 1, 0.0).unwrap();
    a[(0, 1)] = 1.0;
    assert_eq!(jacobi(&a, true).status, Status::InvalidInput);
    a[(0, 0)] = f64::INFINITY;
    assert!(matrix_sqrt(&a).is_none());
    let d = Vector::filled(0, 1, 1.0).unwrap();
    let e = Vector::filled(0, 0, f64::NAN).unwrap();
    assert_eq!(eigen_tridiagonal(&d, &e, 30).status, Status::InvalidInput);
}
