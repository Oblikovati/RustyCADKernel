//! Supplemental Laguerre-side MathPoly::Utils behavior pinned against native OCCT.
use rustycad_tkmath::math_poly::utils::{self, IterationResult};
use rustycad_tkmath::math_poly::{Complex, PolyResult};
use rustycad_tkmath::math_utils::Status;

#[test]
fn complex_evaluation_and_residual() {
    let e = utils::evaluate_complex(&[1.0, 0.0, 1.0], 2, Complex::new(0.0, 1.0)).unwrap();
    assert_eq!(e.value, Complex::new(0.0, 0.0));
    assert_eq!(e.derivative, Complex::new(0.0, 2.0));
    assert_eq!(e.second_derivative, Complex::new(2.0, 0.0));
    assert_eq!(e.magnitude, 2.0);
    assert!(
        utils::has_small_complex_residual(&[1.0, 0.0, 1.0], 2, Complex::new(0.0, 1.0), 1e-12)
            .unwrap()
    );
    assert!(
        !utils::has_small_complex_residual(&[1.0, 0.0, 1.0], 2, Complex::new(1.0, 0.0), 1e-12)
            .unwrap()
    );
    assert!(utils::is_finite(Complex::new(1.0, 0.0)));
    assert!(!utils::is_finite(Complex::new(f64::NAN, 0.0)));
}

#[test]
fn laguerre_iteration_reports_the_native_iteration_index() {
    let c = [-2.0, 0.0, 1.0];
    let exact: IterationResult =
        utils::laguerre_iteration(&c, 2, Complex::new(0.0, 0.0), 1e-12, 0).unwrap();
    assert_eq!(
        (exact.status, exact.nb_iterations, exact.root_error),
        (Status::MaxIterations, 0, None)
    );
    let found: IterationResult =
        utils::laguerre_iteration(&c, 2, Complex::new(1.0, 0.0), 1e-12, 100).unwrap();
    assert_eq!(found.status, Status::Ok);
    assert!((found.root.real() - 2.0_f64.sqrt()).abs() < 1e-15);
    assert!(found.nb_iterations > 0 && found.root_error.is_some());
    // A NaN tolerance never accepts a candidate, so the budget runs out.
    let nan: IterationResult =
        utils::laguerre_iteration(&c, 2, Complex::new(1.0, 0.0), f64::NAN, 5).unwrap();
    assert_eq!((nan.status, nan.nb_iterations), (Status::MaxIterations, 5));
    let root: IterationResult = utils::find_root(&[-6.0, 11.0, -6.0, 1.0], 3, 1e-12, 100).unwrap();
    assert_eq!(root.status, Status::Ok);
    assert_eq!(
        utils::find_root(&c, 2, 1e-12, 0).unwrap().status,
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
    assert!(!utils::deflate_real(&mut c, &mut degree, 5.0, 1e-12).unwrap());
    assert_eq!((degree, c), (3, [-6.0, 11.0, -6.0, 1.0]));
    assert!(utils::deflate_real(&mut c, &mut degree, 1.0, 1e-12).unwrap());
    assert_eq!((degree, &c[..3]), (2, &[6.0, -5.0, 1.0][..]));
    // (x^2 + 1)(x - 2) = x^3 - 2x^2 + x - 2.
    let mut cubic = [-2.0, 1.0, -2.0, 1.0];
    let mut degree: i32 = 3;
    assert!(
        utils::deflate_complex(&mut cubic, &mut degree, Complex::new(0.0, 1.0), 1e-12).unwrap()
    );
    assert_eq!((degree, &cubic[..2]), (1, &[-2.0, 1.0][..]));
}

#[test]
fn multiplicity_checks_refine_the_root() {
    // (x - 1)^2 (x + 3).
    let c = [3.0, -5.0, 1.0, 1.0];
    let mut root: f64 = 1.0 + 1e-9;
    assert!(utils::has_multiplicity(&c, 3, &mut root, 2).unwrap());
    assert!((root - 1.0).abs() < 1e-12);
    let mut simple: f64 = -3.0;
    assert!(!utils::has_multiplicity(&c, 3, &mut simple, 2).unwrap());
    // (x^2 + 1)^2 = x^4 + 2x^2 + 1.
    let quartic = [1.0, 0.0, 2.0, 0.0, 1.0];
    let mut z: Complex = Complex::new(1e-9, 1.0);
    assert!(utils::has_complex_multiplicity(&quartic, 4, &mut z, 2).unwrap());
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
    utils::consolidate_real_roots(&mut result, 1e-12, &c, 3, 0).unwrap();
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
    utils::consolidate_complex_roots(&mut pairs, 1e-12, &quartic, 4, 0).unwrap();
    assert_eq!(pairs.nb_complex_roots, 2);
    assert_eq!(pairs.complex_multiplicities[..2], [2, 2]);
    assert_eq!(pairs.complex_roots[1], pairs.complex_roots[0].conj());
}

#[test]
fn laguerre_helpers_refuse_short_or_invalid_polynomials() {
    let failure = utils::evaluate_complex(&[1.0, 2.0], 2, Complex::new(0.0, 0.0))
        .err()
        .unwrap();
    assert_eq!(failure.exception_type(), "Standard_OutOfRange");
    assert!(
        failure.message().contains("degree 2 with 2 coefficients"),
        "{}",
        failure.message()
    );
    assert!(utils::has_small_complex_residual(&[1.0], -1, Complex::new(0.0, 0.0), 1e-12).is_err());
    // No iteration reads the coefficients, as natively.
    let idle = utils::laguerre_iteration(&[1.0], 5, Complex::new(0.0, 0.0), 1e-12, 0).unwrap();
    assert_eq!(idle.status, Status::MaxIterations);
    assert!(utils::laguerre_iteration(&[1.0], 5, Complex::new(0.0, 0.0), 1e-12, 1).is_err());
    assert!(utils::find_root(&[1.0], 5, 1e-12, 3).is_err());
    let (mut short, mut degree) = ([1.0, 2.0], 0);
    assert!(utils::deflate_real(&mut short, &mut degree, 1.0, 1e-12).is_err());
    degree = 1;
    assert!(
        utils::deflate_complex(&mut short, &mut degree, Complex::new(0.0, 1.0), 1e-12).is_err()
    );
    // A non-finite factor is rejected before the degree is read, as natively.
    assert!(
        !utils::deflate_complex(&mut short, &mut degree, Complex::new(f64::NAN, 1.0), 1e-12)
            .unwrap()
    );
    let mut root: f64 = 1.0;
    let failure = utils::has_multiplicity(&[1.0, 1.0], 1, &mut root, 3)
        .err()
        .unwrap();
    assert!(
        failure.message().contains("multiplicity 3 of a degree 1"),
        "{}",
        failure.message()
    );
    let mut z = Complex::new(0.0, 1.0);
    assert!(utils::has_complex_multiplicity(&[1.0; 22], 21, &mut z, 1).is_err());
}

#[test]
fn consolidation_refuses_overfull_root_lists() {
    let c = [3.0, -5.0, 1.0, 1.0];
    let mut reals: PolyResult = PolyResult {
        nb_roots: 21,
        ..Default::default()
    };
    let failure = utils::consolidate_real_roots(&mut reals, 1e-12, &c, 3, 0)
        .err()
        .unwrap();
    assert!(
        failure
            .message()
            .contains("21 real roots in a 20-slot buffer"),
        "{}",
        failure.message()
    );
    let mut complexes: PolyResult = PolyResult {
        nb_complex_roots: 11,
        ..Default::default()
    };
    for (index, slot) in complexes.complex_roots[..11].iter_mut().enumerate() {
        *slot = Complex::new(index as f64, 1.0);
    }
    let failure = utils::consolidate_complex_roots(&mut complexes, 1e-12, &c, 3, 0)
        .err()
        .unwrap();
    assert!(
        failure
            .message()
            .contains("upper-half-plane roots 11 of a 10-slot buffer"),
        "{}",
        failure.message()
    );
}
