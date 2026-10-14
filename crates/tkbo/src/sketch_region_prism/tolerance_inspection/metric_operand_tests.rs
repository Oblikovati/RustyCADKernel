// SPDX-License-Identifier: GPL-2.0-only
//! Independent world coordinates pin placement composition without changing body admission.
use super::{inspection_fixture::*, *};
use rustycad_tkmath::predicates::roots::{DyadicBound, metric_bounds::*};

#[test]
fn noncommuting_world_rotation_scale_and_translation_keep_exact_endpoint_equality() {
    let fixture = InspectionFixture::new();
    let mut inspection = fixture.captured(ordinary_request());
    for endpoint in &mut inspection.census.endpoints {
        endpoint.space.placement = rotated_scaled_translated(endpoint.space.placement);
        assert_eq!(endpoint.point[0..2], [0., 0.]);
        endpoint.point = [3., 5., 7. + 2. * endpoint.point[2]];
    }
    let reports = reports_for(&inspection);
    assert!(reports.iter().all(|row| matches!(&row.report.decision, Ok(ResidualDecision::Within { proven_squared_upper }) if *proven_squared_upper == DyadicBound::from_f64(0.).unwrap())));
    // Translation before rotation gives a different x/y witness, so order is load-bearing.
    for endpoint in &mut inspection.census.endpoints {
        endpoint.point[0] = -10.;
        endpoint.point[1] = 6.;
    }
    let wrong = reports_for(&inspection);
    assert_all_exceed(&wrong);
}
fn rotated_scaled_translated(placement: MetricEmbedding<3>) -> MetricEmbedding<3> {
    MetricEmbedding {
        linear: [
            placement.linear[1].map(|v| -v),
            placement.linear[0],
            placement.linear[2],
        ],
        origin: [
            3. - 2. * placement.origin[1],
            5. + 2. * placement.origin[0],
            7. + 2. * placement.origin[2],
        ],
        scale: 2. * placement.scale,
    }
}
#[test]
fn invalid_owned_parameter_keeps_incomplete_cause_and_incidence_context() {
    let fixture = InspectionFixture::new();
    let mut inspection = fixture.captured(ordinary_request());
    let index = bezier_endpoint(&inspection);
    inspection.census.endpoints[index].parameter = 2.;
    let reports = reports_for(&inspection);
    assert!(matches!(
        reports[index].report.decision,
        Err(ResidualError::ParameterDomain { .. })
    ));
    assert_operand_context(&reports[index], &inspection.census.endpoints[index]);
}
fn reports_for(inspection: &PrismToleranceInspection) -> Vec<ToleranceEndpointEvidence> {
    super::residuals::endpoint_evidence(
        &inspection.census,
        ordinary_request(),
        &mut ContinueInspection,
    )
    .ok()
    .unwrap()
}
fn bezier_endpoint(inspection: &PrismToleranceInspection) -> usize {
    inspection
        .census
        .endpoints
        .iter()
        .position(|row| {
            matches!(
                row.space.curve,
                crate::sketch_region_geometry_capture::CapturedMetricCurve::Bezier { .. }
            )
        })
        .unwrap()
}
fn assert_operand_context(
    report: &ToleranceEndpointEvidence,
    operand: &super::census_types::ToleranceEndpointCensus,
) {
    assert_eq!(report.edge, operand.edge);
    assert_eq!(report.vertex, operand.vertex);
    assert_eq!(report.contexts, operand.contexts);
}

fn assert_all_exceed(rows: &[ToleranceEndpointEvidence]) {
    assert!(
        rows.iter()
            .all(|row| matches!(row.report.decision, Ok(ResidualDecision::Exceeds { .. })))
    );
}
