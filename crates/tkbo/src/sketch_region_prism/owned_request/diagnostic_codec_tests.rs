// SPDX-License-Identifier: GPL-2.0-only
//! Full typed receipts are separate from the earlier context-plus-Debug development observation.
use super::{diagnostics::*, progress::*, test_fixture::*};
use crate::sketch_region_prism::diagnostic_envelope::*;
use rustycad_tkernel::standard::Failure;
#[test]
fn full_typed_refusal_matches_explicit_early_callback_fields() {
    let value = cancelled();
    let mut expected = b"Martensite.OwnedPrismDiagnostic/v1\0".to_vec();
    expected.extend((value.context().replay_bytes().len() as u64).to_le_bytes());
    expected.extend(value.context().replay_bytes());
    expected.extend([0, 0, 3, 0, 2, 1, 0]);
    expected.extend([0; 8]);
    expected.push(0);
    expected.extend([0; 8]);
    assert_eq!(value.replay_bytes(), expected);
    assert_ne!(value.replay_bytes(), value.context().replay_bytes());
}
#[test]
fn full_typed_refusal_process_receipt() {
    let bytes = corpus_receipt();
    let Some(path) = std::env::var_os("MARTENSITE_TYPED_DIAGNOSTIC_RECEIPT") else {
        return;
    };
    std::fs::write(path, bytes).unwrap();
}
#[test]
fn full_typed_refusal_is_identical_across_threads_and_lock_free_readers() {
    let expected = corpus_receipt();
    let workers: Vec<_> = (0..8).map(|_| std::thread::spawn(corpus_receipt)).collect();
    for worker in workers {
        assert_eq!(worker.join().unwrap(), expected);
    }
    let shared = std::sync::Arc::new(cancelled());
    let expected = shared.replay_bytes();
    std::thread::scope(|scope| {
        let readers: Vec<_> = (0..8)
            .map(|_| scope.spawn(|| shared.replay_bytes()))
            .collect();
        for reader in readers {
            assert_eq!(reader.join().unwrap(), expected);
        }
    });
}
#[test]
fn full_typed_refusal_changes_for_exact_native_payload_and_original_class() {
    let baseline = native(OwnedPreparationClass::Failed, native_cause()).replay_bytes();
    let changed = native(OwnedPreparationClass::Incomplete, native_cause()).replay_bytes();
    assert_ne!(baseline, changed);
    let mut changed = native_cause();
    changed.set_message("changed");
    assert_ne!(
        baseline,
        native(OwnedPreparationClass::Failed, changed).replay_bytes()
    );
    let changed = native_cause().with_exception_parent("Standard_DomainError");
    assert_ne!(
        baseline,
        native(OwnedPreparationClass::Failed, changed).replay_bytes()
    );
}
fn cancelled() -> OwnedPrismDiagnostic {
    let mut observer = RequestObserver {
        stop: Some(0),
        ..RequestObserver::default()
    };
    let original = super::prepare_owned_polynomial_prism(request(), &mut observer)
        .err()
        .unwrap();
    let (diagnostic, original) = project_owned_preparation_refusal(original).into_parts();
    assert!(original.is_none());
    *diagnostic
}
fn native(class: OwnedPreparationClass, cause: Failure) -> OwnedPrismDiagnostic {
    let original = Box::new(OwnedPrismPreparationFailure {
        class,
        cause: OwnedPreparationCause::Native(cause),
        callback: None,
        retained: PreparationRetained::new(request()),
    });
    let (diagnostic, original) = project_owned_preparation_refusal(original).into_parts();
    assert!(original.is_none());
    *diagnostic
}
fn native_cause() -> Failure {
    let value = Failure::with_type_bytes("Native", &[255, 17], &[254, 29]);
    value.with_exception_parent("Standard_Failure")
}

#[test]
fn full_typed_declined_admission_retains_explicit_complete_projection_fields() {
    let mut input = request();
    input.sources.clear();
    let original = super::prepare_owned_polynomial_prism(input, &mut RequestObserver::default())
        .err()
        .unwrap();
    let projection = project_owned_preparation_refusal(original);
    let value = projection.diagnostic();
    let prefix =
        b"Martensite.OwnedPrismDiagnostic/v1\0".len() + 8 + value.context().replay_bytes().len();
    assert_eq!(
        &value.replay_bytes()[prefix..],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    );
}
#[test]
fn full_typed_incomplete_projection_includes_outer_owner_coverage() {
    let mut observer = RequestObserver {
        fail: Some(2),
        ..RequestObserver::default()
    };
    let original = super::prepare_owned_polynomial_prism(request(), &mut observer)
        .err()
        .unwrap();
    let projection = project_owned_preparation_refusal(original);
    let value = projection.diagnostic();
    let prefix =
        b"Martensite.OwnedPrismDiagnostic/v1\0".len() + 8 + value.context().replay_bytes().len();
    let receipt = value.replay_bytes();
    assert_eq!(&receipt[prefix..prefix + 4], &[0, 2, 2, 1]);
    assert!(receipt.ends_with(&[1, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0]));
    assert!(projection.into_parts().1.is_some());
}

fn corpus_receipt() -> Vec<u8> {
    let mut bytes = Vec::new();
    for value in [
        cancelled(),
        native(OwnedPreparationClass::Failed, native_cause()),
        native(OwnedPreparationClass::Incomplete, empty_derived()),
    ] {
        let receipt = value.replay_bytes();
        bytes.extend((receipt.len() as u64).to_le_bytes());
        bytes.extend(receipt);
    }
    bytes
}
fn empty_derived() -> Failure {
    let mut value = rustycad_tkernel::standard::exceptions::out_of_memory_bytes(&[]);
    value.set_stack_string("");
    value
}
