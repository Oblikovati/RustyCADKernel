// SPDX-License-Identifier: GPL-2.0-only
//! Independent complete polynomial charts for the ADR0014 cubic; body gates remain incomplete.
use super::super::{tests::*, *};
use super::*;

#[test]
fn cubic_actual_caps_and_side_have_complete_exact_chart_and_trim_proofs() {
    let receipt = cubic_receipt([1., 1.], &mut progress(None, None));
    assert_cubic_collection(inspection(&receipt));
    assert_body_incomplete(&receipt);
}
pub(super) fn inspection(receipt: &PrismReceipt) -> &PrismSurfaceInspection {
    receipt
        .diagnostic
        .as_ref()
        .unwrap()
        .surface_charts
        .as_ref()
        .unwrap()
}
pub(super) fn assert_cubic_collection(inspection: &PrismSurfaceInspection) {
    assert_eq!(inspection.version, 1);
    assert_eq!(inspection.face_count, 3);
    assert!(inspection.scanned_all_faces);
    assert!(inspection.uncovered_faces.is_empty());
    assert_eq!(inspection.entries.len(), 3);
    for entry in &inspection.entries {
        assert!(
            entry.decision.is_ok(),
            "{:?}: {:?}",
            entry.role,
            entry.decision
        );
        assert!(!entry.parents.is_empty());
    }
}
pub(super) fn assert_body_incomplete(receipt: &PrismReceipt) {
    assert!(matches!(receipt.completion, PrismCompletion::Incomplete));
    assert!(
        receipt
            .gaps
            .iter()
            .any(|gap| matches!(gap, PrismGap::PrismInjectivityAndSurfaceCharts))
    );
    assert!(
        !receipt
            .diagnostic
            .as_ref()
            .unwrap()
            .coverage
            .as_ref()
            .unwrap()
            .complete()
    );
    assert!(receipt.limits.achieved_error.is_none());
}

static_assertions::assert_impl_all!(PrismSurfaceProof: Send, Sync);
static_assertions::assert_impl_all!(CapturedChartSource: Send, Sync);
static_assertions::assert_impl_all!(CapturedPrismSurface: Send, Sync);
static_assertions::assert_impl_all!(CapturedTrimExpectation: Send, Sync);
static_assertions::assert_impl_all!(PrismTrimRole: Send, Sync);
static_assertions::assert_impl_all!(PrismChartStage: Send, Sync);
pub(super) fn entry_by_role(receipt: &PrismReceipt, role: PrismImageRole) -> &PrismSurfaceEntry {
    inspection(receipt)
        .entries
        .iter()
        .find(|entry| entry.role == role)
        .unwrap()
}
pub(super) fn live_face(
    receipt: &PrismReceipt,
    role: PrismImageRole,
) -> rustycad_tkbrep::topods::Face {
    let key = &entry_by_role(receipt, role).output;
    let binding = receipt
        .diagnostic
        .as_ref()
        .unwrap()
        .bindings
        .iter()
        .find(|binding| {
            crate::sketch_region::PartitionOccurrenceIdentity::capture(&binding.context.occurrence)
                == *key
        })
        .unwrap();
    rustycad_tkbrep::topods::face(&binding.context.occurrence).unwrap()
}
pub(super) fn fresh_inspection(receipt: &mut PrismReceipt) {
    retain_surface_charts(receipt, &mut progress(None, None)).unwrap_or_else(|_| {
        panic!("expected diagnostic chart inspection without operation cancellation")
    });
}
