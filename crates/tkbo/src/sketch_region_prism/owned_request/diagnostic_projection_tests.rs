// SPDX-License-Identifier: GPL-2.0-only
//! ADR0048: projection completeness never promotes the original operation class.
use super::{diagnostics::*, pipeline::*, progress::*, test_fixture::*};
use crate::sketch_region_prism::diagnostic_envelope::*;
use rustycad_tkernel::standard::Failure;

#[test]
fn early_admission_class_and_context_are_preserved_without_a_name_plan() {
    for (version, empty, expected) in [
        (1, true, PrismDiagnosticClass::Declined),
        (99, false, PrismDiagnosticClass::Incomplete),
    ] {
        let mut input = request();
        input.version = version;
        if empty {
            input.sources.clear();
        }
        let projection = project(input, &mut RequestObserver::default());
        let value = projection.diagnostic();
        assert_eq!(value.version(), 1);
        assert_eq!(value.class(), expected);
        assert_eq!(value.coverage(), DiagnosticProjectionCoverage::Complete);
        assert!(value.context().plan().is_none());
        assert_eq!(value.context().request().version, version);
        assert!(projection.into_parts().1.is_none());
    }
}
#[test]
fn admission_scalar_payload_and_caller_revision_survive_without_normalization() {
    let mut input = request();
    input.sources[0].world_poles[1][2] = f64::from_bits(0x7ff8_0000_0000_0123);
    input.policy.parameter_limit = -0.;
    let projection = project(input, &mut RequestObserver::default());
    let value = projection.diagnostic();
    assert_eq!(value.class(), PrismDiagnosticClass::Failed);
    assert_nonfinite_cause(value.cause());
    assert_eq!(value.context().request().sources[0].revision.0, 71);
    assert_eq!(
        value.context().request().policy.parameter_limit.to_bits(),
        (-0.0f64).to_bits()
    );
}
#[test]
fn cancellation_is_complete_diagnostic_with_cancelled_operation() {
    let mut observer = RequestObserver {
        stop: Some(0),
        ..RequestObserver::default()
    };
    let projection = project(request(), &mut observer);
    let value = projection.diagnostic();
    assert_eq!(value.class(), PrismDiagnosticClass::Cancelled);
    assert_eq!(value.coverage(), DiagnosticProjectionCoverage::Complete);
    assert!(value.context().plan().is_some());
    assert_admission_checkpoint(value.callback().unwrap().checkpoint);
    assert_eq!(
        value.callback().unwrap().cause,
        OwnedDiagnosticCallbackCause::Cancelled
    );
}
struct NativeBytesObserver;
impl OwnedPrismProgress for NativeBytesObserver {
    fn checkpoint(&mut self, _: OwnedPrismCheckpoint) -> Result<bool, Failure> {
        let native = expected_native();
        Err(native)
    }
}
#[test]
fn callback_native_failure_retains_type_parent_message_and_stack_bytes() {
    let projection = project(request(), &mut NativeBytesObserver);
    let value = projection.diagnostic();
    assert_eq!(value.class(), PrismDiagnosticClass::Failed);
    assert_eq!(value.coverage(), DiagnosticProjectionCoverage::Complete);
    assert_eq!(
        value.callback().unwrap().cause,
        OwnedDiagnosticCallbackCause::Failed(expected_native())
    );
    assert_eq!(value.cause(), &OwnedPreparationDiagnosticCause::Callback);
}
#[test]
fn constructed_source_retains_original_box_and_reports_outer_owner_gap() {
    let mut observer = RequestObserver {
        fail: Some(2),
        ..RequestObserver::default()
    };
    let original = prepare_owned_polynomial_prism(request(), &mut observer)
        .err()
        .unwrap();
    let address = &*original as *const OwnedPrismPreparationFailure;
    let projection = project_owned_preparation_refusal(original);
    let value = projection.diagnostic();
    assert_eq!(value.class(), PrismDiagnosticClass::Failed);
    assert_source_gap(value);
    let (_, original) = projection.into_parts();
    let original = original.unwrap();
    assert_eq!(&*original as *const OwnedPrismPreparationFailure, address);
    assert!(original.retained.source.is_some());
}
#[test]
fn request_buffers_are_isolated_from_later_caller_mutation() {
    let mut input = request();
    let projection = project(input.clone(), &mut NativeBytesObserver);
    let before = projection.diagnostic().context().replay_bytes().to_vec();
    input.sources[0].world_poles[1][0] = 900.;
    input.sources[0].revision.0 = 555;
    input.sources.clear();
    assert_eq!(projection.diagnostic().context().replay_bytes(), before);
    assert_eq!(
        projection.diagnostic().context().request().sources[0]
            .revision
            .0,
        71
    );
}
fn project(
    input: super::OwnedPolynomialPrismRequest,
    observer: &mut impl OwnedPrismProgress,
) -> PreparationRefusalProjection {
    let original = prepare_owned_polynomial_prism(input, observer)
        .err()
        .unwrap();
    project_owned_preparation_refusal(original)
}
fn expected_native() -> Failure {
    let native = Failure::with_type_bytes("Standard_DomainError", &[255, 17], &[254, 29]);
    native.with_exception_parent("Standard_Failure")
}

fn assert_nonfinite_cause(cause: &OwnedPreparationDiagnosticCause) {
    let OwnedPreparationDiagnosticCause::Admission(
        super::OwnedPrismAdmissionCause::NonfinitePole {
            source,
            pole,
            axis,
            actual,
        },
    ) = cause
    else {
        panic!("expected exact nonfinite pole cause")
    };
    assert_eq!(
        (source.0, *pole, *axis, actual.to_bits()),
        (0, 1, 2, 0x7ff8_0000_0000_0123)
    );
}

fn assert_admission_checkpoint(checkpoint: OwnedPrismCheckpoint) {
    assert_eq!(
        checkpoint,
        OwnedPrismCheckpoint {
            phase: OwnedPrismPhase::Admission,
            ordinal: 0
        }
    );
}

fn assert_source_gap(value: &OwnedPrismDiagnostic) {
    assert_eq!(
        value.phase(),
        PrismDiagnosticPhase::PreparationBoundary(OwnedPrismPhase::SourceCapture)
    );
    assert_eq!(
        value.gaps(),
        &[PrismDiagnosticGap::OuterOwners(vec![
            super::RetainedPreparationStage::ConstructedSource
        ])]
    );
}
