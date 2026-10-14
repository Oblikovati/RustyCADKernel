// SPDX-License-Identifier: GPL-2.0-only
//! Callback mutations exercise the entry; cached Shape aliases cannot stand in for root incidence.
use super::{
    tests::{authored_bindings, authored_receipt},
    *,
};
use rustycad_tkbrep::topods::{Builder, Shape};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;

struct RemoveNamedChild {
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
            self.mutated = true;
        }
        Ok(false)
    }
}
fn remove_child_progress(
    receipt: &super::super::PrismReceipt,
    kind: ShapeEnum,
    phase: FreezePhase,
) -> RemoveNamedChild {
    let contexts =
        super::super::contexts::body_contexts(&receipt.diagnostic.as_ref().unwrap().body)
            .ok()
            .unwrap();
    let parent = contexts
        .iter()
        .find(|c| c.occurrence.shape_type().ok() == Some(kind))
        .unwrap();
    let child = contexts
        .iter()
        .find(|c| {
            c.child_slots.len() == parent.child_slots.len() + 1
                && c.child_slots.starts_with(&parent.child_slots)
        })
        .unwrap();
    RemoveNamedChild {
        phase,
        parent: parent.occurrence.clone(),
        child: child.occurrence.clone(),
        mutated: false,
    }
}
fn reject_callback_removal(kind: ShapeEnum, phase: FreezePhase) {
    let (source, receipt) = authored_receipt();
    let bindings = authored_bindings(&receipt, &source);
    let mut progress = remove_child_progress(&receipt, kind, phase);
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
        reject_callback_removal(kind, FreezePhase::Incidence);
    }
}
#[test]
fn actual_root_children_are_rebound_at_commit() {
    for kind in [ShapeEnum::Solid, ShapeEnum::Shell, ShapeEnum::Wire] {
        reject_callback_removal(kind, FreezePhase::Commit);
    }
}
#[test]
fn present_region_error_is_never_dropped_as_unbound_success() {
    let (source, mut receipt) = authored_receipt();
    let bindings = authored_bindings(&receipt, &source);
    receipt.region = Some(
        super::super::region_certificate::RegionCertificateInspection {
            version: 1,
            decision: Err(
                super::super::region_certificate::RegionCertificateCause::UnavailableCandidates,
            ),
        },
    );
    let mut progress = remove_child_progress(&receipt, ShapeEnum::Solid, FreezePhase::Commit);
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
