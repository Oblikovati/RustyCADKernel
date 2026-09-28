//! Supplemental Laguerre-side MathPoly::Utils behavior pinned against native OCCT.
use rustycad_tkmath::math_poly::utils::{self, IterationResult};
use rustycad_tkmath::math_poly::{Complex, PolyResult};
use rustycad_tkmath::math_utils::Status;

#[test]
fn complex_evaluation_and_residual() {
    let e = utils::evaluate_complex(&[1.0, 0.0, 1.0], 2, Complex::new(0.0, 1.0));
    assert_eq!(e.value, Complex::new(0.0, 0.0));
    assert_eq!(e.derivative, Complex::new(0.0, 2.0));
    assert_eq!(e.second_derivative, Complex::new(2.0, 0.0));
    assert_eq!(e.magnitude, 2.0);
    assert!(utils::has_small_complex_residual(
        &[1.0, 0.0, 1.0],
        2,
        Complex::new(0.0, 1.0),
        1e-12
    ));
    assert!(!utils::has_small_complex_residual(
        &[1.0, 0.0, 1.0],
        2,
        Complex::new(1.0, 0.0),
        1e-12
    ));
    assert!(utils::is_finite(Complex::new(1.0, 0.0)));
    assert!(!utils::is_finite(Complex::new(f64::NAN, 0.0)));
}

#[test]
fn laguerre_iteration_reports_the_native_iteration_index() {
    let c = [-2.0, 0.0, 1.0];
    let exact: IterationResult = utils::laguerre_iteration(&c, 2, Complex::new(0.0, 0.0), 1e-12, 0);
    assert_eq!(
        (exact.status, exact.nb_iterations, exact.root_error),
        (Status::MaxIterations, 0, None)
    );
    let found: IterationResult =
        utils::laguerre_iteration(&c, 2, Complex::new(1.0, 0.0), 1e-12, 100);
    assert_eq!(found.status, Status::Ok);
    assert!((found.root.real() - 2.0_f64.sqrt()).abs() < 1e-15);
    assert!(found.nb_iterations > 0 && found.root_error.is_some());
    // A NaN tolerance never accepts a candidate, so the budget runs out.
    let nan: IterationResult =
        utils::laguerre_iteration(&c, 2, Complex::new(1.0, 0.0), f64::NAN, 5);
    assert_eq!((nan.status, nan.nb_iterations), (Status::MaxIterations, 5));
    let root: IterationResult = utils::find_root(&[-6.0, 11.0, -6.0, 1.0], 3, 1e-12, 100);
    assert_eq!(root.status, Status::Ok);
    assert_eq!(
        utils::find_root(&c, 2, 1e-12, 0).status,
        Status::MaxIterations
    );
    assert_eq!(
        (
            utils::LAGUERRE_NB_STARTS,
            utils::MULTIPLICITY_REFINEMENT_MAX_ITERATIONS
        ),
        (16, 64)
    );
    assert_eq!(utils::DISTINCT_ROOT_BISECTION_ITERATIONS, 200);
}

#[test]
fn deflation_divides_only_small_remainders() {
    let mut c = [-6.0, 11.0, -6.0, 1.0];
    let mut degree: i32 = 3;
    assert!(!utils::deflate_real(&mut c, &mut degree, 5.0, 1e-12));
    assert_eq!((degree, c), (3, [-6.0, 11.0, -6.0, 1.0]));
    assert!(utils::deflate_real(&mut c, &mut degree, 1.0, 1e-12));
    assert_eq!((degree, &c[..3]), (2, &[6.0, -5.0, 1.0][..]));
    // (x^2 + 1)(x - 2) = x^3 - 2x^2 + x - 2.
    let mut cubic = [-2.0, 1.0, -2.0, 1.0];
    let mut degree: i32 = 3;
    assert!(utils::deflate_complex(
        &mut cubic,
        &mut degree,
        Complex::new(0.0, 1.0),
        1e-12
    ));
    assert_eq!((degree, &cubic[..2]), (1, &[-2.0, 1.0][..]));
}

#[test]
fn multiplicity_checks_refine_the_root() {
    // (x - 1)^2 (x + 3).
    let c = [3.0, -5.0, 1.0, 1.0];
    let mut root: f64 = 1.0 + 1e-9;
    assert!(utils::has_multiplicity(&c, 3, &mut root, 2));
    assert!((root - 1.0).abs() < 1e-12);
    let mut simple: f64 = -3.0;
    assert!(!utils::has_multiplicity(&c, 3, &mut simple, 2));
    // (x^2 + 1)^2 = x^4 + 2x^2 + 1.
    let quartic = [1.0, 0.0, 2.0, 0.0, 1.0];
    let mut z: Complex = Complex::new(1e-9, 1.0);
    assert!(utils::has_complex_multiplicity(&quartic, 4, &mut z, 2));
    assert!(z.real().abs() < 1e-12 && (z.imag() - 1.0).abs() < 1e-12);
}

#[test]
fn consolidation_merges_clusters_with_verified_multiplicity() {
    let c = [3.0, -5.0, 1.0, 1.0];
    let mut result: PolyResult = PolyResult {
        nb_roots: 3,
        ..Default::default()
    };
    result.roots[..3].copy_from_slice(&[1.0 + 1e-9, -3.0, 1.0 - 1e-9]);
    result.multiplicities[..3].copy_from_slice(&[1, 1, 1]);
    utils::consolidate_real_roots(&mut result, 1e-12, &c, 3, 0);
    assert_eq!(result.nb_roots, 2);
    assert_eq!(result.roots[0], -3.0);
    assert_eq!(result.multiplicities[..2], [1, 2]);
    assert!((result.roots[1] - 1.0).abs() < 1e-12);
    let quartic = [1.0, 0.0, 2.0, 0.0, 1.0];
    let mut pairs: PolyResult = PolyResult {
        nb_complex_roots: 4,
        ..Default::default()
    };
    let (a, b) = (Complex::new(1e-9, 1.0), Complex::new(-1e-9, 1.0));
    pairs.complex_roots[..4].copy_from_slice(&[a, a.conj(), b.conj(), b]);
    utils::consolidate_complex_roots(&mut pairs, 1e-12, &quartic, 4, 0);
    assert_eq!(pairs.nb_complex_roots, 2);
    assert_eq!(pairs.complex_multiplicities[..2], [2, 2]);
    assert_eq!(pairs.complex_roots[1], pairs.complex_roots[0].conj());
}
