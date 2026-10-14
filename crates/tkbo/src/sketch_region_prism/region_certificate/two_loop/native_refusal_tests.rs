// SPDX-License-Identifier: GPL-2.0-only
//! Actual native failure payloads may be borrowed, but their class must survive the wrapper.
use super::{failure::*, fixture::*, operation_tests::*, outcome::*};
use crate::sketch_region_candidates::CandidateCopyTreeObservation;
use rustycad_tkernel::standard::Failure;
use std::sync::Arc;

#[test]
fn native_copied_tree_refusal_preserves_class_cause_and_reusable_inputs() {
    let mut fixture = TwoLoopFixture::nested();
    let before = accepted(inspect(&fixture, &mut OperationProgress))
        .replay
        .clone();
    let tree = &mut fixture.bridge.mapped.copied.as_mut().unwrap().copied_tree;
    let original = std::mem::replace(
        tree,
        CandidateCopyTreeObservation::Incomplete {
            members: Vec::new(),
            cause: Arc::new(Failure::new("native copied-tree fixture failure")),
        },
    );
    check_native_copy_refusal(inspect(&fixture, &mut OperationProgress));
    fixture.bridge.mapped.copied.as_mut().unwrap().copied_tree = original;
    assert_eq!(
        accepted(inspect(&fixture, &mut OperationProgress)).replay,
        before
    );
}
fn check_native_copy_refusal(outcome: TwoLoopRegionOutcome<'_>) {
    let TwoLoopRegionOutcome::Failed(failure) = outcome else {
        panic!("native copy cause; expected Failed")
    };
    let Some(TwoLoopLocalDiagnostic::Copy(copy)) = failure.local else {
        panic!("expected exact copy receipt")
    };
    let CandidateCopyTreeObservation::Incomplete { cause, .. } = &copy.copied_tree else {
        panic!("expected original cause")
    };
    assert_eq!(cause.message(), "native copied-tree fixture failure");
}
#[test]
fn mapped_generated_universe_refusal_keeps_native_failure_class() {
    let mut progress = OperationProgress;
    let work = super::work::TwoLoopWork::new(77, super::fixture_bindings::options(), &mut progress);
    let mut failure = work.failure(TwoLoopRegionCause::UnsupportedDiagnostic {
        stage: "generated universe",
    });
    failure.local = Some(TwoLoopLocalDiagnostic::Region(Box::new(
        super::super::RegionCertificateCause::Mapped(Failure::new(
            "native generated member read failure",
        )),
    )));
    assert_eq!(failure.classification(), super::TwoLoopRefusalClass::Failed);
}
