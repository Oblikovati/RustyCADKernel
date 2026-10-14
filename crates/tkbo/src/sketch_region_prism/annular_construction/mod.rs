// SPDX-License-Identifier: GPL-2.0-only
//! ADR0050 actual general construction baseline; no annular validity certificate.
mod charts;
mod counts;
mod shared_capture;
use super::{region_certificate::two_loop::fixture::TwoLoopFixture, tests::*, *};

fn annular_receipt() -> PrismReceipt {
    retained_annular_sweep(&TwoLoopFixture::nested())
}
/// Keep the same bridge available for source-domain rebind; `retained_annular_sweep(&fixture)`.
pub(in crate::sketch_region_prism) fn retained_annular_sweep(
    fixture: &TwoLoopFixture,
) -> PrismReceipt {
    let batch = super::region_certificate::borrowed_batch(&fixture.bridge)
        .expect("expected actual completed candidate capture");
    let view = super::region_certificate::temporary_view(batch);
    retain_selected_prism(view, request([0.25, 0.25]), &mut progress(None, None))
}

#[test]
fn construction_borrows_same_bridge_and_preserves_original_source_owners() {
    let fixture = TwoLoopFixture::nested();
    let receipt = retained_annular_sweep(&fixture);
    let batch = super::region_certificate::borrowed_batch(&fixture.bridge).unwrap();
    let selected = receipt.input.selected.as_ref().unwrap();
    assert_eq!(
        batch
            .candidates
            .iter()
            .filter(|c| c.face.is_equal(&selected.face))
            .count(),
        1
    );
    assert_eq!(fixture.sources().map(|s| s.identity.0), [101, 202]);
    assert_eq!(selected.wires.len(), 2);
    assert_inspection(&receipt, receipt.diagnostic.as_ref().unwrap());
}

#[test]
fn actual_annular_sweep_preserves_independent_incidence_and_all_chart_trims() {
    let receipt = annular_receipt();
    assert!(matches!(receipt.completion, PrismCompletion::Incomplete));
    assert!(receipt.limits.achieved_error.is_none());
    let diagnostic = receipt
        .diagnostic
        .as_ref()
        .expect("actual diagnostic annular body");
    counts::assert_body(diagnostic);
    assert!(diagnostic.transport_gaps.is_empty());
    assert_inspection(&receipt, diagnostic);
}
fn assert_inspection(receipt: &PrismReceipt, diagnostic: &PrismDiagnostic) {
    let charts = diagnostic
        .surface_charts
        .as_ref()
        .expect("actual chart inspection");
    assert!(charts.scanned_all_faces);
    assert!(charts.uncovered_faces.is_empty());
    assert_eq!(charts.face_count, 4);
    assert_eq!(charts.entries.len(), 4);
    charts::assert_charts(receipt, charts);
}
