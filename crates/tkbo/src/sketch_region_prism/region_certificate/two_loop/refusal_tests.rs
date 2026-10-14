// SPDX-License-Identifier: GPL-2.0-only
use super::{
    failure::*, failure_class::*, fixture::*, fixture_bindings, operation_tests::*, outcome::*,
    work::*,
};
use crate::sketch_region::{ArrangementCause, ArrangementCompletion, ArrangementPhase};
use crate::sketch_region_candidates::{CandidateCause, CandidatePhase};
use crate::sketch_region_geometry_capture::CapturedGeometryCause;
use crate::sketch_region_source_conformance::{ConstructionStage, SourceConformanceCause};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::predicates::roots::{RootIsolationError, metric_bounds::ResidualError};
use std::sync::Arc;

fn failure(cause: TwoLoopRegionCause) -> Box<TwoLoopRegionFailure<'static>> {
    let mut progress = OperationProgress;
    TwoLoopWork::new(77, fixture_bindings::options(), &mut progress).failure(cause)
}
#[test]
fn wrapped_native_and_exact_cancellation_keep_the_original_failure_class() {
    let native = CapturedGeometryCause::Native(Failure::new("named source failure"));
    let wrapped = failure(TwoLoopRegionCause::Geometry(Arc::new(native)));
    assert_eq!(classify(&wrapped), RefusalClass::Failed);
    let nested = super::super::super::surface_charts::PrismSurfaceCause::SourceSnapshot(Arc::new(
        super::super::super::surface_charts::PrismSurfaceCause::Exact(ResidualError::Roots(
            RootIsolationError::Cancelled,
        )),
    ));
    assert_eq!(
        classify(&failure(TwoLoopRegionCause::Capture(Arc::new(nested)))),
        RefusalClass::Cancelled
    );
    let exhausted = failure(TwoLoopRegionCause::Residual(ResidualError::Roots(
        RootIsolationError::SubdivisionLimit { limit: 0 },
    )));
    assert_eq!(classify(&exhausted), RefusalClass::Incomplete);
}
#[test]
fn borrowed_source_and_candidate_causes_retain_class_and_typed_payload() {
    let source = SourceConformanceCause::Exact(ResidualError::Roots(RootIsolationError::Cancelled));
    let mut refusal = failure(TwoLoopRegionCause::SourceRefusal {
        stage: ConstructionStage::CopiedPlane,
    });
    refusal.local = Some(TwoLoopLocalDiagnostic::Source(&source));
    assert_eq!(classify(&refusal), RefusalClass::Cancelled);
    let candidate = CandidateCause::Mapped(Failure::new("native candidate failure"));
    refusal.local = Some(TwoLoopLocalDiagnostic::Candidate(&candidate));
    assert_eq!(classify(&refusal), RefusalClass::Failed);
    let cancelled = CandidateCause::Cancelled(CandidatePhase::Capture);
    refusal.local = Some(TwoLoopLocalDiagnostic::Candidate(&cancelled));
    assert_eq!(classify(&refusal), RefusalClass::Cancelled);
}
#[test]
fn failed_and_cancelled_upstream_arrangements_cannot_hide_behind_existing_candidates() {
    let mut fixture = TwoLoopFixture::nested();
    fixture.bridge.mapped.arrangement.completion =
        ArrangementCompletion::Failed(ArrangementCause::Failure {
            phase: ArrangementPhase::History,
            cause: Failure::new("native history failure"),
        });
    let TwoLoopRegionOutcome::Failed(refusal) = inspect(&fixture, &mut OperationProgress) else {
        panic!("existing candidate after failed arrangement; expected Failed");
    };
    assert!(matches!(
        refusal.local,
        Some(TwoLoopLocalDiagnostic::Arrangement(
            ArrangementCompletion::Failed(_)
        ))
    ));
    drop(refusal);
    fixture.bridge.mapped.arrangement.completion =
        ArrangementCompletion::Cancelled(ArrangementPhase::Interference);
    assert!(matches!(
        inspect(&fixture, &mut OperationProgress),
        TwoLoopRegionOutcome::Cancelled(_)
    ));
}
#[test]
fn current_pair_capture_failure_preserves_owned_cause_and_stale_success_is_distinct() {
    use rustycad_tkmath::predicates::roots::bezier_pair::BezierPairCaptureFailure;
    let mut fixture = TwoLoopFixture::nested();
    fixture.bridge.census.paired.as_mut().unwrap().reports[0].outcome =
        Err(Box::new(BezierPairCaptureFailure::EmptyControls));
    let TwoLoopRegionOutcome::Incomplete(refusal) = inspect(&fixture, &mut OperationProgress)
    else {
        panic!("capture failure; expected incomplete");
    };
    assert!(matches!(refusal.cause, TwoLoopRegionCause::PairCapture(_)));
    let mut fixture = TwoLoopFixture::nested();
    fixture.bridge.census.paired.as_mut().unwrap().reports[0]
        .effective
        .requested_world_error = 0.25;
    let TwoLoopRegionOutcome::Incomplete(refusal) = inspect(&fixture, &mut OperationProgress)
    else {
        panic!("stale complete report; expected incomplete");
    };
    assert!(matches!(refusal.cause, TwoLoopRegionCause::Stale { .. }));
    assert!(matches!(
        refusal.local,
        Some(TwoLoopLocalDiagnostic::Pair(_))
    ));
}
#[test]
fn every_actual_candidate_retains_signed_domain_and_material_conclusions() {
    let fixture = TwoLoopFixture::nested();
    let proof = accepted(inspect(&fixture, &mut OperationProgress));
    let actual: Vec<_> = proof
        .candidates()
        .iter()
        .map(|c| (c.name, c.signed_signature, c.material_winding))
        .collect();
    assert_eq!(
        actual,
        [(301, [1, -1], 1), (302, [0, 1], 0), (303, [-1, 0], -1)]
    );
}
