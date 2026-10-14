// SPDX-License-Identifier: GPL-2.0-only
//! Fixture authors names for one edge, its closure uses and each known operation-stage role.
use super::super::lineage::PrismCopyBindings;
use super::super::proof_join::{JoinedPrismProof, operation_tests::JoinFixture};
use super::*;
use crate::sketch_region::{ArrangementSource, PartitionObservation, RegionObservedContext};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

pub(super) struct HistoryFixture {
    pub operation: JoinFixture,
    pub joined: JoinedPrismProof,
    pub copied: PrismCopyBindings,
    pub historical: PrismHistoryBindings,
}
impl HistoryFixture {
    pub fn new() -> Self {
        let operation = JoinFixture::new();
        let joined = operation.compose(|| false).ok().unwrap();
        let copied = super::super::lineage::tests::authored_copy_bindings(&operation.receipt);
        let historical = additional_names(&operation, &joined);
        Self {
            operation,
            joined,
            copied,
            historical,
        }
    }
    pub fn request<'a>(&'a self, sources: &'a [ArrangementSource<'a>]) -> PrismHistoryRequest<'a> {
        PrismHistoryRequest {
            bridge: &self.operation.bridge,
            sources,
            receipt: &self.operation.receipt,
            publication: &self.operation.authored,
            copied: &self.copied,
            historical: &self.historical,
            joined: &self.joined,
            requested: PrismHistoryOptions::v1(),
        }
    }
}
fn additional_names(operation: &JoinFixture, join: &JoinedPrismProof) -> PrismHistoryBindings {
    let PartitionObservation::Captured(partition) = &operation.bridge.mapped.arrangement.partition
    else {
        panic!("expected actual partition");
    };
    let initial = operands::copied_contexts(
        &partition.input_contexts[0].observation,
        PrismHistoryStage::InputCopy,
    )
    .unwrap();
    let copier = operation.bridge.mapped.copied.as_ref().unwrap();
    let candidate =
        operands::copied_contexts(&copier.contexts, PrismHistoryStage::CandidateCopy).unwrap();
    let complement = copier
        .completion
        .as_ref()
        .ok()
        .unwrap()
        .candidates
        .iter()
        .find(|c| {
            crate::sketch_region::PartitionOccurrenceIdentity::capture(&c.face)
                == join
                    .embedding()
                    .binding()
                    .region
                    .complementary_candidate
                    .face
        })
        .unwrap();
    let complementary = operands::current_contexts(
        &complement.face,
        PrismHistoryStage::Complement,
        PrismHistoryLimits::bounded_v1(),
    )
    .unwrap();
    let intermediate = [
        (PrismHistoryStage::InputCopy, &initial.copied),
        (PrismHistoryStage::Partition, &candidate.source),
        (PrismHistoryStage::CandidateCopy, &candidate.copied),
        (PrismHistoryStage::Complement, &complementary),
    ]
    .into_iter()
    .map(|(stage, rows)| authored_stage(stage, rows))
    .collect();
    PrismHistoryBindings {
        complementary_region: AuthoredPrismId(15000),
        intermediate,
    }
}
fn authored_stage(
    stage: PrismHistoryStage,
    rows: &[RegionObservedContext],
) -> PrismHistoricalStageBindings {
    let mut members: Vec<PrismSourceBinding> = Vec::new();
    let mut uses = Vec::new();
    for row in rows {
        let member = semantic_member(stage, row.kind);
        if !members
            .iter()
            .any(|b| same_definition(&b.actual, &row.occurrence))
        {
            assert!(!members.iter().any(|b| b.member == member));
            members.push(PrismSourceBinding {
                actual: row.occurrence.clone(),
                member,
            });
        }
        uses.push(PrismHistoricalUseBinding {
            actual: row.clone(),
            authored_use: semantic_use(stage, row),
        });
    }
    PrismHistoricalStageBindings {
        stage,
        members,
        uses,
    }
}
fn semantic_member(stage: PrismHistoryStage, kind: ShapeEnum) -> AuthoredPrismId {
    let prefix = match stage {
        PrismHistoryStage::InputCopy => 11000,
        PrismHistoryStage::Partition => 12000,
        PrismHistoryStage::CandidateCopy => 13000,
        PrismHistoryStage::Complement => 14000,
        _ => panic!("fixture requires authored intermediate stage"),
    };
    AuthoredPrismId(
        prefix
            + match kind {
                ShapeEnum::Compound => 1,
                ShapeEnum::Face => 2,
                ShapeEnum::Wire => 3,
                ShapeEnum::Edge => 4,
                ShapeEnum::Vertex => 5,
                _ => panic!("fixture requires known single-Jordan semantic member role"),
            },
    )
}
fn semantic_use(stage: PrismHistoryStage, row: &RegionObservedContext) -> AuthoredPrismId {
    let member = semantic_member(stage, row.kind).0;
    let endpoint_end =
        row.kind == ShapeEnum::Vertex && row.occurrence.orientation == Orientation::Reversed;
    AuthoredPrismId(member + if endpoint_end { 200 } else { 100 })
}
#[test]
fn same_operation_fixture_has_exhaustive_historical_query_and_context_operands() {
    let fixture = HistoryFixture::new();
    let sources =
        crate::sketch_region_partition_bindings::binding_sources(&fixture.operation.shapes);
    let request = fixture.request(&sources);
    assert_eq!(
        operands::partition(&request).unwrap().input_contexts.len(),
        1
    );
    assert_eq!(fixture.historical.intermediate.len(), 4);
    for names in &fixture.historical.intermediate {
        let rows: Vec<_> = names.uses.iter().map(|b| b.actual.clone()).collect();
        contexts::bind_historical_contexts(
            request.publication.operation,
            names,
            &rows,
            request.requested.limits,
        )
        .unwrap();
    }
    assert_eq!(
        operands::generated_contexts(&request, PrismHistoryStage::Selected)
            .unwrap()
            .len(),
        5
    );
    assert_eq!(
        operands::generated_contexts(&request, PrismHistoryStage::Complement)
            .unwrap()
            .len(),
        5
    );
}

#[test]
fn full_semantic_stage_roster_binds_actual_censuses_with_no_implicit_names() {
    let fixture = HistoryFixture::new();
    let sources =
        crate::sketch_region_partition_bindings::binding_sources(&fixture.operation.shapes);
    let request = fixture.request(&sources);
    let operands = stages::capture_operands(&request).unwrap();
    let bound = stages::bind_stages(&request, &operands).unwrap();
    assert_eq!(bound.len(), 7);
    assert_eq!(
        bound
            .iter()
            .map(|s| s.contexts.owned.len())
            .collect::<Vec<_>>(),
        [3, 3, 4, 4, 5, 5, 5]
    );
    let originals = stages::stage(&bound, PrismHistoryStage::Original).unwrap();
    assert_eq!(
        stage_names::original_names(&request, originals.actual)
            .unwrap()
            .uses
            .len(),
        3
    );
}

#[test]
fn all_original_uses_have_actual_empty_partition_queries_and_unique_continuations() {
    let fixture = HistoryFixture::new();
    let sources =
        crate::sketch_region_partition_bindings::binding_sources(&fixture.operation.shapes);
    let request = fixture.request(&sources);
    let operands = stages::capture_operands(&request).unwrap();
    let bound = stages::bind_stages(&request, &operands).unwrap();
    let queries = partition_queries::project_partition_queries(&request, &bound).unwrap();
    assert_eq!(queries.len(), 3);
    assert!(
        queries
            .iter()
            .all(|q| !q.deleted && q.modified.is_empty() && q.generated.is_empty())
    );
}

#[test]
fn actual_copy_generation_and_rejected_region_operands_compose_without_identity_synthesis() {
    let fixture = HistoryFixture::new();
    let sources =
        crate::sketch_region_partition_bindings::binding_sources(&fixture.operation.shapes);
    let request = fixture.request(&sources);
    let operands = stages::capture_operands(&request).unwrap();
    let stages = stages::bind_stages(&request, &operands).unwrap();
    let history = intermediate::capture(&request, &stages).unwrap();
    assert_eq!(history.initial_copy.len(), 3);
    assert_eq!(history.candidate_copy.len(), 4);
    assert_eq!(history.candidates.copy_tree.len(), 3);
    assert_eq!(history.candidates.directed.len(), 3);
    assert_eq!(history.selected_relations.len(), 5);
    assert_eq!(history.complementary_relations.len(), 5);
}

#[test]
fn partition_metadata_preserves_native_absent_images_and_explicit_closure_uses() {
    let fixture = HistoryFixture::new();
    let sources =
        crate::sketch_region_partition_bindings::binding_sources(&fixture.operation.shapes);
    let request = fixture.request(&sources);
    let operands = stages::capture_operands(&request).unwrap();
    let stages = stages::bind_stages(&request, &operands).unwrap();
    let result = partition_metadata::capture(
        &request,
        stages::stage(&stages, PrismHistoryStage::InputCopy).unwrap(),
    )
    .unwrap();
    assert!(!result.images_present);
    assert_eq!(result.vertices.len(), 1);
    assert_eq!(result.endpoints.len(), 2);
    assert_ne!(result.endpoints[0].use_key, result.endpoints[1].use_key);
}

#[test]
fn exhaustive_sweep_queries_distinguish_retired_intermediate_wire_from_original_deletion() {
    let fixture = HistoryFixture::new();
    let sources =
        crate::sketch_region_partition_bindings::binding_sources(&fixture.operation.shapes);
    let request = fixture.request(&sources);
    let operands = stages::capture_operands(&request).unwrap();
    let stages = stages::bind_stages(&request, &operands).unwrap();
    let bound =
        super::super::bindings::bind_projection(request.receipt, request.publication).unwrap();
    let lineage = sweep_lineage::capture(&request, &bound).unwrap();
    let queries = sweep::capture(
        &request,
        &bound,
        stages::stage(&stages, PrismHistoryStage::SweepBase).unwrap(),
    )
    .unwrap();
    assert_eq!(queries.len(), 5);
    assert_eq!(queries.iter().filter(|q| q.retired).count(), 1);
    assert_eq!(lineage.retired().len(), 1);
    let retired = queries.iter().find(|q| q.retired).unwrap();
    assert!(retired.generated.is_empty());
    assert!(retired.first.is_some() && retired.last.is_some());
}

#[test]
fn complete_original_use_chains_reach_kept_caps_and_preserve_complement_rejection() {
    let fixture = HistoryFixture::new();
    let sources =
        crate::sketch_region_partition_bindings::binding_sources(&fixture.operation.shapes);
    let result = capture::capture(&fixture.request(&sources)).unwrap();
    assert_eq!(result.continued.len(), 3);
    assert!(
        result
            .continued
            .iter()
            .all(|r| !r.kept_first.is_empty() && !r.kept_last.is_empty())
    );
    assert!(result.continued.iter().all(|r| r.selected != r.complement));
    assert_eq!(result.lineage.retired().len(), 1);
    assert!(fixture.operation.receipt.gaps.iter().any(|g| matches!(
        g,
        crate::sketch_region_prism::PrismGap::ComposedDeletionCoverage
    )));
}

#[test]
fn complete_history_entry_rechecks_all_operands_and_owns_send_sync_evidence() {
    fn check<T: Send + Sync>() {}
    check::<PrismCompleteHistory>();
    check::<PrismHistoryFinding>();
    let fixture = HistoryFixture::new();
    let sources =
        crate::sketch_region_partition_bindings::binding_sources(&fixture.operation.shapes);
    let request = fixture.request(&sources);
    let result = match inspect_complete_prism_history(&request, |_| false) {
        PrismHistoryOutcome::Inspected(value) => value,
        PrismHistoryOutcome::Incomplete(f) | PrismHistoryOutcome::Failed(f) => {
            panic!("{:?}", f.finding)
        }
        PrismHistoryOutcome::Cancelled { .. } => panic!("unexpected cancellation"),
    };
    assert!(result.binds(&fixture.joined));
    assert_eq!(result.continued().len(), 3);
    assert!(result.deleted_originals().is_empty());
    assert!(!result.is_valid_body());
    assert!(reproject_complete_prism_history(&request, &result).is_ok());
}
