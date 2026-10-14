// SPDX-License-Identifier: GPL-2.0-only
use super::mutation_tests::{edit_target, live_face};
use super::*;
use crate::sketch_region_prism::region_certificate::current_binding::*;
use rustycad_tkbrep::topods::Shape;

fn full_options() -> CompletePrismMeasureOptions {
    CompletePrismMeasureOptions {
        subset: options(),
        side_area: PolynomialSideOptions::default(),
        total_area: EnclosureSumOptions::default(),
    }
}
fn fast_options() -> CompletePrismMeasureOptions {
    // Mutation/replay fixtures use an explicit coarse budget; oracle tests retain defaults.
    let mut policy = full_options();
    policy.side_area.absolute_error_limit = 0.01;
    policy.total_area.absolute_error_limit = 0.02;
    policy
}
fn full_attempt(
    bridge: &PairedContactTransaction,
    shapes: &[Shape],
    receipt: &PrismReceipt,
    policy: CompletePrismMeasureOptions,
    cancelled: impl FnMut() -> bool,
) -> Result<CompletePrismMeasureInspection, CompletePrismMeasureFailure> {
    inspect_complete_prism_measurements(
        bridge,
        &binding_sources(shapes),
        receipt,
        policy,
        cancelled,
    )
}
fn full_report(
    bridge: &PairedContactTransaction,
    shapes: &[Shape],
    receipt: &PrismReceipt,
    policy: CompletePrismMeasureOptions,
) -> CompletePrismMeasureInspection {
    full_attempt(bridge, shapes, receipt, policy, || false)
        .map_err(full_failure_name)
        .unwrap()
}
fn full_failure_name(failure: CompletePrismMeasureFailure) -> String {
    match failure {
        CompletePrismMeasureFailure::Binding(cause) => failure_name(&(*cause).into()),
        CompletePrismMeasureFailure::Subset(cause) => failure_name(&cause),
        CompletePrismMeasureFailure::Side { cause, .. } => format!("side {cause:?}"),
        CompletePrismMeasureFailure::Total(cause) => format!("total {cause:?}"),
    }
}
pub(in crate::sketch_region_prism) fn oracle_interval(quantity: &str) -> [f64; 2] {
    let fixture = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../tkmath/tests/fixtures/prism_complete_measures/closed_cubic.tsv"
    ));
    let row = fixture
        .lines()
        .find(|line| line.starts_with(quantity))
        .unwrap();
    let bits: Vec<u64> = row
        .split('\t')
        .skip(1)
        .map(|value| value.parse().unwrap())
        .collect();
    [f64::from_bits(bits[0]), f64::from_bits(bits[1])]
}
pub(in crate::sketch_region_prism) fn assert_contains_interval(
    observed: [f64; 2],
    expected: [f64; 2],
) {
    assert!(
        observed[0] <= expected[0],
        "{observed:?} must contain {expected:?}"
    );
    assert!(
        expected[1] <= observed[1],
        "{observed:?} must contain {expected:?}"
    );
}
#[test]
fn full_measures_contain_independent_side_and_total_oracles_without_body_promotion() {
    let (bridge, shapes) = fixture();
    let receipt = run(&bridge, &shapes);
    let report = full_report(&bridge, &shapes, &receipt, full_options());
    assert_contains_interval(report.measures.side.enclosure(), oracle_interval("side"));
    assert_contains_interval(report.measures.total.enclosure(), oracle_interval("total"));
    assert_eq!(report.binding.region.selected_winding, 1);
    assert_eq!(report.binding.charts.len(), 3);
    assert!(matches!(receipt.completion, PrismCompletion::Incomplete));
    assert!(receipt.limits.achieved_error.is_none());
    assert_complete_components(&report.measures);
}
fn assert_complete_components(measures: &CompletePrismMeasures) {
    let subset = &measures.subset;
    assert_eq!(
        measures.total.components(),
        &[
            subset.first_cap.enclosure(),
            subset.last_cap.enclosure(),
            measures.side.enclosure()
        ]
    );
    assert_eq!(measures.requested, full_options());
    assert_eq!(measures.side.requested_options(), full_options().side_area);
    assert_eq!(
        measures.total.requested_options(),
        full_options().total_area
    );
}
#[test]
fn complete_inspection_and_all_owned_components_are_send_sync() {
    fn check<T: Send + Sync>() {
        // The entire returned snapshot must be shareable without mutable shape aliases.
        assert!(std::mem::size_of::<T>() < usize::MAX);
    }
    check::<CompletePrismMeasureInspection>();
    check::<CompletePrismMeasureOptions>();
    check::<CompletePrismMeasures>();
}

#[path = "complete/domain.rs"]
mod domain_tests;
#[path = "complete/mutation.rs"]
mod mutation_tests;
#[path = "complete/owned.rs"]
mod owned_tests;
#[path = "complete/policy.rs"]
mod policy_tests;
#[path = "complete/replay.rs"]
mod replay_tests;
