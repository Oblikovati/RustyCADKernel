// SPDX-License-Identifier: GPL-2.0-only
//! Four-face admission exercises raw capture only; no source-domain certificate is fabricated.
use super::*;
use crate::sketch_region_prism::region_certificate::{RegionCertificateCause, current_binding::*};
use crate::sketch_region_source_conformance::{same_metric_curve, same_space_representation};

fn four_faces(faces: &[PrismBodyContext]) -> Result<(), PrismBindingFailure> {
    if faces.len() != 4 {
        return Err(RegionCertificateCause::Hypothesis {
            name: "authored annular construction faces",
            actual: faces.len(),
            expected: 4,
        }
        .into());
    }
    Ok(())
}
#[test]
fn shared_capture_reuses_actual_source_role_and_all_trim_operands() {
    let receipt = annular_receipt();
    let charts = capture_actual_charts(&receipt, &mut || false, four_faces)
        .unwrap_or_else(|_| panic!("expected four individually proven actual charts"));
    assert_eq!(charts.len(), 4);
    compare_retained_charts(&receipt, &charts);
}
fn compare_retained_charts(receipt: &PrismReceipt, charts: &[BoundPrismChart]) {
    let diagnostic = receipt.diagnostic.as_ref().unwrap();
    let retained = &diagnostic.surface_charts.as_ref().unwrap().entries;
    for chart in charts {
        let matches: Vec<_> = retained
            .iter()
            .filter(|c| {
                c.source == chart.source && c.output == chart.output && c.role == chart.role
            })
            .collect();
        assert_eq!(matches.len(), 1);
        assert_chart_operands(chart, matches[0]);
    }
}
fn assert_chart_operands(
    chart: &BoundPrismChart,
    retained: &super::super::surface_charts::PrismSurfaceEntry,
) {
    assert_eq!(chart.parents, retained.parents);
    assert_chart_encoding(chart, retained);
    assert_eq!(chart.proof.identities, retained.proof.identities);
    assert_eq!(chart.proof.trim_roles, retained.proof.trim_roles);
    let actual = &retained.captured.as_ref().unwrap().trims;
    assert_eq!(chart.captured.trims.len(), actual.len());
    for trim in &chart.captured.trims {
        let rows: Vec<_> = actual
            .iter()
            .filter(|t| t.output == trim.output && t.source == trim.source && t.role == trim.role)
            .collect();
        assert_eq!(rows.len(), 1);
        assert_eq!(trim.parents, rows[0].parents);
        assert_eq!(trim.interval, rows[0].interval);
        assert!(same_metric_curve(&trim.pcurve, &rows[0].pcurve));
        assert!(same_space_representation(&trim.space, &rows[0].space));
    }
}
#[test]
fn face_admission_precedes_any_chart_callback_and_keeps_typed_refusal() {
    let receipt = annular_receipt();
    let mut polls = 0;
    let result = capture_actual_charts(
        &receipt,
        &mut || {
            polls += 1;
            false
        },
        refuse_faces,
    );
    assert_eq!(polls, 0);
    assert_count_refusal(result, 4, 3);
}
fn refuse_faces(_: &[PrismBodyContext]) -> Result<(), PrismBindingFailure> {
    Err(RegionCertificateCause::Hypothesis {
        name: "authored face admission",
        actual: 4,
        expected: 3,
    }
    .into())
}
fn assert_count_refusal(
    result: Result<Vec<BoundPrismChart>, PrismBindingFailure>,
    actual: usize,
    expected: usize,
) {
    let Err(PrismBindingFailure::Region(RegionCertificateCause::Hypothesis {
        actual: a,
        expected: e,
        ..
    })) = result
    else {
        panic!("expected count refusal {actual}/{expected}");
    };
    assert_eq!((a, e), (actual, expected));
}

#[test]
fn shared_history_refuses_duplicate_actual_binding_before_face_admission() {
    let mut receipt = annular_receipt();
    duplicate_binding(&mut receipt);
    let mut admitted = false;
    let result = capture_actual_charts(&receipt, &mut || false, |_| {
        admitted = true;
        Ok(())
    });
    assert!(!admitted);
    assert_count_refusal(result, 2, 1);
}
fn duplicate_binding(receipt: &mut PrismReceipt) {
    let diagnostic = receipt.diagnostic.as_mut().unwrap();
    let row = &diagnostic.bindings[0];
    diagnostic.bindings.push(PrismImageBinding {
        native: row.native.clone(),
        context: row.context.clone(),
        transported: row.transported.clone(),
    });
}

fn assert_chart_encoding(
    chart: &BoundPrismChart,
    retained: &super::super::surface_charts::PrismSurfaceEntry,
) {
    assert_eq!(
        super::super::frozen::encode_prism_chart_geometry(&chart.captured),
        super::super::frozen::encode_prism_chart_geometry(retained.captured.as_ref().unwrap())
    );
}
