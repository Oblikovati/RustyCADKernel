// SPDX-License-Identifier: GPL-2.0-only
use super::{copy_tests::*, generated_bindings::*};
use crate::sketch_region_candidates::{
    CandidateCopyInspection, GeneratedMemberSnapshot, GeneratedTopologyObservation,
};

fn generated_copy() -> CandidateCopyInspection {
    let (_, owned) = actual_copy_inputs();
    actual_candidate_copy(&owned, None)
}
fn generated_leaf(copy: &mut CandidateCopyInspection) -> &mut GeneratedMemberSnapshot {
    let GeneratedTopologyObservation::Observed(areas) = &mut copy.generated_topology else {
        panic!("expected actual generated stage");
    };
    areas
        .iter_mut()
        .flat_map(|area| &mut area.members)
        .find(|row| !row.modified.is_empty() && !row.copy_indices.is_empty())
        .unwrap()
}
#[test]
fn generated_stage_parent_omission_is_not_hidden_by_successful_capture() {
    let mut copy = generated_copy();
    generated_leaf(&mut copy).modified.clear();
    assert!(copy.completion.is_ok());
    assert!(!inspect_generated_bindings(&copy).faults.is_empty());
}
#[test]
fn generated_stage_duplicate_parent_is_not_accepted_as_complete_set() {
    let mut copy = generated_copy();
    let member = generated_leaf(&mut copy);
    member.modified.push(member.modified[0].clone());
    assert!(!inspect_generated_bindings(&copy).faults.is_empty());
}
#[test]
fn generated_stage_wrong_kinded_parent_is_distinct_from_same_parent_membership() {
    let mut copy = generated_copy();
    let member = generated_leaf(&mut copy);
    member.generated = std::mem::take(&mut member.modified);
    assert!(!inspect_generated_bindings(&copy).faults.is_empty());
}
#[test]
fn generated_stage_wrong_copy_index_is_not_recovered_from_parent_sets() {
    let mut copy = generated_copy();
    generated_leaf(&mut copy).copy_indices[0] = usize::MAX;
    assert!(!inspect_generated_bindings(&copy).faults.is_empty());
}
#[test]
fn generated_stage_kind_mismatch_rejects_existing_physical_index() {
    let mut copy = generated_copy();
    generated_leaf(&mut copy).kind = rustycad_tkg3d::top_abs::ShapeEnum::Solid;
    assert!(!inspect_generated_bindings(&copy).faults.is_empty());
}
