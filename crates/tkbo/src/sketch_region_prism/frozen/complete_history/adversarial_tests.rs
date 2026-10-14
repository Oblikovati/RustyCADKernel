// SPDX-License-Identifier: GPL-2.0-only
//! Missing observations never inherit certified empty answers; retained multiplicity is part of proof.
use super::*;
use fixture::HistoryFixture;
use crate::sketch_region::{PartitionObservation, PartitionMemberQueries, RegionCopyContextObservation};
fn refuse(mut edit: impl FnMut(&mut HistoryFixture)) {
    let mut fixture = HistoryFixture::new();
    edit(&mut fixture);
    let sources = crate::sketch_region_partition_bindings::binding_sources(&fixture.operation.shapes);
    assert!(capture::capture(&fixture.request(&sources)).is_err());
}
fn initial(fixture: &mut HistoryFixture) -> &mut crate::sketch_region::RegionCopyContexts {
    let PartitionObservation::Captured(partition) = &mut fixture.operation.bridge.mapped.arrangement.partition else { panic!("partition"); };
    let RegionCopyContextObservation::Captured(contexts) = &mut partition.input_contexts[0].observation else { panic!("contexts"); };
    contexts
}
#[test]
fn missing_duplicate_empty_and_native_failed_initial_copy_answers_refuse() {
    refuse(|f| { initial(f).queries.pop(); });
    refuse(|f| { let rows = &mut initial(f).queries; rows.push(rows[0].clone()); });
    refuse(|f| { initial(f).queries[0].images = Ok(vec![]); });
    refuse(|f| { initial(f).queries[0].images = Err(std::sync::Arc::new(rustycad_tkernel::standard::Failure::new("owned history copier failure"))); });
}
fn partition(fixture: &mut HistoryFixture) -> &mut crate::sketch_region::OwnedPartitionSnapshot {
    let PartitionObservation::Captured(partition) = &mut fixture.operation.bridge.mapped.arrangement.partition else { panic!("partition"); };
    partition
}
fn queries(fixture: &mut HistoryFixture) -> &mut Vec<crate::sketch_region::PartitionMemberQuery> {
    let PartitionMemberQueries::Captured { rows, .. } = &mut partition(fixture).member_queries else { panic!("queries"); };
    rows
}
#[test]
fn absent_duplicate_and_false_history_availability_do_not_prove_empty_deletion() {
    refuse(|f| partition(f).member_queries = PartitionMemberQueries::NotReached);
    refuse(|f| { let rows = queries(f); rows.push(rows[0].clone()); });
    refuse(|f| { queries(f).pop(); });
    refuse(|f| { queries(f)[0].deleted = Ok(true); });
    refuse(|f| { if let PartitionMemberQueries::Captured { history_available, .. } = &mut partition(f).member_queries { *history_available = false; } });
}
#[test]
fn exhaustive_candidate_links_and_generated_members_reject_gaps_duplicates_and_parent_loss() {
    refuse(|f| { f.operation.bridge.mapped.copied.as_mut().unwrap().links.pop(); });
    refuse(|f| { let rows = &mut f.operation.bridge.mapped.copied.as_mut().unwrap().links; rows.push(rows[0].clone()); });
    refuse(|f| { f.operation.bridge.mapped.copied.as_mut().unwrap().links.iter_mut().find(|r| !r.generated.is_empty()).unwrap().generated.pop(); });
    refuse(|f| { let copy = f.operation.bridge.mapped.copied.as_mut().unwrap();
        let crate::sketch_region_candidates::GeneratedTopologyObservation::Observed(rows) = &mut copy.generated_topology else { panic!("areas"); };
        rows[0].members.pop(); });
}
#[test]
fn complementary_rejected_receipt_must_equal_actual_batch_with_full_multiplicity() {
    refuse(|f| { f.operation.receipt.input.rejected.clear(); });
    refuse(|f| { f.operation.receipt.input.rejected[0][0].generated.pop(); });
    refuse(|f| { let rows = f.operation.receipt.input.selected.as_ref().unwrap().relations.clone(); f.operation.receipt.input.rejected[0] = rows; });
    refuse(|f| { let parent = f.operation.receipt.input.rejected[0][0].generated[0].clone(); f.operation.receipt.input.rejected[0][0].generated.push(parent); });
}
fn sweep_queries(fixture: &mut HistoryFixture) -> &mut Vec<crate::sketch_region_prism::PrismSweepQueryObservation> {
    let crate::sketch_region_prism::PrismSweepQueries::Captured(rows) = &mut fixture.operation.receipt.diagnostic.as_mut().unwrap().sweep_queries else { panic!("sweep"); };
    rows
}
#[test]
fn exhaustive_sweep_query_census_preserves_empty_false_and_native_retirement() {
    refuse(|f| { sweep_queries(f).pop(); });
    refuse(|f| { let rows = sweep_queries(f); rows.push(rows[0].clone()); });
    refuse(|f| { sweep_queries(f).iter_mut().find(|q| q.retired).unwrap().retired = false; });
    refuse(|f| { sweep_queries(f)[0].first = Default::default(); });
    refuse(|f| { f.operation.receipt.diagnostic.as_mut().unwrap().native_retired.clear(); });
}
#[test]
fn authored_binding_bijection_rejects_missing_extra_and_same_parent_identity_collapse() {
    refuse(|f| { f.historical.intermediate[0].uses.pop(); });
    refuse(|f| { let names = &mut f.historical.intermediate[0]; let row = &names.uses[0];
        names.uses.push(PrismHistoricalUseBinding { actual:row.actual.clone(), authored_use:AuthoredPrismId(99999) }); });
    refuse(|f| { let names = &mut f.historical.intermediate[2]; names.members[1].member = names.members[0].member; });
    refuse(|f| { f.operation.authored.originals.pop(); });
}
#[test]
fn distinct_generated_face_and_wire_keep_identity_despite_equal_original_parent_sets() {
    let fixture = HistoryFixture::new();
    let sources = crate::sketch_region_partition_bindings::binding_sources(&fixture.operation.shapes);
    let captured = capture::capture(&fixture.request(&sources)).unwrap();
    let rows = &captured.intermediate.candidates.selected;
    let face = rows.iter().find(|r| r.kind == rustycad_tkg3d::top_abs::ShapeEnum::Face as u8).unwrap();
    let wire = rows.iter().find(|r| r.kind == rustycad_tkg3d::top_abs::ShapeEnum::Wire as u8).unwrap();
    assert_eq!(face.parents, wire.parents);
    assert_ne!(face.occurrence, wire.occurrence);
}

#[test]
fn duplicated_generated_answer_cannot_replace_another_query_answer_at_equal_count() {
    refuse(|f| {
        let rows = sweep_queries(f);
        let populated: Vec<_> = rows.iter().enumerate().filter(|(_, q)| !q.generated.is_empty()).map(|(i,_)| i).collect();
        assert!(populated.len() >= 2);
        let duplicate = rows[populated[0]].generated[0].clone();
        rows[populated[0]].generated.push(duplicate);
        rows[populated[1]].generated.pop();
    });
}
#[test]
fn swapped_native_parameters_cannot_keep_the_same_original_directed_endpoint_association() {
    refuse(|f| {
        let endpoints = &mut partition(f).sources[0].source_vertices;
        assert_eq!(endpoints.len(), 2);
        let first = endpoints[0].parameter;
        endpoints[0].parameter = endpoints[1].parameter;
        endpoints[1].parameter = first;
    });
}
