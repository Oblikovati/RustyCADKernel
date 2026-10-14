// SPDX-License-Identifier: GPL-2.0-only
//! A retained cap vertex shares PrismCopy closure storage; preserve that earlier refusal.
use super::super::region_certificate::{
    RegionCertificateCause, current_binding::PrismBindingFailure,
};
use super::{inspection_fixture::*, inspection_tests::assert_zero, *};
use rustycad_tkbrep::brep::access::vertex_tshape;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::{gp::Point3, predicates::roots::metric_bounds::ResidualDecision};

#[test]
fn displaced_output_vertex_preserves_earlier_exact_copy_closure_refusal() {
    let fixture = InspectionFixture::new();
    let before = fixture.captured(ordinary_request());
    let actual = fixture.actual(ShapeEnum::Vertex);
    let address =
        crate::sketch_region::PartitionOccurrenceIdentity::capture(&actual).topology_address;
    displace_vertex(&actual);
    for budget in [0., 0.125] {
        let error = fixture
            .inspect(
                request(StoredToleranceCap::refuse(1e-6).unwrap(), budget),
                &mut ContinueInspection,
            )
            .err()
            .unwrap();
        assert_copy_refusal(error, address);
    }
    assert_zero(&before);
}
fn assert_copy_refusal(error: ToleranceInspectionFailure, address: usize) {
    assert_eq!(error.phase, ToleranceInspectionPhase::Capture);
    let ToleranceInspectionCause::Binding(cause) = error.cause else {
        panic!("expected binding refusal");
    };
    let PrismBindingFailure::Region(RegionCertificateCause::ClosurePoint(proof)) = *cause else {
        panic!("expected actual closure point failure");
    };
    assert_eq!(proof.point, [0.125, 0., 0.]);
    assert_eq!(proof.exact_certificate_limit, 0.);
    assert_eq!(proof.contexts.len(), 1);
    assert_eq!(format!("{:?}", proof.contexts[0].stage), "PrismCopy");
    assert_eq!(proof.contexts[0].occurrence.topology_address, address);
    assert!(matches!(
        proof.decision,
        Ok(ResidualDecision::Exceeds { .. })
    ));
}
#[test]
fn owned_endpoint_operands_certify_independent_one_eighth_thresholds_without_body_admission() {
    let fixture = InspectionFixture::new();
    let mut captured = fixture.captured(ordinary_request());
    for endpoint in &mut captured.census.endpoints {
        endpoint.point[0] += 0.125;
    }
    for (budget, expected_within) in [(0., false), (0.1, false), (0.125, true)] {
        let request = request(StoredToleranceCap::refuse(0.).unwrap(), budget);
        let reports =
            super::residuals::endpoint_evidence(&captured.census, request, &mut ContinueInspection)
                .ok()
                .unwrap();
        assert_eq!(reports.len(), 6);
        for row in reports {
            assert_threshold(row, budget, expected_within);
        }
    }
    // No source-bound inspection is returned for these deliberately changed owned operands.
    assert_zero(&fixture.captured(ordinary_request()));
}
fn assert_threshold(row: ToleranceEndpointEvidence, budget: f64, within: bool) {
    assert_eq!(row.requested_spatial, budget);
    assert_eq!(row.effective_spatial, budget);
    assert_eq!(row.report.bisections, 0);
    match row.report.decision.unwrap() {
        ResidualDecision::Within {
            proven_squared_upper,
        } => {
            assert!(within);
            assert!(proven_squared_upper.compare_f64(0.015625).unwrap().is_eq());
        }
        ResidualDecision::Exceeds { .. } => assert!(!within),
    }
}

fn displace_vertex(actual: &rustycad_tkbrep::topods::Shape) {
    vertex_tshape(actual, "moved endpoint")
        .unwrap()
        .set_pnt(Point3::new(0.125, 0., 0.));
}
