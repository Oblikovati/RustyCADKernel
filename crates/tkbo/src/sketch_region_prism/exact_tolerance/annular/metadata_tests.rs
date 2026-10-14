// SPDX-License-Identifier: GPL-2.0-only
//! Upstream proof metadata and actual body metadata are distinct typed observations.
use super::{tests::*, *};
use crate::sketch_region_prism::{
    exact_tolerance::{StoredToleranceCap, stored_metadata::observe_node},
    frozen::annular_join::operation_tests::joined,
    tolerance_inspection::*,
};
#[test]
fn copied_source_metadata_and_body_metadata_have_explicit_independent_provenance() {
    let join = joined();
    let proof = conformance(&join);
    assert_source_provenance(&proof);
    let observed =
        certify_annular_prism_tolerances(&join, &proof, StoredToleranceCap::NoLimit, &mut Continue)
            .ok()
            .unwrap();
    assert_body_provenance(&join, &observed);
    unequal_owned_metadata(&join);
}
fn unequal_owned_metadata(join: &AnnularFrozenDomain) {
    let mut source = super::source_tests::endpoint_proof();
    source.raw_tolerance = 0.125;
    let mut body = raw_vertex_node(join);
    body.raw_tolerance = Some(0.25);
    let request =
        PrismToleranceRequest::new(StoredToleranceCap::report(0.1875).unwrap(), 0.).unwrap();
    let observation = observe_node(join.projection(), &body, request)
        .ok()
        .unwrap();
    assert_eq!(observation.observation.raw, 0.25);
    assert_eq!(
        observation.observation.decision,
        StoredCapDecision::Reported
    );
    assert_eq!(source.raw_tolerance, 0.125);
}
#[test]
fn annular_raw_zero_subfloor_invalid_and_explicit_cap_modes_preserve_bits() {
    let join = joined();
    let mut node = raw_vertex_node(&join);
    let request = PrismToleranceRequest::new(StoredToleranceCap::NoLimit, 0.).unwrap();
    for value in [0., -0., f64::from_bits(1), 0.125] {
        node.raw_tolerance = Some(value);
        let row = observe_node(join.projection(), &node, request)
            .ok()
            .unwrap();
        assert_eq!(row.observation.raw.to_bits(), value.to_bits());
        assert_eq!(
            row.observation.effective.to_bits(),
            node.effective_tolerance.unwrap().to_bits()
        );
    }
    for value in [-1., f64::INFINITY, f64::from_bits(0x7ff8000000000123)] {
        node.raw_tolerance = Some(value);
        assert!(
            matches!(observe_node(join.projection(),&node,request),Err(JoinedToleranceCause::MetadataScalar { cause,.. }) if cause.actual.to_bits() == value.to_bits())
        );
    }
}

fn assert_source_provenance(proof: &AnnularPrismConformance) {
    for row in proof
        .evidence()
        .closures
        .iter()
        .filter(|row| row.reused_from.is_some())
    {
        assert_reused_source(proof, row);
    }
}

fn assert_body_provenance(join: &AnnularFrozenDomain, observed: &AnnularPrismTolerance) {
    for row in observed.metadata() {
        let node = join
            .projection()
            .nodes()
            .iter()
            .find(|node| node.key == row.entity)
            .unwrap();
        assert_eq!(
            row.observation.raw.to_bits(),
            node.raw_tolerance.unwrap().to_bits()
        );
        assert_eq!(
            row.observation.effective.to_bits(),
            node.effective_tolerance.unwrap().to_bits()
        );
    }
}

fn raw_vertex_node(join: &AnnularFrozenDomain) -> FrozenPrismNode {
    join.projection()
        .nodes()
        .iter()
        .find(|node| node.key.kind == 7)
        .unwrap()
        .clone()
}

fn assert_reused_source(proof: &AnnularPrismConformance, row: &AnnularClosureEvidence) {
    let origin = row.reused_from.unwrap();
    let original = proof
        .evidence()
        .closures
        .iter()
        .find(|row| row.context == origin)
        .unwrap();
    assert_eq!(
        row.source_raw_tolerance.to_bits(),
        original.source_raw_tolerance.to_bits()
    );
}
