//! Supplemental direct-solver status, representation and multiplicity branches.
use rustycad_tkmath::{math_poly::*, math_utils::Status};
#[test]
fn linear_degeneracy_and_unrepresentable_root() {
    assert_eq!(linear(0.0, 0.0).status, Status::InfiniteSolutions);
    assert_eq!(linear(0.0, 1.0).status, Status::NoSolution);
    assert_eq!(linear(1e-300, 1e300).status, Status::NumericalError);
    assert_eq!(linear(f64::NAN, 1.0).status, Status::InvalidInput);
    let result: PolyResult = linear(2.0, -4.0);
    assert!(bool::from(&result));
    assert_eq!(result[0], 2.0);
    assert_eq!(result.nb_real_roots_with_multiplicity(), 1);
}
#[test]
fn complex_storage_and_result_defaults() {
    let z: Complex = Complex::new(2.0, -3.0);
    assert_eq!((z.real(), z.imag()), (2.0, -3.0));
    let result: GeneralPolyResult = GeneralPolyResult::default();
    assert!(!result.is_done());
    assert_eq!(result.nb_roots, 0);
    assert_eq!(result.nb_complex_roots, 0);
    assert_eq!(result.complex_roots[0], Complex::new(0.0, 0.0));
}
#[test]
fn repeated_roots_keep_native_multiplicities() {
    let quadratic: PolyResult = quadratic(1.0, -4.0, 4.0);
    assert_eq!(quadratic.multiplicities[0], 2);
    let cubic: PolyResult = cubic(1.0, -3.0, 3.0, -1.0);
    assert_eq!(cubic.nb_real_roots_with_multiplicity(), 3);
    let quartic: PolyResult = quartic(1.0, -8.0, 22.0, -24.0, 9.0);
    assert_eq!(quartic.nb_roots, 2);
    assert_eq!(&quartic.multiplicities[..2], &[2, 2]);
}
#[test]
fn quadratic_signed_zero_and_nonfinite_input() {
    let result: PolyResult = quadratic(1.0, 0.0, 0.0);
    assert!(result.is_done());
    assert_eq!(result[0].to_bits(), (-0.0_f64).to_bits());
    assert_eq!(
        quadratic(1.0, f64::INFINITY, 0.0).status,
        Status::InvalidInput
    );
    assert_eq!(quartic(0.0, 0.0, 0.0, 0.0, 1.0).status, Status::NoSolution);
    assert_eq!(cubic(0.0, 0.0, 1.0, -2.0)[0], 2.0);
}
