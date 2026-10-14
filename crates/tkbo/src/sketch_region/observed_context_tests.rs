// SPDX-License-Identifier: GPL-2.0-only
use super::*;
use crate::sketch_region_candidates::CandidateCause;
use rustycad_tkbrep::topods::borrowed_shape_batch;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPICopy, BRepBuilderAPIMakeShape};

fn captured(observed: &RegionCopyContextObservation) -> &RegionCopyContexts {
    let RegionCopyContextObservation::Captured(rows) = observed else {
        panic!("expected actual complete context observations");
    };
    rows
}
fn assert_context_counts(observed: &RegionCopyContextObservation, expected: usize) {
    let rows = captured(observed);
    assert_eq!(rows.source.len(), expected);
    assert_eq!(rows.copied.len(), expected);
    assert_eq!(rows.queries.len(), expected);
    for query in &rows.queries {
        assert_eq!(query.images.as_ref().unwrap().len(), 1);
        assert!(rows.source.contains(&query.source));
    }
}
#[test]
fn original_candidate_and_generated_stages_keep_actual_directed_censuses() {
    let edge = crate::closed_analytic_cubic::closed_analytic_cubic(1., false);
    let (_, mapped, _) =
        crate::sketch_region_contact_transport::test_fixture::actual_inputs_for(vec![edge]);
    let PartitionObservation::Captured(partition) = &mapped.arrangement.partition else {
        panic!("expected partition");
    };
    assert_eq!(partition.input_contexts.len(), 1);
    assert_eq!(partition.input_contexts[0].source, RegionSourceId(1));
    assert_context_counts(&partition.input_contexts[0].observation, 3);
    let candidate = mapped.copied.as_ref().unwrap();
    assert_context_counts(&candidate.contexts, 4);
    assert_generated_contexts(candidate);
}
fn assert_generated_contexts(candidate: &crate::sketch_region_candidates::CandidateCopyInspection) {
    use crate::sketch_region_candidates::GeneratedTopologyObservation;
    let GeneratedTopologyObservation::Observed(areas) = &candidate.generated_topology else {
        panic!("expected areas");
    };
    assert_eq!(areas.len(), 2);
    for area in areas {
        let RegionContextObservation::Captured(rows) = &area.contexts else {
            panic!("expected contexts");
        };
        assert_eq!(rows.len(), 5);
        assert!(
            rows.iter()
                .all(|row| row.ancestors.len() == row.child_slots.len())
        );
    }
}
#[test]
fn repeated_context_queries_remain_distinct_with_the_same_native_copy_image() {
    let edge = crate::closed_analytic_cubic::closed_analytic_cubic(1., false);
    let root = borrowed_shape_batch(&[&edge, &edge]).unwrap();
    let mut copier = BRepBuilderAPICopy::with_shape(&root, true, true).unwrap();
    let copied = copier.shape().unwrap().clone();
    let observation = observe_copy_contexts(&root, &copied, &mut copier);
    assert_context_counts(&observation, 7);
    let rows = captured(&observation);
    let first = query_at(rows, &[0]);
    let second = query_at(rows, &[1]);
    assert_eq!(
        first.images.as_ref().unwrap(),
        second.images.as_ref().unwrap()
    );
    assert_ne!(first.source.child_slots, second.source.child_slots);
}
#[test]
fn context_snapshot_keeps_native_read_failure() {
    let root =
        crate::changing_child_count::changing_shape(rustycad_tkg3d::top_abs::ShapeEnum::Face);
    let RegionContextObservation::Incomplete(RegionContextReadFailure::Native(cause)) =
        observe_region_contexts(&root)
    else {
        panic!("expected actual iterator failure");
    };
    assert_eq!(cause.exception_type(), "Standard_NoSuchObject");
}
#[test]
fn owned_context_snapshot_and_query_failure_payloads_are_send_sync() {
    fn require<T: Send + Sync>() {}
    require::<RegionObservedContext>();
    require::<RegionContextObservation>();
    require::<RegionContextCopyQuery>();
    require::<RegionCopyContextObservation>();
    require::<RegionContextReadFailure>();
}

#[test]
fn cancelled_candidate_copy_has_no_invented_context_capture() {
    let edge = crate::closed_analytic_cubic::closed_analytic_cubic(1., false);
    let (_, mapped, _) =
        crate::sketch_region_contact_transport::test_fixture::actual_inputs_for(vec![edge]);
    let ArrangementCompletion::Arranged(owned) = &mapped.arrangement.completion else {
        panic!("expected arranged stage");
    };
    let stopped = crate::sketch_region_candidates::observe_candidate_copies(
        owned,
        rustycad_tkmath::gp::Plane3::default(),
        &mut |_| true,
        PartitionOccurrenceIdentity::capture,
    );
    assert!(matches!(
        stopped.contexts,
        RegionCopyContextObservation::NotReached
    ));
    assert!(matches!(
        stopped.completion,
        Err(CandidateCause::Cancelled(_))
    ));
}
#[test]
fn unrelated_copy_queries_retain_native_errors_instead_of_empty_success() {
    let source = crate::closed_analytic_cubic::closed_analytic_cubic(1., false);
    let other = crate::closed_analytic_cubic::closed_analytic_cubic(2., false);
    let mut copier = BRepBuilderAPICopy::with_shape(&source, true, true).unwrap();
    let copied = copier.shape().unwrap().clone();
    let observation = observe_copy_contexts(&other, &copied, &mut copier);
    let rows = captured(&observation);
    assert_eq!(rows.queries.len(), 3);
    assert!(rows.queries.iter().all(|query| query.images.is_err()));
}

fn query_at<'a>(rows: &'a RegionCopyContexts, path: &[usize]) -> &'a RegionContextCopyQuery {
    rows.queries
        .iter()
        .find(|row| row.source.child_slots == path)
        .unwrap()
}
