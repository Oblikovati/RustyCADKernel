// SPDX-License-Identifier: GPL-2.0-only
//! ADR0050 actual general construction baseline; no annular validity certificate.
mod charts;
mod counts;
use super::{region_certificate::two_loop::fixture::TwoLoopFixture, tests::*, *};

fn annular_receipt() -> PrismReceipt {
    let mut fixture = TwoLoopFixture::nested();
    let copied = fixture
        .bridge
        .mapped
        .copied
        .take()
        .expect("actual candidate copies");
    let batch = copied
        .completion
        .unwrap_or_else(|_| panic!("expected completed candidate capture"));
    retain_selected_prism(batch, request([0.25, 0.25]), &mut progress(None, None))
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
