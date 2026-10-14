// SPDX-License-Identifier: GPL-2.0-only
//! Arithmetic receipt replay excludes process-local topology addresses; it is not a body serializer.
use super::{inspection_fixture::*, *};
use std::fmt::Write;

fn arithmetic_receipt(report: &PrismToleranceInspection) -> String {
    let mut bytes = format!(
        "version={} request={:?} gaps={:?}\n",
        report.behavior_version, report.requested, report.unresolved
    );
    census_receipt(report, &mut bytes);
    metadata_receipt(report, &mut bytes);
    endpoint_receipt(report, &mut bytes);
    for trim in &report.trims {
        writeln!(bytes, "trim={:?} {:?}", trim.oracle, trim.squared_upper).unwrap();
    }
    bytes
}
fn metadata_receipt(report: &PrismToleranceInspection, bytes: &mut String) {
    for row in &report.metadata {
        let entity = metadata_entity(report, &row.entity);
        writeln!(
            bytes,
            "metadata={:?} contexts={} raw={:016x} effective={:016x} decision={:?} policy={:?}",
            entity.kind,
            entity.contexts.len(),
            row.observation.raw.to_bits(),
            row.observation.effective.to_bits(),
            row.observation.decision,
            row.observation.policy
        )
        .unwrap();
    }
}
fn endpoint_receipt(report: &PrismToleranceInspection, bytes: &mut String) {
    for row in &report.endpoints {
        let operand = report
            .census
            .endpoints
            .iter()
            .find(|operand| operand.edge == row.edge && operand.vertex == row.vertex)
            .unwrap();
        writeln!(bytes, "endpoint={:016x} point={:?} contexts={} requested={:016x} effective={:016x} roots={:?} bisections={} decision={:?}", row.parameter.to_bits(), operand.point.map(f64::to_bits), row.contexts.len(), row.requested_spatial.to_bits(), row.effective_spatial.to_bits(), row.effective_roots, row.report.bisections, row.report.decision).unwrap();
    }
}
fn fresh_arithmetic_receipt() -> String {
    let fixture = InspectionFixture::new();
    let report = fixture.captured(ordinary_request());
    arithmetic_receipt(&report)
}
#[test]
fn tolerance_arithmetic_replays_across_repeated_operations_and_independent_workers() {
    let expected = fresh_arithmetic_receipt();
    for _ in 0..3 {
        assert_eq!(fresh_arithmetic_receipt(), expected);
    }
    std::thread::scope(|scope| {
        let workers: Vec<_> = (0..4)
            .map(|_| scope.spawn(fresh_arithmetic_receipt))
            .collect();
        for worker in workers {
            assert_eq!(worker.join().unwrap(), expected);
        }
    });
    let hex: String = expected
        .as_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    println!("prism-tolerance-arithmetic-v1:{hex}");
}
#[test]
fn owned_arithmetic_evidence_survives_source_metadata_mutation() {
    let fixture = InspectionFixture::new();
    let report = fixture.captured(ordinary_request());
    let before = arithmetic_receipt(&report);
    set_raw(
        &fixture.actual(rustycad_tkg3d::top_abs::ShapeEnum::Face),
        0.125,
    );
    assert_eq!(arithmetic_receipt(&report), before);
    assert_ne!(
        arithmetic_receipt(&fixture.captured(ordinary_request())),
        before
    );
}

fn census_receipt(report: &PrismToleranceInspection, bytes: &mut String) {
    writeln!(
        bytes,
        "contexts={} entities={} trims={} history={} endpoints={}",
        report.census.contexts.len(),
        report.census.entities.len(),
        report.census.trims.len(),
        report.census.history.len(),
        report.census.endpoints.len()
    )
    .unwrap();
}

fn metadata_entity<'a>(
    report: &'a PrismToleranceInspection,
    key: &crate::sketch_region::PartitionOccurrenceIdentity,
) -> &'a super::census_types::ToleranceEntityCensus {
    report
        .census
        .entities
        .iter()
        .find(|entity| entity.entity == *key)
        .unwrap()
}
