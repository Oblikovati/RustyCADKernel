// SPDX-License-Identifier: GPL-2.0-only
use super::{tests::*, *};
use rustycad_tkbrep::topods::Builder;
use rustycad_tkernel::standard::Failure;

struct NamedLineageStop {
    phase: FreezePhase,
    failure: bool,
    visited: bool,
}
impl FreezeProgress for NamedLineageStop {
    fn checkpoint(&mut self, phase: FreezePhase) -> Result<bool, Failure> {
        if phase != self.phase {
            return Ok(false);
        }
        self.visited = true;
        if self.failure {
            return Err(Failure::new(
                "injected lineage failure; expected uninterrupted capture",
            ));
        }
        Ok(true)
    }
}
#[test]
fn success_failure_and_cancellation_preserve_inputs_and_allow_reuse() {
    let (_, receipt, publication, authored) = fixture();
    let before = inspected(run(&receipt, &publication, &authored)).replay_bytes();
    for phase in [
        FreezePhase::Admission,
        FreezePhase::Identity,
        FreezePhase::Geometry,
        FreezePhase::Incidence,
        FreezePhase::Evidence,
        FreezePhase::Commit,
    ] {
        for failure in [false, true] {
            assert_stopped(&receipt, &publication, &authored, phase, failure);
            assert_eq!(
                before,
                inspected(run(&receipt, &publication, &authored)).replay_bytes()
            );
        }
    }
}
fn assert_stopped(
    receipt: &PrismReceipt,
    publication: &PrismPublicationBindings,
    authored: &PrismCopyBindings,
    phase: FreezePhase,
    failure: bool,
) {
    let mut stop = NamedLineageStop {
        phase,
        failure,
        visited: false,
    };
    let result = inspect_prism_lineage(receipt, publication, authored, &mut stop);
    assert!(stop.visited);
    match result {
        PrismLineageOutcome::Cancelled { phase: actual, .. } => {
            assert!(!failure && actual == phase)
        }
        PrismLineageOutcome::Failed(value) => assert!(
            failure
                && matches!(&value.findings()[0].cause, CopyLineageCause::FreezeFailure(e) if e.cause.native_failure() && e.phase == phase)
        ),
        _ => panic!("expected typed interruption"),
    }
}
struct RemoveSelectedChild {
    parent: Shape,
    child: Shape,
    visited: bool,
}
impl FreezeProgress for RemoveSelectedChild {
    fn checkpoint(&mut self, phase: FreezePhase) -> Result<bool, Failure> {
        if phase == FreezePhase::Commit {
            self.parent.set_free(true)?;
            Builder.remove(&mut self.parent, &self.child)?;
            self.visited = true;
        }
        Ok(false)
    }
}
#[test]
fn final_callback_selected_topology_mutation_cannot_reuse_initial_lineage() {
    let (_, receipt, publication, authored) = fixture();
    let mut mutation = selected_removal(&receipt);
    let result = inspect_prism_lineage(&receipt, &publication, &authored, &mut mutation);
    assert!(mutation.visited);
    let PrismLineageOutcome::Incomplete(value) = result else {
        panic!("expected changed selected topology refusal");
    };
    assert!(matches!(
        value.findings()[0].cause,
        CopyLineageCause::Count {
            actual: 0,
            expected: 1,
            ..
        }
    ));
    assert!(value.projection().is_none());
}
#[test]
fn source_and_copy_mutation_after_capture_cannot_change_owned_result() {
    let (source, receipt, publication, authored) = fixture();
    let captured = inspected(run(&receipt, &publication, &authored));
    let before = captured.replay_bytes();
    change_source_pole(&source);
    remove_copied_wire(&receipt);
    assert_eq!(before, captured.replay_bytes());
    assert!(!captured.is_valid_body());
}

fn selected_removal(receipt: &PrismReceipt) -> RemoveSelectedChild {
    let selected = receipt.input.selected.as_ref().unwrap();
    let child = selected
        .relations
        .iter()
        .find(|r| r.output.shape_type().unwrap() == rustycad_tkg3d::top_abs::ShapeEnum::Wire)
        .unwrap()
        .output
        .clone();
    RemoveSelectedChild {
        parent: (*selected.face).clone(),
        child,
        visited: false,
    }
}

fn change_source_pole(source: &Shape) {
    let edge = rustycad_tkbrep::topods::edge(source).unwrap();
    rustycad_tkbrep::brep::BRepTool::curve_with_location(&edge)
        .unwrap()
        .curve
        .cast::<rustycad_tkg3d::geom::GeomBezierCurve>()
        .set_pole(2, rustycad_tkmath::gp::Point3::new(6., 0., 0.))
        .unwrap();
}
fn remove_copied_wire(receipt: &PrismReceipt) {
    let mut root = receipt.copied.as_ref().unwrap().base.clone();
    let wire = receipt
        .copied
        .as_ref()
        .unwrap()
        .relations
        .iter()
        .find(|r| r.output.shape_type().unwrap() == rustycad_tkg3d::top_abs::ShapeEnum::Wire)
        .unwrap()
        .output
        .clone();
    root.set_free(true).unwrap();
    Builder.remove(&mut root, &wire).unwrap();
}
