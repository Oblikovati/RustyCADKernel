// SPDX-License-Identifier: GPL-2.0-only
//! Fresh hypotheses reject edited receipts even when stale success rows and gaps remain.
use super::{tests::*, *};
use crate::sketch_region_candidates::CandidateBatch;

fn batch_mut(bridge: &mut PairedContactTransaction) -> &mut CandidateBatch {
    bridge
        .mapped
        .copied
        .as_mut()
        .unwrap()
        .completion
        .as_mut()
        .ok()
        .unwrap()
}
fn missing_complement(bridge: &mut PairedContactTransaction) {
    let batch = batch_mut(bridge);
    batch
        .candidates
        .retain(|candidate| !candidate.wires[0].segments[0].reversed);
}
fn extra_face(bridge: &mut PairedContactTransaction) {
    let batch = batch_mut(bridge);
    let duplicate = temporary_candidate(&batch.candidates[0]);
    batch.candidates.push(duplicate);
}
fn missing_source_domain(bridge: &mut PairedContactTransaction) {
    bridge.census.domains.clear();
    bridge.census.missing.clear();
}
fn missing_generation_capture(bridge: &mut PairedContactTransaction) {
    let copy = bridge.mapped.copied.as_mut().unwrap();
    copy.generated_topology =
        crate::sketch_region_candidates::GeneratedTopologyObservation::NotReached;
}
fn broken_original_parent(bridge: &mut PairedContactTransaction) {
    let batch = batch_mut(bridge);
    batch.candidates[0].relations[0].generated.clear();
}
fn stale_source_basis(bridge: &mut PairedContactTransaction) {
    bridge.census.inputs.segments[0].controls[1][0] = 5.;
    bridge.census.missing.clear();
}
#[test]
fn failed_hypotheses_keep_all_four_region_gaps() {
    let mutations: [fn(&mut PairedContactTransaction); 6] = [
        missing_complement,
        extra_face,
        missing_source_domain,
        missing_generation_capture,
        broken_original_parent,
        stale_source_basis,
    ];
    for mutate in mutations {
        let (mut bridge, shapes) = fixture();
        mutate(&mut bridge);
        assert_four_present(&run(&bridge, &shapes));
    }
}
#[test]
fn actual_candidate_order_is_not_a_contact_or_region_identity() {
    let (mut bridge, shapes) = fixture();
    let original = run(&bridge, &shapes);
    batch_mut(&mut bridge).candidates.reverse();
    let reordered = run(&bridge, &shapes);
    assert_eq!(
        proof(&original).source_candidate.face,
        proof(&reordered).source_candidate.face
    );
    assert_eq!(proof(&original).contact, proof(&reordered).contact);
    assert_eq!(proof(&reordered).selected_winding, 1);
}
#[test]
fn zero_current_sweep_jacobian_is_unproved_region_admission() {
    let (bridge, shapes) = fixture();
    let mut receipt = run(&bridge, &shapes);
    receipt.limits.requested.extrusion = rustycad_tkmath::gp::Vector3::new(1., 0., 0.);
    super::super::affine_chart::retain_affine_chart(&mut receipt);
    let decision = inspect::inspect(
        &bridge,
        &crate::sketch_region_partition_bindings::binding_sources(&shapes),
        &receipt,
    );
    assert!(matches!(
        decision,
        Err(RegionCertificateCause::Hypothesis {
            name: "nondegenerate support and requested sweep",
            ..
        })
    ));
}

#[test]
fn declared_seed_plane_cannot_change_while_actual_source_captures_remain_unchanged() {
    let (mut bridge, shapes) = fixture();
    batch_mut(&mut bridge)
        .plane
        .set_location(rustycad_tkmath::gp::Point3::new(0., 0., 1.));
    let receipt = run(&bridge, &shapes);
    assert_four_present(&receipt);
    assert!(matches!(
        receipt.region.unwrap().decision,
        Err(RegionCertificateCause::Hypothesis {
            name: "declared seed plane and actual support",
            ..
        })
    ));
}
