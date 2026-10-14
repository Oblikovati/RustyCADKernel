// SPDX-License-Identifier: GPL-2.0-only
use super::{tests::*, *};
use rustycad_tkbrep::topods::Builder;

fn incomplete(
    receipt: &PrismReceipt,
    publication: &PrismPublicationBindings,
    authored: &PrismCopyBindings,
) -> PrismLineageInspection {
    match run(receipt, publication, authored) {
        PrismLineageOutcome::Incomplete(value) => *value,
        _ => panic!("expected typed incomplete lineage"),
    }
}
#[test]
fn duplicate_member_observation_is_not_extra_history_coverage() {
    let (_, mut receipt, publication, authored) = fixture();
    let observations = &mut receipt.copied.as_mut().unwrap().observations;
    observations[1] = observations[0].clone();
    let result = incomplete(&receipt, &publication, &authored);
    assert!(matches!(
        result.findings()[0].cause,
        CopyLineageCause::Count {
            actual: 2,
            expected: 1,
            ..
        }
    ));
    assert_eq!(result.coverage().0, CopyLineageCoverage::Incomplete);
}
#[test]
fn wrong_operation_copy_history_cannot_bind_the_current_body() {
    let (_, mut receipt, publication, authored) = fixture();
    let (_, other, _, _) = fixture();
    receipt.copied.as_mut().unwrap().observations = other.copied.unwrap().observations;
    let result = incomplete(&receipt, &publication, &authored);
    assert!(matches!(
        result.findings()[0].cause,
        CopyLineageCause::Count {
            actual: 0,
            expected: 1,
            ..
        }
    ));
}
#[test]
fn original_parent_multiplicity_and_class_cannot_change_silently() {
    let (_, mut receipt, publication, authored) = fixture();
    let observed = &mut receipt.copied.as_mut().unwrap().observations[0];
    let parent = observed
        .selected
        .generated
        .first()
        .or_else(|| observed.selected.modified.first())
        .unwrap()
        .clone();
    observed.selected.generated.push(parent);
    let result = incomplete(&receipt, &publication, &authored);
    assert!(matches!(
        result.findings()[0].cause,
        CopyLineageCause::Parents { .. }
    ));
    assert!(result.findings()[0].entity.is_some());
}
#[test]
fn same_parent_sets_keep_distinct_face_and_wire_intermediates() {
    let (_, receipt, publication, authored) = fixture();
    let result = inspected(run(&receipt, &publication, &authored));
    let face = result
        .members()
        .iter()
        .find(|r| r.selected.member.member == AuthoredPrismId(1701))
        .unwrap();
    let wire = result
        .members()
        .iter()
        .find(|r| r.selected.member.member == AuthoredPrismId(1702))
        .unwrap();
    assert_eq!(face.selected_parents, wire.selected_parents);
    assert_ne!(face.selected.member, wire.selected.member);
    assert_ne!(face.copied.member, wire.copied.member);
}
#[test]
fn colliding_authored_selected_members_are_incomplete() {
    let (_, receipt, publication, mut authored) = fixture();
    for binding in &mut authored.selected_members {
        binding.member = AuthoredPrismId(1701);
    }
    let result = incomplete(&receipt, &publication, &authored);
    assert!(matches!(
        result.findings()[0].cause,
        CopyLineageCause::Count { .. }
    ));
}
#[test]
fn authored_labels_cannot_disambiguate_identical_repeated_copy_uses() {
    let (_, receipt, publication, mut authored) = fixture();
    duplicate_selected_and_copy_edge(&receipt);
    rebind_added_uses(&receipt, &mut authored);
    let result = incomplete(&receipt, &publication, &authored);
    assert_eq!(
        result.coverage(),
        (
            CopyLineageCoverage::Complete,
            CopyLineageCoverage::Incomplete
        )
    );
    assert!(matches!(
        result.findings()[0].cause,
        CopyLineageCause::Count {
            actual: 2,
            expected: 1,
            ..
        }
    ));
}
fn duplicate_selected_and_copy_edge(receipt: &PrismReceipt) {
    for root in [
        &*receipt.input.selected.as_ref().unwrap().face,
        &receipt.copied.as_ref().unwrap().base,
    ] {
        let contexts = super::super::super::contexts::body_contexts(root)
            .ok()
            .unwrap();
        let mut wire = member_kind(&contexts, rustycad_tkg3d::top_abs::ShapeEnum::Wire);
        let edge = member_kind(&contexts, rustycad_tkg3d::top_abs::ShapeEnum::Edge);
        wire.set_free(true).unwrap();
        Builder.add(&mut wire, &edge).unwrap();
    }
}
fn repeated_use_names(
    root: &Shape,
    original: &[PrismDirectedBinding],
) -> Vec<PrismDirectedBinding> {
    let contexts = super::super::super::contexts::body_contexts(root)
        .ok()
        .unwrap();
    contexts
        .into_iter()
        .map(|actual| {
            let authored_use = original
                .iter()
                .find(|b| super::super::identities::same_context(&b.actual, &actual))
                .map(|b| b.authored_use)
                .unwrap_or_else(|| added_use_role(&actual));
            PrismDirectedBinding {
                actual,
                authored_use,
            }
        })
        .collect()
}
fn added_use_role(actual: &PrismBodyContext) -> AuthoredPrismId {
    use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
    // Explicit newly added edge/start/end roles name the adversarial extra use, never prove it.
    AuthoredPrismId(match actual.occurrence.shape_type().unwrap() {
        ShapeEnum::Edge => 1801,
        ShapeEnum::Vertex if actual.occurrence.orientation() == Orientation::Forward => 1802,
        ShapeEnum::Vertex => 1803,
        kind => panic!("added authored role {kind:?}; expected duplicate edge or endpoint"),
    })
}

#[test]
fn retirement_parent_classes_cannot_be_rewritten_as_original_deletion() {
    let (_, mut receipt, publication, authored) = fixture();
    let row = &mut receipt.diagnostic.as_mut().unwrap().native_retired[0];
    row.preserved.append(&mut row.modified);
    row.preserved.append(&mut row.generated);
    let result = incomplete(&receipt, &publication, &authored);
    assert!(matches!(
        result.findings()[0].cause,
        CopyLineageCause::Parents { .. }
    ));
    assert!(result.findings()[0].entity.is_some());
    assert!(receipt.input.input_deleted.is_empty());
}
#[test]
fn altered_native_sweep_parent_class_is_typed_incomplete() {
    let (_, mut receipt, publication, authored) = fixture();
    let row = &mut receipt.diagnostic.as_mut().unwrap().bindings[0]
        .native
        .returned;
    row.preserved.append(&mut row.modified);
    row.preserved.append(&mut row.generated);
    let result = incomplete(&receipt, &publication, &authored);
    assert!(matches!(
        result.findings()[0].cause,
        CopyLineageCause::Parents { .. }
    ));
    assert!(result.findings()[0].entity.is_some());
}

fn rebind_added_uses(receipt: &PrismReceipt, authored: &mut PrismCopyBindings) {
    authored.selected_uses = repeated_use_names(
        &receipt.input.selected.as_ref().unwrap().face,
        &authored.selected_uses,
    );
    authored.copied_uses = repeated_use_names(
        &receipt.copied.as_ref().unwrap().base,
        &authored.copied_uses,
    );
}

fn member_kind(contexts: &[PrismBodyContext], kind: rustycad_tkg3d::top_abs::ShapeEnum) -> Shape {
    contexts
        .iter()
        .find(|c| c.occurrence.shape_type().unwrap() == kind)
        .unwrap()
        .occurrence
        .clone()
}

#[test]
fn duplicate_authored_member_lookup_is_typed_incomplete() {
    let (_, receipt, publication, mut authored) = fixture();
    let first = &authored.selected_members[0];
    authored.selected_members.push(PrismSourceBinding {
        actual: first.actual.clone(),
        member: first.member,
    });
    let result = incomplete(&receipt, &publication, &authored);
    assert!(matches!(
        result.findings()[0].cause,
        CopyLineageCause::Count {
            actual: 2,
            expected: 1,
            ..
        }
    ));
}
#[test]
fn duplicate_retirement_observation_never_adds_deletion_credit() {
    let (_, mut receipt, publication, authored) = fixture();
    let diagnostic = receipt.diagnostic.as_mut().unwrap();
    diagnostic
        .native_retired
        .push(diagnostic.native_retired[0].clone());
    let result = incomplete(&receipt, &publication, &authored);
    assert!(matches!(
        result.findings()[0].cause,
        CopyLineageCause::Count {
            actual: 2,
            expected: 1,
            ..
        }
    ));
    assert!(receipt.input.input_deleted.is_empty());
}

#[test]
fn unsupported_source_failure_is_retained_before_missing_copy_fields() {
    let (_, mut receipt, publication, authored) = fixture();
    receipt.completion =
        super::super::super::PrismCompletion::Failed(super::super::super::PrismCause::Mapped {
            phase: super::super::super::PrismPhase::Copy,
            failure: rustycad_tkernel::standard::Failure::new(
                "test copy failure; expected complete copy",
            ),
        });
    receipt.copied = None;
    let result = incomplete(&receipt, &publication, &authored);
    assert!(
        matches!(&result.findings()[0].cause, CopyLineageCause::Frozen(cause)
        if matches!(**cause, PrismFreezeCause::UnsupportedDiagnostic { actual: "non-incomplete source completion carries unprojected cause", .. }))
    );
}
#[test]
fn count_limit_and_usize_max_refuse_without_allocating_large_inputs() {
    let (_, receipt, publication, authored) = fixture();
    let inputs = LineageInputs {
        receipt: &receipt,
        publication: &publication,
        authored: &authored,
    };
    assert!(inputs.limit("selected members", 4096).is_ok());
    for actual in [4097, usize::MAX] {
        let failure = inputs.limit("selected members", actual).unwrap_err();
        assert!(matches!(&failure.cause, CopyLineageCause::Frozen(cause)
            if matches!(**cause, PrismFreezeCause::LimitExceeded { quantity: "selected members", actual: count, maximum: 4096 } if count == actual)));
    }
}
