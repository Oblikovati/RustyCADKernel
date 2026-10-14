// SPDX-License-Identifier: GPL-2.0-only
//! Owned pre-theorem operands; none of these observations promotes body completion.
use super::{tests::*, *};
use crate::sketch_region_geometry_capture::CapturedMetricCurve;
use crate::sketch_region_prism::surface_charts::*;
use rustycad_tkmath::predicates::roots::{RootIsolationOptions, metric_bounds::*};

fn chart(receipt: &PrismReceipt, role: PrismImageRole) -> &PrismSurfaceEntry {
    let entries = &receipt
        .diagnostic
        .as_ref()
        .unwrap()
        .surface_charts
        .as_ref()
        .unwrap()
        .entries;
    assert_eq!(entries.iter().filter(|row| row.role == role).count(), 1);
    let entry = entries.iter().find(|row| row.role == role).unwrap();
    assert!(entry.decision.is_ok());
    entry
}
fn trim(entry: &PrismSurfaceEntry, role: PrismTrimRole) -> &CapturedPrismTrim {
    let trims = &entry.captured.as_ref().unwrap().trims;
    assert_eq!(trims.iter().filter(|row| row.role == role).count(), 1);
    trims.iter().find(|row| row.role == role).unwrap()
}
#[test]
fn embedding_baseline_cubic_operands_do_not_promote_existing_body_gaps() {
    let (bridge, shapes) = fixture();
    let receipt = run(&bridge, &shapes);
    let region = proof(&receipt);
    let support = &region.copied.support;
    let determinant =
        affine_prism_jacobian(support.frame, support.placement, [0., 0., 2.], || false).unwrap();
    assert_eq!(
        determinant.compare_f64(2.).unwrap(),
        std::cmp::Ordering::Equal
    );
    assert_regular_boundary(
        &region.copied.boundaries[0].space,
        receipt.limits.requested.roots,
    );
    assert_eq!(
        chart(&receipt, PrismImageRole::Generated).proof.side_height,
        Some(2.)
    );
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
}
fn assert_regular_boundary(
    source: &crate::sketch_region_source_conformance::CapturedSpaceRepresentation,
    roots: RootIsolationOptions,
) {
    let CapturedMetricCurve::Bezier { poles, weights } = &source.curve else {
        panic!("owned fixture must be a Bezier")
    };
    let mut query = BezierRegularityQuery::new(41, poles, BezierRegularityOptions::v1(roots));
    query.weights = weights.as_deref();
    query.interval = source.interval;
    query.placement = source.placement;
    let report = certify_bezier_regularity(query, || false).unwrap();
    assert!(matches!(
        report.decision(),
        BezierRegularityDecision::Regular { .. }
    ));
    assert_eq!(report.source().interval(), [0., 1.]);
}
#[test]
fn embedding_baseline_actual_rims_and_seam_have_authored_directed_uses() {
    let (bridge, shapes) = fixture();
    let receipt = run(&bridge, &shapes);
    let side = chart(&receipt, PrismImageRole::Generated);
    assert_eq!(side.captured.as_ref().unwrap().trims.len(), 4);
    for (face, role) in [
        (PrismImageRole::First, PrismTrimRole::First),
        (PrismImageRole::Last, PrismTrimRole::Last),
    ] {
        let cap = chart(&receipt, face);
        assert_eq!(cap.captured.as_ref().unwrap().trims.len(), 1);
        assert!(history::same_physical(
            &trim(cap, role).output,
            &trim(side, role).output
        ));
        assert!(!trim(side, role).parents.is_empty());
    }
    let first = trim(side, PrismTrimRole::Start);
    let last = trim(side, PrismTrimRole::End);
    assert!(history::same_physical(&first.output, &last.output));
    assert!(history::same_physical(&first.source, &last.source));
    assert_eq!(first.interval, [0., 2.]);
    assert_eq!(last.interval, [0., 2.]);
    assert_ne!(first.output.orientation, last.output.orientation);
}
#[test]
fn embedding_baseline_stationary_quartic_is_jordan_but_not_regular() {
    // x=24t²(1-t)²,y=16t³(1-t); y/x=(2/3)t/(1-t) proves interior injectivity.
    let poles = [[0.; 3], [0.; 3], [4., 0., 0.], [0., 4., 0.], [0.; 3]];
    let options = BezierRegularityOptions::v1(RootIsolationOptions::new(1e-6));
    let report =
        certify_bezier_regularity(BezierRegularityQuery::new(42, &poles, options), || false)
            .unwrap();
    let BezierRegularityDecision::Stationary {
        zero_speed_isolation,
    } = report.decision()
    else {
        panic!("C'(0)=0")
    };
    assert_eq!(zero_speed_isolation.roots().len(), 1);
    let zero = &zero_speed_isolation.roots()[0];
    assert_eq!(zero.bounds().0, zero.bounds().1);
    assert_eq!(
        zero.bounds().0.compare_f64(0.).unwrap(),
        std::cmp::Ordering::Equal
    );
    assert_eq!(zero.multiplicity(), 2);
}
