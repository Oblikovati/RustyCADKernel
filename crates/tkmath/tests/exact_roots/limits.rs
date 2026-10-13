// SPDX-License-Identifier: GPL-2.0-only
use super::*;
#[test]
fn malformed_requests_name_the_rejected_value_and_expected_shape() {
    let error = isolate_polynomial_roots(&[1., f64::INFINITY], [0., 1.], options(1.), || false)
        .unwrap_err();
    assert_eq!(
        error,
        RootIsolationError::Coefficient {
            index: 1,
            value: f64::INFINITY
        }
    );
    assert!(
        error
            .to_string()
            .contains("coefficient 1 is inf; expected finite")
    );
    for interval in [[1., 0.], [f64::NEG_INFINITY, 1.], [0., f64::NAN]] {
        assert!(matches!(
            isolate_polynomial_roots(&[1.], interval, options(1.), || false),
            Err(RootIsolationError::Interval(_))
        ));
    }
    for width in [0., -1., f64::NAN, f64::INFINITY] {
        assert!(matches!(
            isolate_polynomial_roots(&[1.], [0., 1.], options(width), || false),
            Err(RootIsolationError::Width(_))
        ));
    }
}
#[test]
fn degree_admission_ignores_zero_leading_coefficients() {
    let mut policy = options(1.);
    policy.max_degree = 1;
    let error = isolate_polynomial_roots(&[1., 0., 1.], [0., 1.], policy, || false).unwrap_err();
    assert_eq!(
        error,
        RootIsolationError::Degree {
            actual: 2,
            limit: 1
        }
    );
    assert!(isolate_polynomial_roots(&[1., 1., 0.], [0., 1.], policy, || false).is_ok());
}
#[test]
fn exhausted_budget_is_incomplete_instead_of_empty_or_partially_successful() {
    let mut policy = options(0.00001);
    policy.max_bisections = 0;
    let error =
        isolate_polynomial_roots(&[0., -2., 0., 1.], [0., 2.], policy, || false).unwrap_err();
    assert_eq!(error, RootIsolationError::SubdivisionLimit { limit: 0 });
    assert!(error.to_string().contains("limit 0 exhausted"));
    assert!(
        isolate_polynomial_roots(&[1.], [-2., 2.], policy, || false)
            .unwrap()
            .roots()
            .is_empty()
    );
}
#[test]
fn cancellation_at_every_observed_checkpoint_discards_the_entire_request() {
    let coefficients = [-4., 12., -8., -8., 11., -1., -3., 1.];
    let mut calls = 0;
    let successful = isolate_polynomial_roots(&coefficients, [-2., 2.], options(0.1), || {
        calls += 1;
        false
    })
    .unwrap();
    assert_eq!(successful.roots().len(), 3);
    for stop in 1..=calls {
        let mut current = 0;
        let result = isolate_polynomial_roots(&coefficients, [-2., 2.], options(0.1), || {
            current += 1;
            current == stop
        });
        assert_eq!(
            result,
            Err(RootIsolationError::Cancelled),
            "checkpoint {stop}"
        );
    }
    assert_eq!(
        isolate_polynomial_roots(&coefficients, [-2., 2.], options(0.1), || false).unwrap(),
        successful
    );
}

#[test]
fn policy_defaults_and_all_declines_have_explicit_diagnostics() {
    let mut policy = RootIsolationOptions::new(1.);
    assert_eq!(
        (policy.version, policy.max_degree, policy.max_bisections),
        (1, 32, 4096)
    );
    policy.version = 2;
    let error = isolate_polynomial_roots(&[1.], [0., 1.], policy, || false).unwrap_err();
    assert_eq!(error, RootIsolationError::OptionsVersion(2));
    let failures = [
        error,
        RootIsolationError::Interval([2., 1.]),
        RootIsolationError::Width(-2.),
        RootIsolationError::ZeroPolynomial { coefficients: 3 },
        RootIsolationError::Degree {
            actual: 4,
            limit: 3,
        },
        RootIsolationError::Cancelled,
        RootIsolationError::Variation { lower: 1, upper: 2 },
    ];
    for failure in failures {
        assert!(failure.to_string().contains("; expected "));
    }
}

#[test]
fn requests_preserve_inputs_and_results_outlive_source_mutation() {
    let mut coefficients = vec![-2., -0., 1.];
    let bits: Vec<u64> = coefficients
        .iter()
        .map(|value| f64::to_bits(*value))
        .collect();
    let result = isolate(&coefficients, [-2., 2.]);
    let original = result.clone();
    let mut limited = options(0.001);
    limited.max_bisections = 0;
    assert!(isolate_polynomial_roots(&coefficients, [-2., 2.], limited, || false).is_err());
    assert_eq!(
        isolate_polynomial_roots(&coefficients, [-2., 2.], options(1.), || true),
        Err(RootIsolationError::Cancelled)
    );
    assert_eq!(
        coefficients
            .iter()
            .map(|value| f64::to_bits(*value))
            .collect::<Vec<_>>(),
        bits
    );
    coefficients[0] = 0.;
    assert_eq!(isolate(&coefficients, [-2., 2.]).roots().len(), 1);
    coefficients.clear();
    assert_eq!(result, original);
}
