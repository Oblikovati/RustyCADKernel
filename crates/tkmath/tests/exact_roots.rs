// SPDX-License-Identifier: GPL-2.0-only
//! Independent root identities and endpoint certificates for kernel issue #10.
use rustycad_tkmath::predicates::roots::{
    DyadicBound, RootBracket, RootIsolation, RootIsolationError, RootIsolationOptions,
    isolate_polynomial_roots,
};
use std::cmp::Ordering;
mod bernstein;
mod bezier_line;
mod bezier_regularity;
mod limits;
mod oracle;
mod signs;
fn options(width: f64) -> RootIsolationOptions {
    RootIsolationOptions::new(width)
}
fn isolate(coefficients: &[f64], interval: [f64; 2]) -> RootIsolation {
    isolate_polynomial_roots(coefficients, interval, options(0.0001), || false).unwrap()
}
fn verify_order_and_width(result: &RootIsolation, width: f64) {
    for pair in result.roots().windows(2) {
        assert!(pair[0].bounds().1 < pair[1].bounds().0);
    }
    for root in result.roots() {
        assert!(root.bounds().0 <= root.bounds().1);
        assert!(root.width().compare_f64(width).unwrap() != Ordering::Greater);
        assert!(root.multiplicity() > 0);
    }
}
#[test]
fn repeated_endpoint_roots_are_counted_once_with_exact_multiplicity() {
    let coefficients = [0., 0., 1., 0., -2., 0., 1.];
    let result = isolate(&coefficients, [-1., 1.]);
    assert_eq!(result.roots().len(), 3);
    for (root, exact) in result.roots().iter().zip([-1., 0., 1.]) {
        assert_eq!(root.bounds().0, root.bounds().1);
        assert_eq!(root.bounds().0.compare_f64(exact).unwrap(), Ordering::Equal);
        assert_eq!(root.multiplicity(), 2);
    }
    assert_eq!(result.bisections(), 1);
    verify_order_and_width(&result, 0.0001);
}
#[test]
fn closed_intervals_handle_point_queries_constants_and_zero_polynomials() {
    assert_eq!(isolate(&[0., 0., 1.], [0., 0.]).roots().len(), 1);
    assert!(isolate(&[0., 0., 1.], [1., 1.]).roots().is_empty());
    assert!(isolate(&[-1., 0., 0.], [-2., 2.]).roots().is_empty());
    for coefficients in [vec![], vec![0., -0., 0.]] {
        assert!(matches!(
            isolate_polynomial_roots(&coefficients, [-1., 1.], options(1.), || false),
            Err(RootIsolationError::ZeroPolynomial { .. })
        ));
    }
}
#[test]
fn signs_and_roots_are_invariant_under_every_finite_binary_exponent() {
    let reference = isolate(&[0., -1., 1.], [-2., 2.]);
    for exponent in 0_u64..2047 {
        let scale = f64::from_bits((exponent << 52) | 1);
        for sign in [-1., 1.] {
            assert_eq!(
                isolate(&[0., -sign * scale, sign * scale], [-2., 2.]),
                reference
            );
        }
    }
}
#[test]
fn isolation_separates_a_positive_root_smaller_than_the_smallest_binary64() {
    let tiny = f64::from_bits(1);
    let result =
        isolate_polynomial_roots(&[0., -tiny, f64::MAX], [0., tiny], options(tiny), || false)
            .unwrap();
    assert_eq!(result.roots().len(), 2);
    let positive = &result.roots()[1];
    assert_eq!(
        positive.bounds().0.compare_f64(0.).unwrap(),
        Ordering::Greater
    );
    assert_eq!(
        positive.bounds().1.compare_f64(tiny).unwrap(),
        Ordering::Less
    );
    verify_order_and_width(&result, tiny);
}
#[test]
fn maximal_interval_does_not_overflow_midpoints_or_widths() {
    let result =
        isolate_polynomial_roots(&[-1., 0., 1.], [-f64::MAX, f64::MAX], options(0.1), || {
            false
        })
        .unwrap();
    assert_eq!(result.roots().len(), 2);
    for (root, exact) in result.roots().iter().zip([-1., 1.]) {
        assert!(root.bounds().0.compare_f64(exact).unwrap() != Ordering::Greater);
        assert!(root.bounds().1.compare_f64(exact).unwrap() != Ordering::Less);
    }
    verify_order_and_width(&result, 0.1);
}
#[test]
fn dyadic_values_have_canonical_exact_order_and_text() {
    let half = DyadicBound::from_f64(0.5).unwrap();
    assert_eq!(half.to_string(), "1 * 2^-1");
    assert_eq!(half.compare_f64(0.25).unwrap(), Ordering::Greater);
    assert_eq!(half.partial_cmp(&half), Some(Ordering::Equal));
    assert_eq!(
        DyadicBound::from_f64(0.).unwrap(),
        DyadicBound::from_f64(-0.).unwrap()
    );
    assert_eq!(DyadicBound::from_f64(-2.).unwrap().to_string(), "-1 * 2^1");
    assert!(half.compare_f64(f64::INFINITY).is_err());
}

#[test]
fn repeated_degree_32_and_sparse_degree_64_use_the_same_pipeline() {
    let mut coefficients = vec![1.];
    for _ in 0..16 {
        coefficients = multiplied(&coefficients, &[-2., 0., 1.]);
    }
    let repeated = isolate(&coefficients, [-2., 2.]);
    assert_eq!(repeated.roots().len(), 2);
    assert!(
        repeated
            .roots()
            .iter()
            .all(|root| root.multiplicity() == 16)
    );
    let mut sparse = vec![0.; 65];
    sparse[0] = -1.;
    sparse[64] = 1.;
    let mut policy = options(0.01);
    policy.max_degree = 64;
    let result = isolate_polynomial_roots(&sparse, [-2., 2.], policy, || false).unwrap();
    assert_eq!(result.roots().len(), 2);
    assert!(result.roots().iter().all(|root| root.multiplicity() == 1));
    verify_order_and_width(&result, 0.01);
}
fn multiplied(left: &[f64], right: &[f64]) -> Vec<f64> {
    let mut product = vec![0.; left.len() + right.len() - 1];
    for (i, a) in left.iter().enumerate() {
        for (j, b) in right.iter().enumerate() {
            product[i + j] += a * b;
        }
    }
    product
}
