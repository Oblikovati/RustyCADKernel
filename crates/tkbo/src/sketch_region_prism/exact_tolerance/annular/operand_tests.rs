// SPDX-License-Identifier: GPL-2.0-only
//! Malformed owned operands exercise shared drivers, never issue a counterfeit sealed join.
use super::{tests::*, *};
use crate::sketch_region_prism::{
    exact_tolerance::{StoredToleranceCap, joined_endpoints::certify_projection_endpoints},
    frozen::{annular_join::operation_tests::joined, proof_join::JoinedVertexOperand},
};
impl JoinedConformanceProgress for Continue {
    fn cancelled(&mut self, _: JoinedConformancePhase) -> bool {
        false
    }
}
fn body_refusal(join: &AnnularFrozenDomain, raw: &[JoinedVertexOperand]) -> JoinedConformanceCause {
    certify_projection_endpoints(
        join.projection(),
        raw,
        (12, 24),
        &mut ConformanceWork::new(),
        &mut Continue,
    )
    .err()
    .expect("malformed full-context operands must refuse")
}
#[test]
fn missing_duplicate_foreign_and_mixed_raw_contexts_are_refused() {
    let join = joined();
    let raw = join.raw_vertices();
    assert!(matches!(
        body_refusal(&join, &raw[1..]),
        JoinedConformanceCause::Count { .. }
    ));
    let mut changed = raw.to_vec();
    changed[1] = changed[0].clone();
    assert!(matches!(
        body_refusal(&join, &changed),
        JoinedConformanceCause::Count { .. }
    ));
    changed = raw.to_vec();
    changed[0].vertex.kind = 6;
    assert!(matches!(
        body_refusal(&join, &changed),
        JoinedConformanceCause::Mismatch { .. }
    ));
    changed = raw.to_vec();
    changed[0].local_point[0] += 0.125;
    assert!(matches!(
        body_refusal(&join, &changed),
        JoinedConformanceCause::Mismatch { .. }
    ));
}
#[test]
fn a_matching_revision_cannot_replace_full_geometry_binding() {
    let join = joined();
    let mut proof = conformance(&join);
    proof.binding[0] ^= 1;
    assert!(!proof.binds(&join));
    let failure =
        certify_annular_prism_tolerances(&join, &proof, StoredToleranceCap::NoLimit, &mut Continue)
            .err()
            .unwrap();
    assert!(matches!(
        failure.cause,
        JoinedToleranceCause::ConformanceBinding
    ));
    assert!(failure.metadata.is_empty());
}
