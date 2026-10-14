// SPDX-License-Identifier: GPL-2.0-only
//! Callback mutations exercise the entry; cached Shape aliases cannot stand in for root incidence.
use super::{
    tests::{authored_bindings, authored_receipt},
    *,
};
use rustycad_tkbrep::topods::{Builder, Shape};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ContainmentEdit {
    Remove,
    Replace,
    Reorder,
}

pub(super) struct RemoveNamedChild {
    edit: ContainmentEdit,
    phase: FreezePhase,
    parent: Shape,
    child: Shape,
    mutated: bool,
}
impl FreezeProgress for RemoveNamedChild {
    fn checkpoint(&mut self, phase: FreezePhase) -> Result<bool, Failure> {
        if phase == self.phase {
            self.parent.set_free(true)?;
            Builder.remove(&mut self.parent, &self.child)?;
            match self.edit {
                ContainmentEdit::Remove => {}
                ContainmentEdit::Replace => {
                    Builder.add(&mut self.parent, &self.child.empty_copied()?)?
                }
                ContainmentEdit::Reorder => Builder.add(&mut self.parent, &self.child)?,
            }
            self.mutated = true;
        }
        Ok(false)
    }
}
pub(super) fn remove_child_progress(
    receipt: &super::super::PrismReceipt,
    kind: ShapeEnum,
    phase: FreezePhase,
    edit: ContainmentEdit,
) -> RemoveNamedChild {
    let contexts =
        super::super::contexts::body_contexts(&receipt.diagnostic.as_ref().unwrap().body)
            .ok()
            .unwrap();
    let parent = contexts
        .iter()
        .find(|c| {
            c.occurrence.shape_type().ok() == Some(kind)
                && (edit != ContainmentEdit::Reorder || c.occurrence.nb_children() > 1)
        })
        .unwrap();
    let child = first_child_context(&contexts, parent);
    RemoveNamedChild {
        edit,
        phase,
        parent: parent.occurrence.clone(),
        child: child.occurrence.clone(),
        mutated: false,
    }
}
fn first_child_context<'a>(
    contexts: &'a [super::super::PrismBodyContext],
    parent: &super::super::PrismBodyContext,
) -> &'a super::super::PrismBodyContext {
    contexts
        .iter()
        .find(|c| {
            c.child_slots.len() == parent.child_slots.len() + 1
                && c.child_slots.starts_with(&parent.child_slots)
        })
        .unwrap()
}

fn reject_callback_removal(kind: ShapeEnum, phase: FreezePhase, edit: ContainmentEdit) {
    let (source, receipt) = authored_receipt();
    let bindings = authored_bindings(&receipt, &source);
    let mut progress = remove_child_progress(&receipt, kind, phase, edit);
    let result = freeze_prism(&receipt, &bindings, &mut progress);
    assert!(progress.mutated, "must reach mutation checkpoint {phase:?}");
    assert!(
        matches!(
            result,
            PrismFreezeOutcome::Incomplete(PrismFreezeFailure {
                cause: PrismFreezeCause::Identity { .. } | PrismFreezeCause::StaleCapture { .. },
                ..
            })
        ),
        "root incidence mutation must fail identity/freshness, not an unrelated graph gate"
    );
}
#[test]
fn actual_root_children_are_rebound_before_graph_capture() {
    for kind in [ShapeEnum::Solid, ShapeEnum::Shell, ShapeEnum::Wire] {
        for edit in [ContainmentEdit::Remove, ContainmentEdit::Replace] {
            reject_callback_removal(kind, FreezePhase::Incidence, edit);
        }
    }
}
#[test]
fn actual_root_children_are_rebound_at_commit() {
    for kind in [ShapeEnum::Solid, ShapeEnum::Shell, ShapeEnum::Wire] {
        for edit in [ContainmentEdit::Remove, ContainmentEdit::Replace] {
            reject_callback_removal(kind, FreezePhase::Commit, edit);
        }
    }
}
#[test]
fn present_region_error_is_never_dropped_as_unbound_success() {
    use {ContainmentEdit::Remove, FreezePhase::Commit, ShapeEnum::Solid};
    let (source, mut receipt) = authored_receipt();
    let bindings = authored_bindings(&receipt, &source);
    reject_region_evidence(&mut receipt);
    let mut progress = remove_child_progress(&receipt, Solid, Commit, Remove);
    let result = freeze_prism(&receipt, &bindings, &mut progress);
    assert!(!progress.mutated);
    assert!(matches!(
        result,
        PrismFreezeOutcome::Incomplete(PrismFreezeFailure {
            cause: PrismFreezeCause::UnsupportedDiagnostic {
                actual: "present RegionCertificateInspection error",
                ..
            },
            ..
        })
    ));
}

fn reject_region_evidence(receipt: &mut super::super::PrismReceipt) {
    receipt.region = Some(
        super::super::region_certificate::RegionCertificateInspection {
            version: 1,
            decision: Err(
                super::super::region_certificate::RegionCertificateCause::UnavailableCandidates,
            ),
        },
    );
}

#[test]
fn actual_sibling_order_is_rebound_before_graph_and_at_commit() {
    for phase in [FreezePhase::Incidence, FreezePhase::Commit] {
        for kind in [ShapeEnum::Shell, ShapeEnum::Wire] {
            reject_callback_removal(kind, phase, ContainmentEdit::Reorder);
        }
    }
}
