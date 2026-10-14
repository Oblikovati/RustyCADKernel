// SPDX-License-Identifier: GPL-2.0-only
use super::*;
fn options(limit: f64) -> EnclosureSumOptions {
    EnclosureSumOptions {
        absolute_error_limit: limit,
        ..Default::default()
    }
}
#[test]
fn complete_empty_and_exact_sums_have_zero_error() {
    for (components, expected) in [(vec![], 0.), (vec![[1., 1.], [2., 2.]], 3.)] {
        let report = sum_nonnegative_enclosures(&components, options(0.), || false).unwrap();
        assert_eq!(report.enclosure(), [expected; 2]);
        assert_eq!(report.representative(), expected);
        assert_eq!(report.absolute_error_bound(), 0.);
        assert_eq!(report.components(), components);
        assert_eq!(report.requested_options(), report.effective_options());
        assert_eq!(report.operation(), EnclosureSumOperation::NonnegativeBounds);
    }
}
#[test]
fn endpoint_rounding_is_included_even_when_children_are_exact() {
    let components = [[1., 1.], [f64::EPSILON / 2.; 2]];
    let report = sum_nonnegative_enclosures(&components, options(f64::EPSILON), || false).unwrap();
    assert_eq!(report.enclosure(), [1., 1. + f64::EPSILON]);
    assert_eq!(report.absolute_error_bound(), f64::EPSILON);
    let error = sum_nonnegative_enclosures(&components, options(0.), || false).unwrap_err();
    assert_eq!(
        error.cause,
        EnclosureSumCause::ErrorLimit {
            requested: 0.,
            achieved: f64::EPSILON
        }
    );
    assert_eq!(error.requested_options, options(0.));
}
#[test]
fn three_component_width_includes_uncertainty_without_budget_allocation() {
    let report =
        sum_nonnegative_enclosures(&[[1., 2.], [4., 8.], [16., 32.]], options(21.), || false)
            .unwrap();
    assert_eq!(report.enclosure(), [21., 42.]);
    assert_eq!(report.absolute_error_bound(), 21.);
    assert_eq!(report.effective_options(), options(21.));
}
#[test]
fn subnormal_endpoint_addition_is_exact_and_large_overflow_is_incomplete() {
    let tiny = f64::from_bits(1);
    let report = sum_nonnegative_enclosures(&[[tiny; 2]; 3], options(0.), || false).unwrap();
    assert_eq!(report.enclosure(), [f64::from_bits(3); 2]);
    let error = sum_nonnegative_enclosures(&[[f64::MAX; 2]; 2], options(0.), || false).unwrap_err();
    assert!(matches!(
        error.cause,
        EnclosureSumCause::Unrepresentable {
            quantity: "sum lower",
            ..
        }
    ));
}
#[test]
fn malformed_components_keep_index_and_supplied_values() {
    for bad in [[-1., 0.], [2., 1.], [0., f64::INFINITY], [0., f64::NAN]] {
        let error = sum_nonnegative_enclosures(&[[0.; 2], bad], options(1.), || false).unwrap_err();
        let EnclosureSumCause::Component { index, supplied } = error.cause else {
            panic!("expected component refusal")
        };
        assert_eq!(index, 1);
        assert_eq!(supplied.map(f64::to_bits), bad.map(f64::to_bits));
    }
}
#[test]
fn original_policy_and_count_limits_are_never_clamped() {
    for policy in invalid_policies() {
        let error = sum_nonnegative_enclosures(&[], policy, || false).unwrap_err();
        assert!(matches!(error.cause, EnclosureSumCause::Policy(_)));
    }
}
#[test]
fn zero_component_limit_admits_only_completed_empty_sum() {
    let policy = EnclosureSumOptions {
        max_components: 0,
        ..options(0.)
    };
    assert!(sum_nonnegative_enclosures(&[], policy, || false).is_ok());
    assert_eq!(
        sum_nonnegative_enclosures(&[[0.; 2]], policy, || false)
            .unwrap_err()
            .cause,
        EnclosureSumCause::ComponentCount {
            actual: 1,
            maximum: 0
        }
    );
}
#[test]
fn every_poll_cancels_without_retained_state_and_retry_preserves_input() {
    let components = [[1., 2.]; 3];
    let mut count = 0;
    let baseline = sum_nonnegative_enclosures(&components, options(3.), || {
        count += 1;
        false
    })
    .unwrap();
    assert_cancelled_components(&components, count);
    assert_eq!(
        sum_nonnegative_enclosures(&components, options(3.), || false).unwrap(),
        baseline
    );
}
#[test]
fn owned_report_does_not_observe_later_source_edits() {
    let mut components = [[-0., 1.]];
    let report = sum_nonnegative_enclosures(&components, options(1.), || false).unwrap();
    components[0] = [9., 10.];
    assert_eq!(
        report.components()[0].map(f64::to_bits),
        [-0., 1.].map(f64::to_bits)
    );
    assert_eq!(report.enclosure(), [0., 1.]);
    assert_eq!(components, [[9., 10.]]);
}
#[test]
fn exact_accumulation_is_permutation_independent() {
    let tiny = f64::from_bits(1);
    for power in -20..20 {
        let large = 2_f64.powi(power);
        let mut components = [[large; 2], [tiny; 2], [large; 2]];
        let first = sum_nonnegative_enclosures(&components, options(1.), || false).unwrap();
        components.reverse();
        let last = sum_nonnegative_enclosures(&components, options(1.), || false).unwrap();
        assert_eq!(first.enclosure(), last.enclosure());
    }
}
#[test]
fn sum_types_are_send_sync_and_independent_workers_repeat() {
    check_sum_type::<CertifiedEnclosureSum>();
    check_sum_type::<EnclosureSumError>();
    check_sum_type::<EnclosureSumCause>();
    check_sum_type::<EnclosureSumOptions>();
    check_sum_type::<EnclosureSumOperation>();
    check_sum_type::<EnclosureSumEvidence>();
    let workers: Vec<_> = (0..8)
        .map(|_| std::thread::spawn(repeated_sum_reports))
        .collect();
    let expected = sum_nonnegative_enclosures(&[[1., 2.]; 3], options(3.), || false).unwrap();
    for worker in workers {
        for actual in worker.join().unwrap() {
            assert_eq!(actual, expected);
        }
    }
}

fn invalid_policies() -> [EnclosureSumOptions; 5] {
    [
        options(-1.),
        options(f64::NAN),
        options(f64::INFINITY),
        EnclosureSumOptions {
            behavior_version: 2,
            ..options(0.)
        },
        EnclosureSumOptions {
            max_components: 65,
            ..options(0.)
        },
    ]
}

fn assert_cancelled_components(components: &[[f64; 2]], count: usize) {
    for cutoff in 1..=count {
        let mut poll = 0;
        let error = sum_nonnegative_enclosures(components, options(3.), || {
            poll += 1;
            poll == cutoff
        })
        .unwrap_err();
        assert_eq!(error.cause, EnclosureSumCause::Cancelled);
        assert_eq!(components, &[[1., 2.]; 3]);
    }
}

fn repeated_sum_reports() -> Vec<CertifiedEnclosureSum> {
    (0..32)
        .map(|_| sum_nonnegative_enclosures(&[[1., 2.]; 3], options(3.), || false).unwrap())
        .collect()
}

#[test]
fn evidence_scope_is_conditional_endpoint_arithmetic() {
    let report = sum_nonnegative_enclosures(&[[1., 1.]], options(0.), || false).unwrap();
    assert_eq!(
        report.evidence(),
        EnclosureSumEvidence::CertifiedConditionalEndpointArithmetic
    );
    assert!(
        sum_nonnegative_enclosures(&[[2., 1.]], options(0.), || false)
            .unwrap_err()
            .to_string()
            .contains("finite ordered nonnegative")
    );
}

fn check_sum_type<T: Send + Sync>() {
    // Compile-time ownership bound covers the complete reachable report.
    assert!(std::mem::size_of::<T>() < usize::MAX);
}
