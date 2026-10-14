// SPDX-License-Identifier: GPL-2.0-only
use super::super::domain_tests::with_request;
use super::*;
use crate::sketch_region_prism::region_certificate::tests::fixture_for;
use crate::sketch_region_prism::tests::request;
use rustycad_tkmath::gp::Vector3;

#[test]
fn complete_scaled_reversed_and_negative_sweeps_keep_independent_world_units() {
    for (scale, reversed, height) in [(0.5, false, 2.), (2., true, 2.), (1., false, -2.)] {
        let source = crate::closed_analytic_cubic::closed_analytic_cubic(scale, reversed);
        let (bridge, shapes) = fixture_for(vec![source]);
        let mut requested = request([scale, scale]);
        requested.extrusion = Vector3::new(0., 0., height);
        let receipt = with_request(&bridge, &shapes, requested);
        let report = full_report(&bridge, &shapes, &receipt, fast_options());
        assert_scaled_measure(&report, scale, height);
    }
}
#[test]
fn complete_result_retains_actual_direction_placement_height_and_both_seams() {
    let (bridge, shapes) = fixture();
    let receipt = run(&bridge, &shapes);
    let report = full_report(&bridge, &shapes, &receipt, fast_options());
    let side = report
        .binding
        .charts
        .iter()
        .find(|chart| chart.role == PrismImageRole::Generated)
        .unwrap();
    assert_eq!(side.proof.side_height, Some(2.));
    assert_eq!(side.captured.extrusion, [0., 0., 2.]);
    assert_eq!(side.captured.trims.len(), 4);
    assert!(
        matches!(&side.captured.actual, CapturedPrismSurface::Extrusion { direction, .. } if *direction == [0., 0., -1.])
    );
}

fn assert_scaled_measure(report: &CompletePrismMeasureInspection, scale: f64, height: f64) {
    let expected_side = oracle_interval("side").map(|v| v * scale * height.abs() / 2.);
    assert_contains_interval(report.measures.side.enclosure(), expected_side);
    assert!(
        report
            .measures
            .subset
            .signed_volume
            .representative()
            .signum()
            == height.signum()
    );
}
