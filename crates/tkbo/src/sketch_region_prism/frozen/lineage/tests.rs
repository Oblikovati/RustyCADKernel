// SPDX-License-Identifier: GPL-2.0-only
use super::super::tests::{NamedFreezeProgress, authored_bindings, authored_receipt};
use super::*;
use crate::sketch_region::PartitionOccurrenceIdentity;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

pub(super) fn fixture() -> (
    Shape,
    PrismReceipt,
    PrismPublicationBindings,
    PrismCopyBindings,
) {
    let (source, receipt) = authored_receipt();
    let publication = authored_bindings(&receipt, &source);
    let authored = authored_copy_bindings(&receipt);
    (source, receipt, publication, authored)
}
pub(in crate::sketch_region_prism::frozen) fn authored_copy_bindings(
    receipt: &PrismReceipt,
) -> PrismCopyBindings {
    let selected = receipt.input.selected.as_ref().unwrap();
    let selected_members = selected
        .relations
        .iter()
        .map(|row| PrismSourceBinding {
            actual: PartitionOccurrenceIdentity::capture(&row.output),
            member: semantic_member(&row.output),
        })
        .collect();
    PrismCopyBindings {
        selected_members,
        selected_uses: authored_uses(&selected.face),
        copied_uses: authored_uses(&receipt.copied.as_ref().unwrap().base),
    }
}

fn semantic_member(shape: &Shape) -> AuthoredPrismId {
    AuthoredPrismId(match shape.shape_type().unwrap() {
        ShapeEnum::Face => 1701,
        ShapeEnum::Wire => 1702,
        ShapeEnum::Edge => 1703,
        ShapeEnum::Vertex => 1704,
        actual => panic!("fixture member {actual:?}; expected authored cubic member"),
    })
}
fn authored_uses(root: &Shape) -> Vec<PrismDirectedBinding> {
    super::super::super::contexts::body_contexts(root)
        .ok()
        .unwrap()
        .into_iter()
        .map(|actual| {
            let role = authored_use_role(&actual);
            PrismDirectedBinding {
                actual,
                authored_use: role,
            }
        })
        .collect()
}
pub(super) fn inspected(outcome: PrismLineageOutcome) -> PrismLineageInspection {
    match outcome {
        PrismLineageOutcome::Inspected(value) => *value,
        PrismLineageOutcome::Incomplete(value) | PrismLineageOutcome::Failed(value) => {
            panic!("{:?}", value.findings())
        }
        PrismLineageOutcome::Cancelled { phase, .. } => panic!("unexpected cancellation {phase:?}"),
    }
}
#[test]
fn borrowed_lineage_views_preserve_explicit_operation_and_context_members() {
    let (_, receipt, publication, authored) = fixture();
    let result = inspected(run(&receipt, &publication, &authored));
    assert_eq!(result.operation(), publication.operation);
    assert_eq!(result.contexts().len(), 10);
    for context in result.contexts() {
        assert_eq!(context.key.0.last().unwrap().0, context.occurrence.member);
        assert_eq!(context.occurrence.member.operation, result.operation());
    }
}
#[test]
fn complete_copy_lineage_keeps_retirement_separate_and_body_incomplete() {
    let (_, receipt, publication, authored) = fixture();
    let result = inspected(run(&receipt, &publication, &authored));
    assert_eq!(
        result.coverage(),
        (CopyLineageCoverage::Complete, CopyLineageCoverage::Complete)
    );
    assert!(!result.members().is_empty());
    assert_eq!(result.directed().len(), 5); // One face, wire, edge, and two authored endpoint uses.
    assert!(!result.sweep().is_empty());
    assert_eq!(result.retired().len(), 1); // The copied wire retires; its original edge survives.
    assert!(result.projection().unwrap().history().deleted.is_empty());
    assert!(result.findings().is_empty());
    assert!(!result.is_valid_body());
    assert!(
        receipt
            .gaps
            .iter()
            .any(|g| matches!(g, super::super::super::PrismGap::ComposedDeletionCoverage))
    );
}
#[test]
fn incomplete_copy_observations_preserve_member_and_directed_coverage_distinction() {
    let (_, mut receipt, publication, authored) = fixture();
    receipt.copied.as_mut().unwrap().observations.pop();
    let PrismLineageOutcome::Incomplete(result) = run(&receipt, &publication, &authored) else {
        panic!("expected incomplete member history");
    };
    assert_eq!(
        result.coverage(),
        (
            CopyLineageCoverage::Incomplete,
            CopyLineageCoverage::Blocked
        )
    );
    assert!(matches!(
        result.findings()[0].cause,
        CopyLineageCause::Count {
            actual: 4,
            expected: 5,
            ..
        }
    ));
}
#[test]
fn missing_directed_binding_never_fabricates_use_correspondence() {
    let (_, receipt, publication, mut authored) = fixture();
    authored.selected_uses.pop();
    let PrismLineageOutcome::Incomplete(result) = run(&receipt, &publication, &authored) else {
        panic!("expected incomplete directed history");
    };
    assert_eq!(
        result.coverage(),
        (
            CopyLineageCoverage::Complete,
            CopyLineageCoverage::Incomplete
        )
    );
}
#[test]
fn owned_lineage_types_are_send_sync() {
    fn require<T: Send + Sync>() {}
    require::<PrismLineageInspection>();
    require::<PrismLineageOutcome>();
    require::<CopyLineageFinding>();
    require::<PrismCopyBindings>();
    require::<CopyMemberKey>();
    require::<CopyUseKey>();
    require::<CopyOccurrence>();
    require::<CopyMemberObservation>();
    require::<CopyContextObservation>();
    require::<CopyDirectedObservation>();
    require::<CopySweepObservation>();
    require::<CopyRetirementObservation>();
    require::<CopyLineageCause>();
    require::<CopyLineageLimitation>();
}

pub(super) fn run(
    receipt: &PrismReceipt,
    publication: &PrismPublicationBindings,
    authored: &PrismCopyBindings,
) -> PrismLineageOutcome {
    inspect_prism_lineage(receipt, publication, authored, &mut NamedFreezeProgress)
}

fn authored_use_role(actual: &PrismBodyContext) -> AuthoredPrismId {
    AuthoredPrismId(match actual.occurrence.shape_type().unwrap() {
        ShapeEnum::Face => 1710,
        ShapeEnum::Wire => 1711,
        ShapeEnum::Edge => 1712,
        ShapeEnum::Vertex if actual.occurrence.orientation() == Orientation::Forward => 1713,
        ShapeEnum::Vertex => 1714,
        kind => panic!("fixture use {kind:?}; expected authored cubic use"),
    })
}
