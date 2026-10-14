// SPDX-License-Identifier: GPL-2.0-only
use crate::predicates::roots::metric_bounds::*;

#[test]
fn original_side_policy_is_refused_before_a_smaller_budget_can_hide_it() {
    for limit in [65537, u32::MAX] {
        let requested = PolynomialSideOptions {
            max_subdivisions: limit,
            ..Default::default()
        };
        let cause = validate_polynomial_side_options(requested).unwrap_err();
        assert!(matches!(cause, PolynomialSideFailure::Policy(actual) if actual == requested));
    }
}

#[test]
fn standalone_side_policy_keeps_zero_and_backend_maximum_admissible() {
    for limit in [0, 65536] {
        let requested = PolynomialSideOptions {
            max_subdivisions: limit,
            ..Default::default()
        };
        assert!(validate_polynomial_side_options(requested).is_ok());
    }
}
