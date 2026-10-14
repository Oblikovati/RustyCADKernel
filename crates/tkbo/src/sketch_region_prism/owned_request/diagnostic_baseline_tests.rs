// SPDX-License-Identifier: GPL-2.0-only
//! ADR0048 baseline: local class, input bits and callback payload precede projection.
use super::{diagnostics::*, pipeline::*, progress::*, test_fixture::*};
use rustycad_tkernel::standard::Failure;
struct NativeBytesObserver;
impl OwnedPrismProgress for NativeBytesObserver {
    fn checkpoint(&mut self, _: OwnedPrismCheckpoint) -> Result<bool, Failure> {
        let cause = expected_native_failure();
        Err(cause)
    }
}
#[test]
fn diagnostic_baseline_empty_request_retains_declined_class_without_plan() {
    let mut input = request();
    input.sources.clear();
    let failure = prepare_owned_polynomial_prism(input, &mut RequestObserver::default())
        .err()
        .unwrap();
    assert_eq!(failure.class, OwnedPreparationClass::Declined);
    assert_eq!(failure.retained.phase, OwnedPrismPhase::Admission);
    assert!(failure.retained.plan.is_none());
    assert!(failure.retained.request.sources.is_empty());
    assert_eq!(failure.retained.request.operation.0, 257);
}
#[test]
fn diagnostic_baseline_invalid_float_retains_payload_bits_and_original_revision() {
    let mut input = request();
    input.sources[0].world_poles[1][2] = f64::from_bits(0x7ff8_0000_0000_0123);
    input.policy.parameter_limit = -0.;
    let failure = prepare_owned_polynomial_prism(input, &mut RequestObserver::default())
        .err()
        .unwrap();
    assert_eq!(failure.class, OwnedPreparationClass::Failed);
    assert_eq!(failure.retained.request.sources[0].revision.0, 71);
    assert_input_bits(&failure);
    assert!(failure.retained.plan.is_none());
}
#[test]
fn diagnostic_baseline_callback_error_keeps_native_bytes_and_terminal_checkpoint() {
    let failure = prepare_owned_polynomial_prism(request(), &mut NativeBytesObserver)
        .err()
        .unwrap();
    assert_eq!(failure.class, OwnedPreparationClass::Failed);
    assert!(failure.retained.plan.is_some());
    assert!(failure.retained.source.is_none());
    assert_native_stop(failure.callback.unwrap());
}
#[test]
fn diagnostic_baseline_later_callback_retains_constructed_local_source() {
    let mut observer = RequestObserver {
        fail: Some(2),
        ..RequestObserver::default()
    };
    let failure = prepare_owned_polynomial_prism(request(), &mut observer)
        .err()
        .unwrap();
    assert_eq!(failure.class, OwnedPreparationClass::Failed);
    assert_eq!(failure.retained.phase, OwnedPrismPhase::SourceCapture);
    assert!(failure.retained.source.is_some());
    assert_eq!(failure.callback.unwrap().checkpoint.ordinal, 2);
}

fn assert_input_bits(failure: &OwnedPrismPreparationFailure) {
    assert_eq!(
        failure.retained.request.sources[0].world_poles[1][2].to_bits(),
        0x7ff8_0000_0000_0123
    );
    assert_eq!(
        failure.retained.request.policy.parameter_limit.to_bits(),
        (-0.0f64).to_bits()
    );
}

fn assert_native_stop(stop: OwnedPrismCallbackStop) {
    assert_eq!(
        stop.checkpoint,
        OwnedPrismCheckpoint {
            phase: OwnedPrismPhase::Admission,
            ordinal: 0
        }
    );
    let OwnedPrismCallbackCause::Failed(cause) = stop.cause else {
        panic!("expected native failure")
    };
    assert_eq!(cause.message_bytes(), &[255, 17]);
    assert_eq!(cause.stack_bytes(), &[254, 29]);
    assert_eq!(cause, expected_native_failure());
}

fn expected_native_failure() -> Failure {
    Failure::with_type_bytes("Standard_DomainError", &[255, 17], &[254, 29])
        .with_exception_parent("Standard_Failure")
}
