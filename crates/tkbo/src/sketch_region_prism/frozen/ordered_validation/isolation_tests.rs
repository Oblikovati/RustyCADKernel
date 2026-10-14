// SPDX-License-Identifier: GPL-2.0-only
use super::{test_support::*, *};
use rustycad_tkbrep::topods::Shape;
fn body_vertex(fixture: &JoinFixture) -> Shape {
    let contexts = crate::sketch_region_prism::contexts::body_contexts(
        &fixture.receipt.diagnostic.as_ref().unwrap().body,
    )
    .ok()
    .unwrap();
    contexts
        .into_iter()
        .find(|row| {
            row.occurrence.shape_type().unwrap() == rustycad_tkg3d::top_abs::ShapeEnum::Vertex
        })
        .unwrap()
        .occurrence
}
fn move_vertex(vertex: &Shape) {
    let raw =
        rustycad_tkbrep::brep::access::vertex_tshape(vertex, "owned ordered-validation mutation")
            .unwrap();
    raw.set_pnt(rustycad_tkmath::gp::Point3::new(0.25, 0., 0.));
}
#[test]
fn immutable_receipt_survives_source_mutation_but_final_live_rebind_refuses() {
    let fixture = JoinFixture::new();
    let joined = fixture.compose(|| false).ok().unwrap();
    let proof = complete(&joined);
    let bytes = proof.replay_bytes();
    move_vertex(&body_vertex(&fixture));
    assert!(proof.binds(&joined));
    assert_eq!(proof.replay_bytes(), bytes);
    assert_eq!(complete(&joined).replay_bytes(), bytes);
    assert!(fixture.rebind(&joined).is_err());
}
#[test]
fn moved_live_vertex_is_refused_by_join_before_ordered_validation() {
    use super::super::proof_join::{PrismProofJoinCause, PrismProofJoinPhase};
    let fixture = JoinFixture::new();
    move_vertex(&body_vertex(&fixture));
    let refused = fixture
        .compose(|| false)
        .err()
        .expect("expected stale live source refusal");
    assert_eq!(refused.phase, PrismProofJoinPhase::Embedding);
    assert!(matches!(refused.cause, PrismProofJoinCause::Embedding(_)));
}
