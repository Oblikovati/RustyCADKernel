// SPDX-License-Identifier: GPL-2.0-only
use super::*;
#[test]
fn bernstein_degree_admission_uses_the_exact_reduced_degree() {
    let mut policy = options(0.0001);
    policy.max_degree = 1;
    let linear: Vec<f64> = (0..=64).map(|index| f64::from(index) - 32.).collect();
    let result = isolate_bernstein_roots(&linear, [0., 1.], policy, || false).unwrap();
    assert_eq!(result, isolate(&[-32., 64.], [0., 1.]));
    assert_eq!(
        isolate_bernstein_roots(&[0., 0., 1.], [0., 1.], policy, || false),
        Err(RootIsolationError::Degree {
            actual: 2,
            limit: 1
        })
    );
}
#[test]
fn bernstein_constant_has_zero_degree_despite_its_control_count() {
    let mut policy = options(0.0001);
    policy.max_degree = 0;
    assert!(
        isolate_bernstein_roots(&[7.; 65], [-4., 4.], policy, || false)
            .unwrap()
            .roots()
            .is_empty()
    );
}
#[test]
fn bernstein_malformed_inputs_and_zero_polynomials_are_typed_declines() {
    for controls in [vec![], vec![0.; 5]] {
        assert_eq!(
            isolate_bernstein_roots(&controls, [0., 1.], options(0.1), || false),
            Err(RootIsolationError::ZeroPolynomial {
                coefficients: controls.len()
            })
        );
    }
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let failure = isolate_bernstein_roots(&[0., value, 1.], [0., 1.], options(0.1), || false)
            .unwrap_err();
        assert!(matches!(
            failure,
            RootIsolationError::Coefficient { index: 1, .. }
        ));
        assert!(failure.to_string().contains("expected finite binary64"));
    }
}
#[test]
fn bernstein_cancellation_discards_conversion_and_isolation_state() {
    let controls = [0., 2., -2., 0.];
    let mut checkpoints = 0;
    let expected = isolate_bernstein_roots(&controls, [-2., 2.], options(0.001), || {
        checkpoints += 1;
        false
    })
    .unwrap();
    check_bernstein_cancellation(&controls, checkpoints);
    assert_eq!(
        isolate_bernstein_roots(&controls, [-2., 2.], options(0.001), || false).unwrap(),
        expected
    );
}
#[test]
fn bernstein_inputs_and_completed_results_are_isolated_from_later_edits() {
    let mut controls = vec![0., 1., -1.];
    let original: Vec<u64> = controls.iter().map(|value: &f64| value.to_bits()).collect();
    let completed = bernstein(&controls, [0., 1.]);
    let snapshot = completed.clone();
    check_bernstein_failures(&controls);
    assert_eq!(
        controls
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<u64>>(),
        original
    );
    controls.fill(1.);
    assert!(bernstein(&controls, [0., 1.]).roots().is_empty());
    controls.clear();
    assert_eq!(completed, snapshot);
}

fn check_bernstein_cancellation(controls: &[f64], checkpoints: usize) {
    for stop in 1..=checkpoints {
        let mut calls = 0;
        let result = isolate_bernstein_roots(controls, [-2., 2.], options(0.001), || {
            calls += 1;
            calls == stop
        });
        assert_eq!(
            result,
            Err(RootIsolationError::Cancelled),
            "checkpoint {stop}"
        );
    }
}

fn check_bernstein_failures(controls: &[f64]) {
    let mut policy = options(0.0001);
    policy.max_bisections = 0;
    assert_eq!(
        isolate_bernstein_roots(controls, [0., 1.], policy, || false),
        Err(RootIsolationError::SubdivisionLimit { limit: 0 })
    );
    assert_eq!(
        isolate_bernstein_roots(controls, [0., 1.], policy, || true),
        Err(RootIsolationError::Cancelled)
    );
}
