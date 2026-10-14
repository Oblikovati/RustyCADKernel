// SPDX-License-Identifier: GPL-2.0-only
//! Terminal callback edits must be seen by the one source check and extra-history reproject.
use super::{fixture::*, progress::*, result::*, stage_error::*, *};
use crate::sketch_region_partition_bindings::binding_sources;
use crate::sketch_region_prism::frozen::publication_read_snapshot::read_snapshot;
use rustycad_tkbrep::topods::{Shape, ShapeIterator};
use rustycad_tkernel::standard::Failure;

struct CommitMutation {
    action: Box<dyn FnMut() + Send>,
    commits: usize,
}
impl PrismPublicationProgress for CommitMutation {
    fn checkpoint(&mut self, point: PrismPublicationCheckpoint) -> Result<bool, Failure> {
        if point.phase == PrismPublicationPhase::Commit {
            self.commits += 1;
            (self.action)();
        }
        Ok(false)
    }
}
fn first_vertex(root: &Shape) -> Shape {
    crate::sketch_region_prism::contexts::body_contexts(root)
        .ok()
        .unwrap()
        .into_iter()
        .find(|row| {
            row.occurrence.shape_type().ok() == Some(rustycad_tkg3d::top_abs::ShapeEnum::Vertex)
        })
        .unwrap()
        .occurrence
}
fn vertex_edit(target: Shape) -> CommitMutation {
    CommitMutation {
        commits: 0,
        action: Box::new(move || {
            rustycad_tkbrep::brep::access::vertex_tshape(&target, "publication final vertex edit")
                .unwrap()
                .set_pnt(rustycad_tkmath::gp::Point3::new(0.25, 0., 0.));
        }),
    }
}
#[test]
fn final_source_and_body_raw_vertex_edits_refuse_without_mutating_previous_result() {
    for body in [false, true] {
        let fixture = PublicationFixture::new();
        let previous = fixture.publish(&mut PublicationObserver::default());
        let bytes = previous.replay_bytes().to_vec();
        let fresh = read_snapshot(&previous);
        let root = if body {
            &fixture.operation.receipt.diagnostic.as_ref().unwrap().body
        } else {
            &fixture.operation.shapes[0]
        };
        let mut progress = vertex_edit(first_vertex(root));
        let sources = binding_sources(&fixture.operation.shapes);
        let failure = publish_named_prism(&fixture.request(&sources), &mut progress)
            .err()
            .unwrap();
        assert_eq!(failure.phase, PrismPublicationPhase::Commit);
        assert_eq!(progress.commits, 1);
        assert_eq!(previous.replay_bytes(), bytes);
        assert_eq!(read_snapshot(&previous), fresh);
    }
}
#[test]
fn final_partition_tree_edit_is_rejected_before_publication() {
    let fixture = PublicationFixture::new();
    let previous = fixture.publish(&mut PublicationObserver::default());
    let mut progress = partition_edit(&fixture);
    let sources = binding_sources(&fixture.operation.shapes);
    let failure = publish_named_prism(&fixture.request(&sources), &mut progress)
        .err()
        .unwrap();
    assert_eq!(failure.phase, PrismPublicationPhase::Commit);
    assert!(matches!(failure.cause, PrismPublicationCause::Stage(_)));
    assert_eq!(progress.commits, 1);
    assert!(fixture.operation.rebind(previous.join()).is_err());
    assert!(!previous.replay_bytes().is_empty());
}
fn partition_edit(fixture: &PublicationFixture) -> CommitMutation {
    let crate::sketch_region::ArrangementCompletion::Arranged(arranged) =
        &fixture.operation.bridge.mapped.arrangement.completion
    else {
        panic!("expected actual arrangement");
    };
    let mut root = arranged.arrangement.clone();
    let child = ShapeIterator::new(&root, true, true)
        .value()
        .unwrap()
        .clone();
    CommitMutation {
        commits: 0,
        action: Box::new(move || {
            root.set_free(true).unwrap();
            rustycad_tkbrep::topods::Builder
                .remove(&mut root, &child)
                .unwrap();
        }),
    }
}
#[test]
fn rejected_receipt_change_cannot_reuse_complete_history_or_old_result_ownership() {
    let mut fixture = PublicationFixture::new();
    let previous = fixture.publish(&mut PublicationObserver::default());
    let bytes = previous.replay_bytes().to_vec();
    let fresh = read_snapshot(&previous);
    fixture.operation.receipt.input.rejected[0][0]
        .generated
        .pop();
    let sources = binding_sources(&fixture.operation.shapes);
    let failure = publish_named_prism(
        &fixture.request(&sources),
        &mut PublicationObserver::default(),
    )
    .err()
    .unwrap();
    assert_eq!(failure.phase, PrismPublicationPhase::History);
    assert_eq!(failure.class, PrismPublicationClass::Incomplete);
    assert_eq!(previous.replay_bytes(), bytes);
    assert_eq!(read_snapshot(&previous), fresh);
}

#[test]
fn extra_partition_query_failure_is_rechecked_after_unchanged_shared_join() {
    let mut fixture = PublicationFixture::new();
    let previous = fixture.publish(&mut PublicationObserver::default());
    let bytes = previous.replay_bytes().to_vec();
    let fresh = read_snapshot(&previous);
    let crate::sketch_region::PartitionObservation::Captured(partition) =
        &mut fixture.operation.bridge.mapped.arrangement.partition
    else {
        panic!("expected retained partition");
    };
    let crate::sketch_region::PartitionMemberQueries::Captured { rows, .. } =
        &mut partition.member_queries
    else {
        panic!("expected exhaustive member queries");
    };
    rows[0].deleted = Err(std::sync::Arc::new(Failure::out_of_range(
        "changed original deletion query; expected completed answer",
    )));
    let sources = binding_sources(&fixture.operation.shapes);
    let request = fixture.request(&sources);
    assert!(request.source.rebind(previous.join()).is_ok());
    let failure = history::reproject(&request, &previous).err().unwrap();
    assert_eq!(failure.phase, PrismPublicationPhase::Commit);
    assert!(matches!(failure.cause, PrismPublicationCause::History(_)));
    assert_eq!(previous.replay_bytes(), bytes);
    assert_eq!(read_snapshot(&previous), fresh);
}

#[test]
fn fresh_read_views_survive_all_operation_inputs_being_dropped() {
    let (result, fresh) = {
        let fixture = PublicationFixture::new();
        let result = fixture.publish(&mut PublicationObserver::default());
        let fresh = read_snapshot(&result);
        (result, fresh)
    };
    assert_eq!(read_snapshot(&result), fresh);
    assert!(result.validation().binds(result.join()));
    assert!(result.history().binds(result.join()));
    assert!(result.measures().matches_join(result.join()));
}
