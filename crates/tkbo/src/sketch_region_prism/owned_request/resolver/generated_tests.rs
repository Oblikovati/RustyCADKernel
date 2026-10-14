// SPDX-License-Identifier: GPL-2.0-only
use super::{source_tests::*, *};
use crate::sketch_region_prism::region_certificate::current_binding::CurrentPrismBinding;
use rustycad_tkbrep::{
    brep::BRepTool,
    topods::{edge, vertex},
};
fn candidate_rows(prepared: &PreparedPolynomialPrism) -> Vec<NamedSourceContext> {
    let input = checked(operands::initial_copy(prepared));
    let original = checked(source::original_contexts(prepared, &input.source));
    let copied = checked(copied::copied_contexts(input, &original));
    let partition = checked(partition::partition_contexts(prepared, &original, &copied));
    checked(copied::copied_contexts(
        checked(operands::candidate_copy(prepared)),
        &partition,
    ))
}
fn stages(prepared: &PreparedPolynomialPrism) -> (CurrentPrismBinding, Vec<NamedSourceContext>) {
    let binding = checked(current::binding(prepared, &mut || false));
    (binding, candidate_rows(prepared))
}
fn assert_native_endpoints(
    prepared: &PreparedPolynomialPrism,
    binding: &CurrentPrismBinding,
    named: &[NamedSourceContext],
    stage: PrismHistoryStage,
) {
    let face = checked(generated::certified_face(binding, stage));
    let candidate = resolve_certified_candidate(checked(current::candidates(prepared)), face)
        .unwrap_or_else(|_| panic!("candidate"));
    let live = checked(current::live_contexts(prepared, &candidate.face, stage));
    for (role, expected) in [
        (SemanticNameRole::StartUse, 0.),
        (SemanticNameRole::EndUse, 1.),
    ] {
        let row = named.iter().find(|row| row.directed_use == role).unwrap();
        assert_eq!(native_parameter(&live, row), expected);
    }
}
#[test]
fn actual_region_roles_preserve_native_endpoints_across_pole_and_traversal_reversals() {
    for (poles, traversal) in [(false, false), (false, true), (true, false), (true, true)] {
        let mut request = super::super::test_fixture::request();
        request.sources[0].reversed_traversal = traversal;
        if poles {
            request.sources[0].world_poles.reverse();
        }
        let prepared = super::super::test_fixture::prepared(request);
        let (binding, copied) = stages(&prepared);
        assert_region_pair(&prepared, &binding, &copied);
    }
}
#[test]
fn fresh_binding_cancellation_has_no_previous_semantic_authority() {
    let prepared = operation(false);
    let _ = checked(current::binding(&prepared, &mut || false));
    assert!(matches!(
        current::binding(&prepared, &mut || true),
        Err(ResolutionCause::Binding(_))
    ));
}
#[test]
fn duplicated_candidate_copy_rows_cannot_rename_generated_endpoints() {
    let prepared = operation(false);
    let (binding, mut copied) = stages(&prepared);
    let first = copied
        .iter()
        .position(|r| r.directed_use == SemanticNameRole::StartUse)
        .unwrap();
    let last = copied
        .iter()
        .position(|r| r.directed_use == SemanticNameRole::EndUse)
        .unwrap();
    copied[first] = copied[last].clone();
    assert!(
        generated::generated_contexts(&prepared, &binding, &copied, PrismHistoryStage::Selected)
            .is_err()
    );
}
#[test]
fn candidate_return_order_cannot_assign_selected_or_complement_names() {
    let mut prepared = operation(false);
    let (binding, copied) = stages(&prepared);
    reverse_candidate_order(&mut prepared);
    for stage in [PrismHistoryStage::Selected, PrismHistoryStage::Complement] {
        let named = checked(generated::generated_contexts(
            &prepared, &binding, &copied, stage,
        ));
        assert_native_endpoints(&prepared, &binding, &named, stage);
    }
}

fn native_parameter(
    live: &[crate::sketch_region_prism::PrismBodyContext],
    row: &NamedSourceContext,
) -> f64 {
    let actual = live
        .iter()
        .find(|row_live| {
            snapshot_directed_history_context(row_live).ok().as_ref() == Some(&row.actual)
        })
        .unwrap();
    BRepTool::parameter(
        &vertex(&actual.occurrence).unwrap(),
        &edge(actual.ancestors.last().unwrap()).unwrap(),
    )
    .unwrap()
}

fn assert_region_pair(
    prepared: &PreparedPolynomialPrism,
    binding: &CurrentPrismBinding,
    copied: &[NamedSourceContext],
) {
    for stage in [PrismHistoryStage::Selected, PrismHistoryStage::Complement] {
        let named = checked(generated::generated_contexts(
            prepared, binding, copied, stage,
        ));
        assert_eq!(named.len(), 5);
        assert_native_endpoints(prepared, binding, &named, stage);
        if stage == PrismHistoryStage::Selected {
            let sweep = checked(selected_copy::sweep_base_contexts(
                prepared, binding, &named,
            ));
            assert_eq!(sweep.len(), 5);
        }
    }
}

fn candidate_batch_mut(
    prepared: &mut PreparedPolynomialPrism,
) -> &mut crate::sketch_region_candidates::CandidateBatch {
    let copy = prepared
        .retained
        .bridge
        .as_mut()
        .unwrap()
        .mapped
        .copied
        .as_mut()
        .unwrap();
    copy.completion
        .as_mut()
        .unwrap_or_else(|_| panic!("candidate completion"))
}
fn reverse_candidate_order(prepared: &mut PreparedPolynomialPrism) {
    candidate_batch_mut(prepared).candidates.reverse();
}
#[test]
fn duplicated_certified_face_and_wrong_semantic_stage_are_typed_refusals() {
    let mut prepared = operation(false);
    let (binding, copied) = stages(&prepared);
    assert!(generated::certified_face(&binding, PrismHistoryStage::Original).is_err());
    let batch = candidate_batch_mut(&mut prepared);
    let target = resolve_certified_candidate(batch, &binding.region.source_candidate.face)
        .ok()
        .unwrap()
        .face
        .clone();
    for candidate in &mut batch.candidates {
        candidate.face = target.clone();
    }
    assert!(
        generated::generated_contexts(&prepared, &binding, &copied, PrismHistoryStage::Selected)
            .is_err()
    );
}
#[test]
fn selected_copy_missing_or_duplicate_target_refuses_complete_role_coverage() {
    let prepared = operation(false);
    let (binding, copied) = stages(&prepared);
    let named = checked(generated::generated_contexts(
        &prepared,
        &binding,
        &copied,
        PrismHistoryStage::Selected,
    ));
    let source = selected_live(&prepared, &binding);
    let copy = selected_copy_operand(&prepared);
    let target = checked(current::live_contexts(
        &prepared,
        &copy.base,
        PrismHistoryStage::SweepBase,
    ));
    assert_copy_target_mutations(&named, &source, target, &copy.observations);
}
fn selected_live(
    prepared: &PreparedPolynomialPrism,
    binding: &CurrentPrismBinding,
) -> Vec<crate::sketch_region_prism::PrismBodyContext> {
    let candidate = resolve_certified_candidate(
        checked(current::candidates(prepared)),
        &binding.region.source_candidate.face,
    )
    .ok()
    .unwrap();
    checked(current::live_contexts(
        prepared,
        &candidate.face,
        PrismHistoryStage::Selected,
    ))
}
fn assert_copy_target_mutations(
    named: &[NamedSourceContext],
    source: &[crate::sketch_region_prism::PrismBodyContext],
    mut target: Vec<crate::sketch_region_prism::PrismBodyContext>,
    observations: &[crate::sketch_region_prism::PrismCopyObservation],
) {
    target.reverse();
    assert!(selected_copy::map_selected_copy(named, source, &target, observations).is_ok());
    target[1] = target[0].clone();
    assert!(selected_copy::map_selected_copy(named, source, &target, observations).is_err());
    target.pop();
    assert!(selected_copy::map_selected_copy(named, source, &target, observations).is_err());
}

fn selected_copy_operand(
    prepared: &PreparedPolynomialPrism,
) -> &crate::sketch_region_prism::PrismCopiedSelection {
    prepared
        .retained
        .prism
        .as_ref()
        .unwrap()
        .copied
        .as_ref()
        .unwrap()
}
