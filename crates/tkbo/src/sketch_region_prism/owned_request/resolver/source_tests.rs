// SPDX-License-Identifier: GPL-2.0-only
use super::*;
use crate::sketch_region::PartitionObservation;
use rustycad_tkg3d::top_abs::Orientation;
pub(super) fn operation(reversed: bool) -> PreparedPolynomialPrism {
    let mut request = super::super::test_fixture::request();
    request.sources[0].reversed_traversal = reversed;
    super::super::test_fixture::prepared(request)
}
pub(super) fn plan_for(prepared: &PreparedPolynomialPrism) -> SingleBoundaryNamePlan {
    let request = &prepared.retained.request;
    SingleBoundaryNamePlan::v2(
        request.sources[0].names(request.operation),
        request.selection,
    )
    .unwrap()
}
pub(super) fn initial_copy(prepared: &PreparedPolynomialPrism) -> RegionCopyContexts {
    let bridge = prepared.retained.bridge.as_ref().unwrap();
    let PartitionObservation::Captured(partition) = &bridge.mapped.arrangement.partition else {
        panic!("actual partition required");
    };
    let identity = prepared.retained.source.as_ref().unwrap().identity;
    let matching: Vec<_> = partition
        .input_contexts
        .iter()
        .filter(|row| row.source == identity)
        .collect();
    assert_eq!((matching.len(), partition.input_contexts.len()), (1, 1));
    retained_copy_contexts(&matching[0].observation, PrismHistoryStage::InputCopy)
        .unwrap_or_else(|_| panic!("actual complete copy observations required"))
        .clone()
}
pub(super) fn checked<T>(value: ResolveCheck<T>) -> T {
    value.unwrap_or_else(|_| panic!("expected uniquely resolved actual context"))
}
pub(super) fn original(
    prepared: &PreparedPolynomialPrism,
    copy: &RegionCopyContexts,
) -> Vec<NamedSourceContext> {
    checked(source::original_contexts(prepared, &copy.source))
}
#[test]
fn actual_original_and_copy_preserve_native_roles_under_traversal_reversal() {
    for reversed in [false, true] {
        let prepared = operation(reversed);
        checked(plan::admit_plan(&prepared, &plan_for(&prepared)));
        let copy = initial_copy(&prepared);
        let names = original(&prepared, &copy);
        assert_endpoint_roles(&names, reversed);
        let copied = checked(copied::copied_contexts(&copy, &names));
        assert_endpoint_roles(&copied, reversed);
        assert_eq!((names.len(), copied.len()), (3, 3));
    }
}
pub(super) fn assert_endpoint_roles(names: &[NamedSourceContext], reversed: bool) {
    let expected = if reversed {
        [Orientation::Reversed, Orientation::Forward]
    } else {
        [Orientation::Forward, Orientation::Reversed]
    };
    for (role, orientation) in [SemanticNameRole::StartUse, SemanticNameRole::EndUse]
        .into_iter()
        .zip(expected)
    {
        let found: Vec<_> = names.iter().filter(|r| r.directed_use == role).collect();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].member, SemanticNameRole::ClosureVertex);
        assert_eq!(found[0].actual.occurrence.orientation, orientation);
    }
}
#[test]
fn duplicate_or_missing_original_context_cannot_keep_semantic_coverage() {
    let prepared = operation(false);
    let copy = initial_copy(&prepared);
    let mut missing = copy.source.clone();
    missing.pop();
    assert!(source::original_contexts(&prepared, &missing).is_err());
    let mut duplicated = copy.source.clone();
    duplicated[1] = duplicated[2].clone();
    assert!(source::original_contexts(&prepared, &duplicated).is_err());
    let mut extra = copy.source.clone();
    extra.push(extra[0].clone());
    assert!(source::original_contexts(&prepared, &extra).is_err());
}
#[test]
fn native_endpoint_handle_swap_is_rejected_without_renaming_caller_uses() {
    let mut prepared = operation(false);
    let copy = initial_copy(&prepared);
    let source = prepared.retained.source.as_mut().unwrap();
    std::mem::swap(&mut source.native_start, &mut source.native_end);
    assert!(matches!(
        source::original_contexts(&prepared, &copy.source),
        Err(ResolutionCause::NativeParameter {
            actual: 1.,
            expected: 0.
        })
    ));
}
#[test]
fn explicit_v2_plan_refuses_legacy_and_cross_caller_confusion() {
    let prepared = operation(false);
    let legacy = prepared.retained.plan.as_ref().unwrap();
    assert!(matches!(
        plan::admit_plan(&prepared, legacy),
        Err(ResolutionCause::PlanVersion {
            actual: 1,
            expected: 2
        })
    ));
    let mut caller = plan_for(&prepared).identities();
    caller.operation.0 += 1;
    let wrong = SingleBoundaryNamePlan::v2(caller, prepared.retained.request.selection).unwrap();
    assert!(matches!(
        plan::admit_plan(&prepared, &wrong),
        Err(ResolutionCause::PlanMismatch { .. })
    ));
}
#[test]
fn copier_refuses_split_answers_duplicate_queries_and_lost_targets() {
    for mutation in 0..3 {
        let prepared = operation(false);
        let mut copy = initial_copy(&prepared);
        let names = original(&prepared, &copy);
        match mutation {
            0 => {
                let image = copy.queries[0].images.as_ref().unwrap()[0].clone();
                copy.queries[0].images = Ok(vec![image.clone(), image]);
            }
            1 => copy.queries[1] = copy.queries[0].clone(),
            _ => copy.copied[1] = copy.copied[2].clone(),
        }
        assert!(copied::copied_contexts(&copy, &names).is_err());
    }
}
#[test]
fn copy_domain_permutation_preserves_roles_and_never_uses_returned_rank() {
    let prepared = operation(false);
    let mut copy = initial_copy(&prepared);
    let names = original(&prepared, &copy);
    copy.copied.reverse();
    copy.queries.reverse();
    let mapped = checked(copied::copied_contexts(&copy, &names));
    assert_endpoint_roles(&mapped, false);
    assert_eq!(
        mapped
            .iter()
            .filter(|row| row.member == SemanticNameRole::Boundary)
            .count(),
        1
    );
}

#[test]
fn stored_pole_reversal_does_not_reassign_native_endpoint_names() {
    for reversed in [false, true] {
        let mut request = super::super::test_fixture::request();
        request.sources[0].world_poles.reverse();
        request.sources[0].reversed_traversal = reversed;
        let prepared = super::super::test_fixture::prepared(request);
        let copy = initial_copy(&prepared);
        let names = original(&prepared, &copy);
        assert_endpoint_roles(&names, reversed);
        assert_endpoint_roles(&checked(copied::copied_contexts(&copy, &names)), reversed);
    }
}
#[test]
fn isolated_plan_admission_compares_signed_zero_and_rejects_nan() {
    let mut prepared = operation(false);
    prepared.retained.request.selection =
        SemanticSelectionIntent::BoundedRegionContaining { seed: [-0., 1.] };
    let plan = plan_for(&prepared);
    prepared.retained.request.selection =
        SemanticSelectionIntent::BoundedRegionContaining { seed: [0., 1.] };
    assert!(matches!(
        plan::admit_plan(&prepared, &plan),
        Err(ResolutionCause::PlanMismatch { .. })
    ));
    prepared.retained.request.selection = SemanticSelectionIntent::BoundedRegionContaining {
        seed: [f64::NAN, 1.],
    };
    assert!(matches!(plan::admit_plan(&prepared, &plan),
        Err(ResolutionCause::Names(SemanticNameFailure::NonfiniteSeed { axis: 0, actual })) if actual.is_nan()));
}
#[test]
fn duplicate_named_use_cannot_replace_another_context_at_equal_count() {
    let prepared = operation(false);
    let copy = initial_copy(&prepared);
    let mut names = original(&prepared, &copy);
    names[1] = names[2].clone();
    assert!(complete_context_coverage(&names, &copy.source).is_err());
    assert!(copied::copied_contexts(&copy, &names).is_err());
}

#[test]
fn current_source_census_rejects_stale_child_paths_before_role_resolution() {
    let prepared = operation(false);
    let mut copy = initial_copy(&prepared);
    let vertex = copy
        .source
        .iter_mut()
        .find(|row| !row.child_slots.is_empty())
        .unwrap();
    vertex.child_slots[0] += 8;
    assert!(source::original_contexts(&prepared, &copy.source).is_err());
}
