// SPDX-License-Identifier: GPL-2.0-only
use super::super::tests::{fixture, run};
use super::*;
use crate::sketch_region_partition_bindings::binding_sources;

fn options() -> PrismMeasureOptions {
    PrismMeasureOptions {
        cap_area: Default::default(),
        signed_volume: Default::default(),
    }
}
fn measured(
    bridge: &PairedContactTransaction,
    shapes: &[rustycad_tkbrep::topods::Shape],
    receipt: &PrismReceipt,
) -> PrismMeasureInspection {
    match inspect_prism_measurements(bridge, &binding_sources(shapes), receipt, options(), || {
        false
    }) {
        Ok(report) => report,
        Err(failure) => panic!("measurement failed: {}", failure_name(&failure)),
    }
}
fn failure_name(failure: &PrismMeasureFailure) -> String {
    match failure {
        PrismMeasureFailure::Region(cause) => format!("region {cause:?}"),
        PrismMeasureFailure::Chart(cause) => format!("chart {cause:?}"),
        PrismMeasureFailure::Cap { role, cause } => format!("cap {role:?}: {cause}"),
        PrismMeasureFailure::Sweep(cause) => format!("sweep {cause}"),
        PrismMeasureFailure::Classification(_) => "classification".into(),
        PrismMeasureFailure::Body(_) => "body".into(),
        PrismMeasureFailure::Cancelled => "cancelled".into(),
    }
}
#[test]
fn current_jordan_cap_and_sweep_subset_completes_without_body_promotion() {
    let (bridge, shapes) = fixture();
    let receipt = run(&bridge, &shapes);
    let report = measured(&bridge, &shapes, &receipt);
    assert_eq!(report.region.selected_winding, 1);
    assert_eq!(report.charts.len(), 3);
    assert_encloses(report.measures.first_cap.enclosure(), 2.4);
    assert_encloses(report.measures.last_cap.enclosure(), 2.4);
    assert_encloses(report.measures.signed_volume.enclosure(), 4.8);
    assert_eq!(
        report.pending,
        [PendingMeasure::SideArea, PendingMeasure::TotalArea]
    );
    assert!(matches!(receipt.completion, PrismCompletion::Incomplete));
    assert!(receipt.limits.achieved_error.is_none());
}
fn assert_encloses(enclosure: [f64; 2], expected: f64) {
    // Smoke checks supplement the independently pinned rational endpoint bits.
    assert!(enclosure[0] <= expected && expected <= enclosure[1]);
}
#[test]
fn scalar_inspection_owns_send_sync_captures() {
    fn check<T: Send + Sync>() {}
    check::<PrismMeasureOptions>();
    check::<PrismMeasureInspection>();
    check::<MeasuredPrismChart>();
    check::<PrismMeasureSubset>();
    check::<PendingMeasure>();
}

#[path = "tests/history.rs"]
mod history_tests;
#[path = "tests/mutation.rs"]
mod mutation_tests;

fn measurement_attempt(
    bridge: &PairedContactTransaction,
    shapes: &[rustycad_tkbrep::topods::Shape],
    receipt: &PrismReceipt,
    limits: PrismMeasureOptions,
    cancelled: impl FnMut() -> bool,
) -> Result<PrismMeasureInspection, PrismMeasureFailure> {
    inspect_prism_measurements(bridge, &binding_sources(shapes), receipt, limits, cancelled)
}
#[path = "tests/domain.rs"]
mod domain_tests;
#[path = "tests/replay.rs"]
mod replay_tests;

fn assert_hypothesis(error: PrismMeasureFailure, expected: &str) {
    let PrismMeasureFailure::Region(RegionCertificateCause::Hypothesis { name, .. }) = error else {
        panic!("{}; expected {expected}", failure_name(&error));
    };
    assert_eq!(name, expected);
}
#[path = "tests/complete.rs"]
mod complete_tests;
