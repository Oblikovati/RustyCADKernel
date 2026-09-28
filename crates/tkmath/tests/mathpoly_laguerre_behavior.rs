//! Supplemental Laguerre entry-point and native complex-arithmetic branches.
use rustycad_tkmath::{math_poly::*, math_utils::Status};
const TOL: f64 = LAGUERRE_TOLERANCE;
const ITER: u32 = LAGUERRE_MAX_ITERATIONS;
#[test]
fn laguerre_rejects_degree_and_leading_coefficient_domain() {
    let c: [f64; 3] = [-1.0, 0.0, 1.0];
    assert_eq!(laguerre(&c, -1, TOL, ITER).status, Status::InvalidInput);
    assert_eq!(laguerre(&c, 0, TOL, ITER).status, Status::InvalidInput);
    assert_eq!(
        laguerre(&[1.0, 2.0, 0.0], 2, TOL, ITER).status,
        Status::InvalidInput
    );
    assert_eq!(
        laguerre(&[0.0; 22], 21, TOL, ITER).status,
        Status::InvalidInput
    );
    assert_eq!(laguerre(&c, 2, -1.0, ITER).status, Status::InvalidInput);
    assert_eq!(
        laguerre(&c, 2, f64::INFINITY, ITER).status,
        Status::InvalidInput
    );
}
#[test]
fn laguerre_rejects_slices_shorter_than_the_degree() {
    // Native code would read past the caller's array; Rust reports invalid input.
    assert_eq!(
        laguerre(&[-1.0, 0.0], 2, TOL, ITER).status,
        Status::InvalidInput
    );
}
#[test]
fn laguerre_ignores_coefficients_past_the_degree() {
    let result: PolyResult = laguerre(&[-2.0, 1.0, f64::NAN], 1, TOL, ITER);
    assert!(result.is_done());
    assert_eq!((result.nb_roots, result.roots[0]), (1, 2.0));
}
#[test]
fn laguerre_n_uses_the_slice_length_as_native_size() {
    assert_eq!(laguerre_n(&[1.0], TOL, ITER).status, Status::InvalidInput);
    assert_eq!(
        laguerre_n(&[1.0; 22], TOL, ITER).status,
        Status::InvalidInput
    );
    let result: GeneralPolyResult = laguerre_n(&[-1.0, 0.0, 1.0], TOL, ITER);
    assert_eq!(
        (result.nb_roots, result.roots[0], result.roots[1]),
        (2, -1.0, 1.0)
    );
}
#[test]
fn quintic_and_sextic_fall_back_on_zero_leading_coefficient() {
    let quintic_result: GeneralPolyResult = quintic(0.0, 1.0, -10.0, 35.0, -50.0, 24.0);
    assert_eq!(quintic_result.nb_roots, 4);
    let sextic_result: GeneralPolyResult = sextic(0.0, 0.0, 0.0, 0.0, 1.0, -3.0, 2.0);
    assert_eq!((sextic_result.nb_roots, sextic_result.roots[1]), (2, 2.0));
}
#[test]
fn octic_uses_default_settings() {
    let result: GeneralPolyResult = octic(&[1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0]);
    assert!(result.is_done());
    assert_eq!((result.nb_roots, result.nb_complex_roots), (0, 8));
}
#[test]
fn complex_operations_follow_libgcc_and_glibc() {
    let z: Complex = Complex::new(3.0, 4.0);
    assert_eq!((z.abs(), z.norm()), (5.0, 25.0));
    assert_eq!(z.conj(), Complex::new(3.0, -4.0));
    assert_eq!(Complex::new(-4.0, 0.0).sqrt(), Complex::new(0.0, 2.0));
    assert_eq!(z * Complex::new(0.0, 1.0), Complex::new(-4.0, 3.0));
    assert_eq!(z / Complex::new(0.0, 2.0), Complex::new(2.0, -1.5));
    assert_eq!(z + 1.0, Complex::new(4.0, 4.0));
    assert_eq!(z - Complex::new(3.0, 4.0), Complex::new(0.0, 0.0));
    assert_eq!(2.0 * z, Complex::new(6.0, 8.0));
    assert_eq!(Complex::polar(2.0, 0.0), Complex::new(2.0, 0.0));
    assert!(!Complex::new(f64::NAN, 0.0).is_finite());
}
#[test]
fn complex_infinity_recovery_matches_annex_g() {
    let infinite: Complex = Complex::new(f64::INFINITY, f64::NAN) * Complex::new(1.0, 1.0);
    assert!(infinite.real().is_infinite());
    let quotient: Complex = Complex::new(1.0, 0.0) / Complex::new(0.0, 0.0);
    assert_eq!(quotient.real(), f64::INFINITY);
    let vanishing: Complex = Complex::new(1.0, 1.0) / Complex::new(f64::INFINITY, 0.0);
    assert_eq!((vanishing.real(), vanishing.imag()), (0.0, 0.0));
    // A real addend keeps a negative-zero imaginary part.
    assert!((Complex::new(1.0, -0.0) + 1.0).imag().is_sign_negative());
}
