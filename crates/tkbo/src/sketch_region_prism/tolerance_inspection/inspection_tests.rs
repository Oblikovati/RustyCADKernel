// SPDX-License-Identifier: GPL-2.0-only
//! Authored cubic/extrusion closed forms remain independent of metadata values and inspection bounds.
use super::super::PrismCompletion;
use super::{inspection_fixture::*, *};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::predicates::roots::metric_bounds::ResidualDecision;

pub(super) fn assert_zero(inspection: &PrismToleranceInspection) {
    assert_eq!(inspection.endpoints.len(), 6);
    for endpoint in &inspection.endpoints {
        assert_eq!(endpoint.contexts.len(), 2);
        assert_eq!(endpoint.requested_spatial, 0.);
        assert_eq!(endpoint.effective_spatial, 0.);
        assert_eq!(endpoint.effective_roots.max_width, 1.);
        assert_eq!(endpoint.report.bisections, 0);
        assert!(
            matches!(&endpoint.report.decision, Ok(ResidualDecision::Within { proven_squared_upper }) if proven_squared_upper.compare_f64(0.).unwrap().is_eq())
        );
    }
    assert_eq!(inspection.trims.len(), 6);
    assert!(
        inspection
            .trims
            .iter()
            .all(|row| row.squared_upper.compare_f64(0.).unwrap().is_eq())
    );
}
#[test]
fn actual_cubic_has_zero_residuals_and_complete_incidence_but_unresolved_tolerance_semantics() {
    let fixture = InspectionFixture::new();
    let report = fixture.captured(ordinary_request());
    assert_zero(&report);
    assert_eq!(report.behavior_version, 1);
    assert_eq!(report.metadata.len(), 8);
    assert_eq!(report.census.contexts.len(), 26);
    assert_eq!(
        report.unresolved,
        [ToleranceRelationGap::StoredTolerancePlacementSemantics]
    );
    assert!(matches!(
        fixture.receipt.completion,
        PrismCompletion::Incomplete
    ));
    assert!(fixture.receipt.limits.achieved_error.is_none());
}
#[test]
fn inflated_metadata_report_and_refuse_do_not_change_exact_geometric_error() {
    for kind in [ShapeEnum::Vertex, ShapeEnum::Edge, ShapeEnum::Face] {
        check_inflated_metadata(kind);
    }
}
fn check_inflated_metadata(kind: ShapeEnum) {
    let fixture = InspectionFixture::new();
    set_raw(&fixture.actual(kind), 0.125);
    for (cap, expected) in metadata_caps() {
        let report = fixture.captured(request(cap, 0.));
        assert_zero(&report);
        assert_metadata_decision(&report, expected);
        assert!(matches!(
            fixture.receipt.completion,
            PrismCompletion::Incomplete
        ));
    }
}
#[test]
fn invalid_actual_raw_metadata_keeps_entity_and_offending_bits() {
    for kind in [ShapeEnum::Vertex, ShapeEnum::Edge, ShapeEnum::Face] {
        for raw in [f64::NAN, -0.125, f64::INFINITY, f64::NEG_INFINITY] {
            check_invalid(kind, raw);
        }
    }
}
fn check_invalid(kind: ShapeEnum, raw: f64) {
    let fixture = InspectionFixture::new();
    let actual = fixture.actual(kind);
    set_raw(&actual, raw);
    let error = fixture
        .inspect(
            request(StoredToleranceCap::NoLimit, 0.),
            &mut ContinueInspection,
        )
        .err()
        .unwrap();
    let ToleranceInspectionCause::Scalar { entity, cause } = error.cause else {
        panic!("expected raw scalar refusal");
    };
    // Placed cap occurrences can share the same stored metadata owner.
    let stored_owner = crate::sketch_region::PartitionOccurrenceIdentity::capture(&actual);
    assert_eq!(entity.topology_address, stored_owner.topology_address);
    assert_eq!(cause.quantity, ToleranceQuantity::RawStoredMetadata);
    assert_eq!(cause.actual.to_bits(), raw.to_bits());
}
#[test]
fn private_owned_inspection_is_send_sync_without_publishing_a_body() {
    fn check<T: Send + Sync>() {}
    check::<PrismToleranceInspection>();
}

fn metadata_caps() -> [(StoredToleranceCap, StoredCapDecision); 2] {
    [
        (
            StoredToleranceCap::report(0.0625).unwrap(),
            StoredCapDecision::Reported,
        ),
        (
            StoredToleranceCap::refuse(0.0625).unwrap(),
            StoredCapDecision::Refused,
        ),
    ]
}
fn assert_metadata_decision(report: &PrismToleranceInspection, expected: StoredCapDecision) {
    let changed: Vec<_> = report
        .metadata
        .iter()
        .filter(|row| row.observation.raw == 0.125)
        .collect();
    assert!(!changed.is_empty());
    assert!(
        changed
            .iter()
            .all(|row| row.observation.decision == expected)
    );
}
