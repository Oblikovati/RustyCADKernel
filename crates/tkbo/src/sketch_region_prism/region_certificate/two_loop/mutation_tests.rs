// SPDX-License-Identifier: GPL-2.0-only
//! Current roots are rebound after the last external callback; cached captured children are insufficient.
use super::{failure::*, fixture::*, operation_tests::*, outcome::*, request::*};
use rustycad_tkbrep::topods::{Builder, Shape, ShapeIterator};
use rustycad_tkernel::{
    message::{AlertHandle, BaseAlert, Gravity, Report},
    standard::Failure,
};
use std::sync::Arc;
struct RemoveWire {
    stage: TwoLoopRegionStage,
    parent: Shape,
    child: Shape,
    visited: bool,
}
impl TwoLoopRegionProgress for RemoveWire {
    fn checkpoint(&mut self, stage: TwoLoopRegionStage) -> Result<bool, Failure> {
        if stage == self.stage && !self.visited {
            self.parent.set_free(true)?;
            Builder.remove(&mut self.parent, &self.child)?;
            self.visited = true;
        }
        Ok(false)
    }
}
#[test]
fn current_face_membership_changes_before_capture_and_at_commit_are_refused() {
    for stage in [TwoLoopRegionStage::Membership, TwoLoopRegionStage::Commit] {
        let fixture = TwoLoopFixture::nested();
        let batch = super::super::borrowed_batch(&fixture.bridge).unwrap();
        let face = batch
            .candidates
            .iter()
            .find(|c| c.wires.len() == 2)
            .unwrap()
            .face
            .clone();
        let child = ShapeIterator::new(&face, true, true)
            .value()
            .unwrap()
            .clone();
        let mut progress = RemoveWire {
            stage,
            parent: face.into(),
            child,
            visited: false,
        };
        assert!(matches!(
            inspect(&fixture, &mut progress),
            TwoLoopRegionOutcome::Incomplete(_)
        ));
        assert!(progress.visited);
    }
}
struct AddReport {
    report: Arc<Report>,
    visited: bool,
}
impl TwoLoopRegionProgress for AddReport {
    fn checkpoint(&mut self, stage: TwoLoopRegionStage) -> Result<bool, Failure> {
        if stage == TwoLoopRegionStage::Commit {
            let alert: AlertHandle = Arc::new(BaseAlert);
            self.report.add_alert(Gravity::Fail, &alert);
            self.visited = true;
        }
        Ok(false)
    }
}
#[test]
fn final_report_alert_cannot_be_discarded_or_promote_partial_geometry() {
    let fixture = TwoLoopFixture::nested();
    let report = super::super::borrowed_batch(&fixture.bridge)
        .unwrap()
        .mapped_report
        .clone();
    let mut progress = AddReport {
        report,
        visited: false,
    };
    let TwoLoopRegionOutcome::Incomplete(failure) = inspect(&fixture, &mut progress) else {
        panic!("required diagnostic; expected incomplete")
    };
    assert!(progress.visited);
    assert!(matches!(
        failure.cause,
        TwoLoopRegionCause::UnsupportedDiagnostic {
            stage: "required candidate report alert payload"
        }
    ));
    assert!(matches!(
        failure.local,
        Some(TwoLoopLocalDiagnostic::Report {
            gravity: Gravity::Fail,
            ..
        })
    ));
}
#[test]
fn source_mutation_preserves_owned_bytes_but_prevents_rebinding_with_same_names() {
    let fixture = TwoLoopFixture::nested();
    let proof = accepted(inspect(&fixture, &mut OperationProgress));
    let bytes = proof.replay.clone();
    crate::sketch_region_prism::isolation_tests::set_pole(
        &rustycad_tkbrep::topods::edge(&fixture.outer).unwrap(),
        5.,
    );
    let sources = fixture.sources();
    let identities = super::fixture_bindings::identities(&fixture);
    let request = TwoLoopRegionRequest {
        bridge: &fixture.bridge,
        sources: &sources,
        identities: &identities,
        inner_interior: [1., 1.],
        material_seed: [0.25, 0.25],
        requested: super::fixture_bindings::options(),
    };
    assert!(super::operation::rebind_two_loop_region(&request, &proof).is_err());
    assert_eq!(proof.replay, bytes);
}
