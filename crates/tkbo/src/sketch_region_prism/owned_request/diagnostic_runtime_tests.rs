// SPDX-License-Identifier: GPL-2.0-only
//! Immutable projections retain request/native buffers independently of operation-local owners.
use super::{diagnostics::*, pipeline::*, progress::*, test_fixture::*};
use crate::sketch_region_prism::diagnostic_envelope::*;
use rustycad_tkernel::standard::Failure;
use static_assertions::assert_impl_all;
assert_impl_all!(OwnedPrismDiagnostic: Send, Sync);
assert_impl_all!(OwnedPrismDiagnosticContext: Send, Sync);
assert_impl_all!(OwnedPreparationDiagnosticCause: Send, Sync);
assert_impl_all!(OwnedPreparationCallback: Send, Sync);
assert_impl_all!(OwnedDiagnosticCallbackCause: Send, Sync);
assert_impl_all!(PrismDiagnosticPhase: Send, Sync);
assert_impl_all!(PrismDiagnosticClass: Send, Sync);
assert_impl_all!(PrismDiagnosticGap: Send, Sync);
assert_impl_all!(DiagnosticProjectionCoverage: Send, Sync);
assert_impl_all!(UnprojectedPreparationPayload: Send, Sync);
struct ChildSweepFailure;
impl OwnedPrismProgress for ChildSweepFailure {
    fn checkpoint(&mut self, point: OwnedPrismCheckpoint) -> Result<bool, Failure> {
        if point.phase == OwnedPrismPhase::Prism(crate::sketch_region_prism::PrismPhase::Sweep) {
            return Err(Failure::with_type("Callback", "child sweep failure", ""));
        }
        Ok(false)
    }
}
#[test]
fn projection_keeps_outer_entry_and_actual_child_callback_separate() {
    use crate::sketch_region_prism::PrismPhase;
    let original = prepare_owned_polynomial_prism(request(), &mut ChildSweepFailure)
        .err()
        .unwrap();
    let projected = project_owned_preparation_refusal(original);
    assert_eq!(
        projected.diagnostic().phase(),
        PrismDiagnosticPhase::PreparationBoundary(OwnedPrismPhase::Prism(PrismPhase::Admission))
    );
    assert_eq!(
        projected.diagnostic().callback().unwrap().checkpoint.phase,
        OwnedPrismPhase::Prism(PrismPhase::Sweep)
    );
    assert_eq!(projected.diagnostic().class(), PrismDiagnosticClass::Failed);
    let expected_coverage = DiagnosticProjectionCoverage::Incomplete;
    assert_eq!(projected.diagnostic().coverage(), expected_coverage);
    assert!(projected.into_parts().1.is_some());
}
#[test]
fn native_projection_owns_stable_bytes_after_original_failure_mutation() {
    let mut original_native = Failure::with_type_bytes("Native", &[255, 1], &[254, 2]);
    let failure = Box::new(OwnedPrismPreparationFailure {
        class: OwnedPreparationClass::Failed,
        cause: OwnedPreparationCause::Native(original_native.clone()),
        callback: None,
        retained: PreparationRetained::new(request()),
    });
    let projected = project_owned_preparation_refusal(failure);
    original_native.set_message("changed");
    original_native.set_stack_string("changed stack");
    let OwnedPreparationDiagnosticCause::Native(native) = projected.diagnostic().cause() else {
        panic!("expected native payload")
    };
    assert_eq!(native.message_bytes(), &[255, 1]);
    assert_eq!(native.stack_bytes(), &[254, 2]);
}
#[test]
fn preparation_diagnostic_threads_replay_identical_context_and_typed_observations() {
    let expected = diagnostic_receipt(&early_cancelled());
    let threads: Vec<_> = (0..8)
        .map(|_| std::thread::spawn(|| diagnostic_receipt(&early_cancelled())))
        .collect();
    for worker in threads {
        assert_eq!(worker.join().unwrap(), expected);
    }
    let shared = std::sync::Arc::new(early_cancelled());
    std::thread::scope(|scope| {
        let readers: Vec<_> = (0..8)
            .map(|_| scope.spawn(|| diagnostic_receipt(&shared)))
            .collect();
        for reader in readers {
            assert_eq!(reader.join().unwrap(), expected);
        }
    });
}
#[test]
fn preparation_diagnostic_process_receipt() {
    let receipt = diagnostic_receipt(&early_cancelled());
    let Some(path) = std::env::var_os("MARTENSITE_PREPARATION_DIAGNOSTIC_RECEIPT") else {
        return;
    };
    std::fs::write(path, receipt).unwrap();
}
fn early_cancelled() -> OwnedPrismDiagnostic {
    let mut observer = RequestObserver {
        stop: Some(0),
        ..RequestObserver::default()
    };
    let original = prepare_owned_polynomial_prism(request(), &mut observer)
        .err()
        .unwrap();
    let (diagnostic, original) = project_owned_preparation_refusal(original).into_parts();
    assert!(original.is_none());
    *diagnostic
}
fn diagnostic_receipt(value: &OwnedPrismDiagnostic) -> Vec<u8> {
    // Debug is a test observation of typed owned causes, not a production serialization contract.
    let mut bytes = value.context().replay_bytes().to_vec();
    bytes.extend(
        format!(
            "{:?}{:?}{:?}{:?}{:?}",
            value.phase(),
            value.class(),
            value.cause(),
            value.callback(),
            value.gaps()
        )
        .as_bytes(),
    );
    bytes
}

#[test]
fn identical_geometry_with_different_caller_ids_retains_distinct_contexts() {
    let first = early_cancelled();
    let mut second = request();
    second.operation.0 = 9001;
    second.sources[0].source.0 = 44;
    let mut observer = RequestObserver {
        stop: Some(0),
        ..RequestObserver::default()
    };
    let original = prepare_owned_polynomial_prism(second, &mut observer)
        .err()
        .unwrap();
    let second = project_owned_preparation_refusal(original);
    assert_ne!(
        first.context().replay_bytes(),
        second.diagnostic().context().replay_bytes()
    );
    assert_second_caller(second.diagnostic().context());
}

fn assert_second_caller(context: &OwnedPrismDiagnosticContext) {
    assert_eq!(context.plan().unwrap().identities().operation.0, 9001);
    assert_eq!(context.request().sources[0].source.0, 44);
}
