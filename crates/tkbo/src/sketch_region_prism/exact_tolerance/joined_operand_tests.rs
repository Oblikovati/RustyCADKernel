// SPDX-License-Identifier: GPL-2.0-only
use super::{endpoint_uses::*, joined_progress::ConformanceWork, joined_test_support::*, *};
use crate::sketch_region_prism::frozen::proof_join::JoinedVertexOperand;
use rustycad_tkg3d::top_abs::Orientation;
fn first_obligation(join: &JoinedPrismProof) -> EndpointObligation {
    let (edge, vertex, _) = join.projection().incidence().endpoints[0];
    let curve = join
        .projection()
        .nodes()
        .iter()
        .find(|node| node.key == edge)
        .unwrap()
        .curve
        .as_ref()
        .unwrap();
    EndpointObligation {
        edge,
        vertex,
        side: CanonicalEndpointSide::First,
        parameter: curve.interval[0],
    }
}
fn endpoint_check(
    join: &JoinedPrismProof,
    raw: &[JoinedVertexOperand],
) -> Result<(), JoinedConformanceCause> {
    endpoint_uses(
        join.projection(),
        raw,
        first_obligation(join),
        &mut ConformanceWork::new(),
    )
    .map(|_| ())
}
#[test]
fn missing_duplicate_and_stale_raw_body_contexts_refuse_before_reuse() {
    let join = JoinFixture::new().compose(|| false).ok().unwrap();
    let chosen = endpoint_uses(
        join.projection(),
        join.raw_vertices(),
        first_obligation(&join),
        &mut ConformanceWork::new(),
    )
    .ok()
    .unwrap();
    let key = chosen[0].occurrence.clone();
    let raw: Vec<_> = join
        .raw_vertices()
        .iter()
        .filter(|row| row.occurrence != key)
        .cloned()
        .collect();
    assert!(endpoint_check(&join, &raw).is_err());
    let mut duplicate = join.raw_vertices().to_vec();
    duplicate.push(chosen[0].clone());
    assert!(endpoint_check(&join, &duplicate).is_err());
    let mut stale = join.raw_vertices().to_vec();
    stale
        .iter_mut()
        .find(|row| row.occurrence == key)
        .unwrap()
        .placement
        .origin[0] += 1.;
    assert!(matches!(
        endpoint_check(&join, &stale),
        Err(JoinedConformanceCause::Mismatch { .. })
    ));
}
#[test]
fn physical_endpoint_reuse_requires_raw_local_bit_equality() {
    let join = JoinFixture::new().compose(|| false).ok().unwrap();
    let proof = conformance(&join);
    let key = proof.evidence().endpoints[0].contexts[0].clone();
    let mut raw = join.raw_vertices().to_vec();
    raw.iter_mut()
        .find(|row| row.occurrence == key)
        .unwrap()
        .local_point[0] += 0.125;
    assert!(matches!(
        endpoint_check(&join, &raw),
        Err(JoinedConformanceCause::Mismatch { .. })
    ));
}
#[test]
fn closed_edges_retain_distinct_native_endpoints_and_both_seam_directions() {
    let join = JoinFixture::new().compose(|| false).ok().unwrap();
    let proof = conformance(&join);
    for (edge, first, last) in &join.projection().incidence().endpoints {
        let rows: Vec<_> = proof
            .evidence()
            .endpoints
            .iter()
            .filter(|row| row.edge == *edge)
            .collect();
        assert_eq!(rows.len(), 2);
        assert_ne!(rows[0].parameter.to_bits(), rows[1].parameter.to_bits());
        assert_eq!((rows[0].vertex, rows[1].vertex), (*first, *last));
        assert_eq!(
            (rows[0].side, rows[1].side),
            (CanonicalEndpointSide::First, CanonicalEndpointSide::Last)
        );
    }
}
#[test]
fn semantic_closure_missing_duplicate_orientation_and_stale_operands_refuse() {
    let join = JoinFixture::new().compose(|| false).ok().unwrap();
    let mut raw = join.raw_closure_points().to_vec();
    let index = raw
        .iter()
        .position(|row| row.stage == "original" && row.orientation == Orientation::Reversed)
        .unwrap();
    raw[index].local_point[0] += 0.125;
    assert!(matches!(
        super::joined_closures::closure_rows(&raw, "original"),
        Err(JoinedConformanceCause::ClosureOperands { .. })
    ));
    raw[index] = join.raw_closure_points()[index].clone();
    raw[index].orientation = Orientation::Forward;
    assert!(super::joined_closures::closure_rows(&raw, "original").is_err());
    raw.remove(index);
    assert!(matches!(
        super::joined_closures::closure_rows(&raw, "original"),
        Err(JoinedConformanceCause::ClosureCensus { actual: 1, .. })
    ));
}
