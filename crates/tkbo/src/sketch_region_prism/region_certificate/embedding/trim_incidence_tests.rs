// SPDX-License-Identifier: GPL-2.0-only
//! These checks preserve supplied trim operands; they cannot issue a plural embedding.
use super::{trim_incidence::*, *};

fn bound_report() -> PrismEmbeddingEvidence {
    let (bridge, shapes) = super::super::tests::fixture();
    let receipt = super::super::tests::run(&bridge, &shapes);
    super::tests::embedded(&bridge, &shapes, &receipt)
}

#[test]
fn shared_seam_conclusion_preserves_the_existing_owned_row() {
    let report = bound_report();
    let expected = &report.incidences()[2].conclusion;
    let PrismIncidenceConclusion::InjectiveModuloSeam { start, end, .. } = expected else {
        panic!("actual {expected:?}; expected authored seam")
    };
    assert_eq!(seam_conclusion(start, end).unwrap(), *expected);
    let mut foreign = end.clone();
    foreign.source.topology_address += 1;
    assert!(seam_conclusion(start, &foreign).is_err());
    let mut foreign = end.clone();
    foreign.output.topology_address += 1;
    assert!(seam_conclusion(start, &foreign).is_err());
}

#[test]
fn shared_rim_conclusion_preserves_both_caps_and_physical_correspondence() {
    let report = bound_report();
    for expected in report.incidences()[4..].iter().map(|row| &row.conclusion) {
        check_retained_rim(expected);
    }
}

fn check_retained_rim(expected: &PrismIncidenceConclusion) {
    let (cap, side, normalized_sweep) = rim_operands(expected);
    assert_eq!(
        rim_conclusion(cap, side, *normalized_sweep).unwrap(),
        *expected
    );
    let mut foreign = side.clone();
    foreign.source.topology_address += 1;
    assert!(rim_conclusion(cap, &foreign, *normalized_sweep).is_err());
    let mut foreign = side.clone();
    foreign.output.topology_address += 1;
    assert!(rim_conclusion(cap, &foreign, *normalized_sweep).is_err());
}

fn rim_operands(
    expected: &PrismIncidenceConclusion,
) -> (&PrismAuthoredTrimUse, &PrismAuthoredTrimUse, &f64) {
    let PrismIncidenceConclusion::ExactlyRim {
        cap,
        side,
        normalized_sweep,
        ..
    } = expected
    else {
        panic!("actual {expected:?}; expected authored rim")
    };
    (cap, side, normalized_sweep)
}

#[test]
fn shared_trim_retention_keeps_every_directed_field_and_parent_multiplicity() {
    let report = bound_report();
    for chart in &report.binding().charts {
        let domain = report
            .domains()
            .iter()
            .find(|row| row.face == chart.output)
            .unwrap();
        for (captured, expected) in chart.captured.trims.iter().zip(&domain.trims) {
            check_retained_trim(chart, captured, expected);
        }
    }
}

fn check_retained_trim(
    chart: &BoundPrismChart,
    captured: &CapturedPrismTrim,
    expected: &PrismAuthoredTrimUse,
) {
    assert_eq!(retain_trim(chart, captured), *expected);
    let mut repeated = captured.clone();
    repeated.parents.push(captured.parents[0].clone());
    let retained = retain_trim(chart, &repeated);
    assert_eq!(retained.parents, repeated.parents);
    assert_eq!(retained.parents.len(), expected.parents.len() + 1);
}
