// SPDX-License-Identifier: GPL-2.0-only
use super::*;
use crate::sketch_region::PartitionOccurrenceIdentity;
type SideKeys = (PartitionOccurrenceIdentity, PartitionOccurrenceIdentity);

#[test]
fn side_work_exhaustion_keeps_current_chart_keys_and_requested_policy() {
    let (bridge, shapes) = fixture();
    let receipt = run(&bridge, &shapes);
    let mut policy = fast_options();
    policy.side_area.max_subdivisions = 0;
    policy.side_area.absolute_error_limit = 0.;
    let failure = full_attempt(&bridge, &shapes, &receipt, policy, || false)
        .err()
        .unwrap();
    let keys = side_error_keys(failure, policy.side_area);
    assert_current_side_keys(&bridge, &shapes, &receipt, keys);
}

#[test]
fn total_zero_budget_refuses_after_children_complete_without_retry() {
    let (bridge, shapes) = fixture();
    let receipt = run(&bridge, &shapes);
    let mut policy = fast_options();
    policy.total_area.absolute_error_limit = 0.;
    let failure = full_attempt(&bridge, &shapes, &receipt, policy, || false)
        .err()
        .unwrap();
    let CompletePrismMeasureFailure::Total(cause) = failure else {
        panic!("expected total failure")
    };
    assert_eq!(cause.requested_options, policy.total_area);
    assert!(
        matches!(cause.cause, EnclosureSumCause::ErrorLimit { requested: 0., achieved } if achieved > 0.)
    );
    let retry = full_report(&bridge, &shapes, &receipt, fast_options());
    assert_eq!(retry.measures.side.requested_options(), policy.side_area);
}
#[test]
fn cap_and_volume_failure_quantities_remain_original_subset_errors() {
    let (bridge, shapes) = fixture();
    let receipt = run(&bridge, &shapes);
    let mut policy = fast_options();
    policy.subset.signed_volume.absolute_error_limit = 0.;
    let failure = full_attempt(&bridge, &shapes, &receipt, policy, || false)
        .err()
        .unwrap();
    assert!(matches!(
        failure,
        CompletePrismMeasureFailure::Subset(PrismMeasureFailure::Sweep(_))
    ));
}
#[test]
fn cap_failure_quantity_remains_original_subset_error() {
    let (bridge, shapes) = fixture();
    let receipt = run(&bridge, &shapes);
    let mut policy = fast_options();
    policy.subset.cap_area.absolute_error_limit = 0.;
    let failure = full_attempt(&bridge, &shapes, &receipt, policy, || false)
        .err()
        .unwrap();
    assert!(matches!(
        failure,
        CompletePrismMeasureFailure::Subset(PrismMeasureFailure::Cap {
            role: PrismImageRole::First,
            ..
        })
    ));
}

fn side_error_keys(
    failure: CompletePrismMeasureFailure,
    expected: PolynomialSideOptions,
) -> SideKeys {
    let CompletePrismMeasureFailure::Side {
        source,
        output,
        cause,
    } = failure
    else {
        panic!("expected side work failure")
    };
    assert_eq!(cause.requested_options, expected);
    assert!(matches!(
        *cause.cause,
        PolynomialSideFailure::WorkLimit { requested: 0, .. }
    ));
    (source, output)
}
fn assert_current_side_keys(
    bridge: &PairedContactTransaction,
    shapes: &[Shape],
    receipt: &PrismReceipt,
    keys: SideKeys,
) {
    let snapshot =
        capture_current_prism_binding(bridge, &binding_sources(shapes), receipt, &mut || false)
            .ok()
            .unwrap();
    let chart = snapshot
        .charts
        .iter()
        .find(|chart| chart.role == PrismImageRole::Generated)
        .unwrap();
    assert_eq!(keys.0, chart.source);
    assert_eq!(keys.1, chart.output);
}
