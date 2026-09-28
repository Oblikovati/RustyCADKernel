//! Supplemental MathUtils_Poly.hxx regressions at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use rustycad_tkmath::math_utils::*;
#[test]
fn horner_orders_and_derivative() {
    assert_eq!(eval_poly(&[2.0, 3.0, 4.0], 2.0), 24.0);
    assert_eq!(eval_poly_desc(&[4.0, 3.0, 2.0], 2.0), 24.0);
    assert_eq!(eval_poly_deriv(&[2.0, 3.0, 4.0], 2.0), (24.0, 19.0));
    assert!(eval_poly(&[], 1.0).is_nan());
    assert!(eval_poly_desc(&[1.0], f64::INFINITY).is_nan());
    assert!(eval_poly_deriv(&[], 1.0).1.is_nan());
}
#[test]
fn refinement_preserves_stopping_and_invalid_inputs() {
    assert_eq!(refine_poly_root(&[-2.0, 1.0], 0.0, 5), 2.0);
    assert_eq!(refine_poly_root_desc(&[1.0, -2.0], 0.0, 5), 2.0);
    assert_eq!(refine_poly_root(&[1.0], 3.0, 5), 3.0);
    assert_eq!(refine_poly_root(&[f64::INFINITY], 3.0, 5), 3.0);
    assert_eq!(refine_poly_root(&[-2.0, 1.0], 0.0, 0), 0.0);
    assert!(refine_poly_root(&[], 0.0, 5).is_nan());
    assert!(refine_poly_root_desc(&[0.0; 6], 0.0, 5).is_nan());
}
#[test]
fn duplicate_compaction_preserves_partial_writes() {
    let mut roots: [f64; 5] = [2.0, 1.0, 1.0, 3.0, 3.0];
    sort_roots(&mut roots);
    assert_eq!(remove_duplicate_roots(&mut roots, 1e-10), 3);
    assert_eq!(&roots[..3], &[1.0, 2.0, 3.0]);
    let mut invalid: [f64; 4] = [1.0, 1.0, 2.0, f64::NAN];
    assert_eq!(remove_duplicate_roots(&mut invalid, 0.0), 0);
    assert_eq!(invalid[1], 2.0);
    assert_eq!(remove_duplicate_roots(&mut [1e10, 1e10 + 0.5], 1e-10), 1);
    assert_eq!(remove_duplicate_roots(&mut [1.0], -1.0), 0);
}
#[test]
fn depressed_polynomial_coefficients() {
    assert_eq!(depress_cubic(3.0, 3.0, 1.0), (0.0, 0.0, 1.0));
    assert_eq!(depress_quartic(4.0, 6.0, 4.0, 1.0), (0.0, 0.0, 0.0, 1.0));
    let (p, q, shift) = depress_cubic(2.0, 3.0, 4.0);
    let t: f64 = 5.0;
    assert!((eval_poly(&[4.0, 3.0, 2.0, 1.0], t - shift) - (t * t * t + p * t + q)).abs() < 1e-12);
}
