// SPDX-License-Identifier: GPL-2.0-only
//! Full actual invocation receipts, separate from context-only encoding and helper fixtures.
use super::*;
#[test]
fn unsupported_version_matches_independent_literal_full_envelope() {
    let value = project_request(request(17));
    let mut expected = b"Martensite.OwnedPublicationDiagnostic/v1\0".to_vec();
    expected.extend((value.context().replay_bytes().len() as u64).to_le_bytes());
    expected.extend(value.context().replay_bytes());
    expected.extend(257u64.to_le_bytes());
    expected.extend([0, 1, 0, 0]);
    expected.extend(17u32.to_le_bytes());
    expected.extend(1u32.to_le_bytes());
    expected.extend(0u64.to_le_bytes());
    assert_eq!(value.replay_bytes(), expected);
    assert_ne!(value.replay_bytes(), value.context().replay_bytes());
    assert_eq!(value.version(), 1);
}
fn cancelled() -> Box<OwnedPrismPublicationDiagnostic> {
    let mut observer = NamedPublicationObserver {
        failure: None,
        cancelled: true,
    };
    with_refusal(request(2), &mut observer, |packet| {
        let result = project_invoked_publication_refusal(packet);
        assert_eq!(
            result.diagnostic().class(),
            PrismPublicationClass::Cancelled
        );
        let (diagnostic, original) = result.into_parts();
        assert!(original.is_none());
        diagnostic
    })
}
fn native(failure: Failure) -> Box<OwnedPrismPublicationDiagnostic> {
    let mut observer = NamedPublicationObserver {
        failure: Some(failure),
        cancelled: false,
    };
    with_refusal(request(2), &mut observer, |packet| {
        project_invoked_publication_refusal(packet).into_parts().0
    })
}
fn receipt() -> Vec<u8> {
    let mut bytes = Vec::new();
    let values = [
        project_request(request(1)),
        project_request(request(2)),
        project_request(request(17)),
        cancelled(),
        native(Failure::out_of_range("native exact")),
        native(Failure::out_of_range("native exact").with_exception_parent("")),
    ];
    for value in values {
        let value = value.replay_bytes();
        bytes.extend((value.len() as u64).to_le_bytes());
        bytes.extend(value);
    }
    bytes
}
#[test]
fn full_publication_diagnostic_process_receipt() {
    let bytes = receipt();
    let Some(path) = std::env::var_os("MARTENSITE_PUBLICATION_DIAGNOSTIC_RECEIPT") else {
        return;
    };
    std::fs::write(path, bytes).unwrap();
}
#[test]
fn full_actual_receipts_repeat_across_process_local_work_and_threads() {
    let expected = receipt();
    assert_eq!(receipt(), expected);
    let workers: Vec<_> = (0..4).map(|_| std::thread::spawn(receipt)).collect();
    for worker in workers {
        assert_eq!(worker.join().unwrap(), expected);
    }
}
#[test]
fn immutable_report_readers_need_no_lock_or_operation_owner() {
    let report = std::sync::Arc::new(cancelled());
    let expected = report.replay_bytes();
    std::thread::scope(|scope| {
        let readers: Vec<_> = (0..8)
            .map(|_| scope.spawn(|| report.replay_bytes()))
            .collect();
        for reader in readers {
            assert_eq!(reader.join().unwrap(), expected);
        }
    });
}
#[test]
fn native_exact_parent_absence_empty_and_base_message_remain_distinct() {
    let first = native(Failure::out_of_range("native exact")).replay_bytes();
    let parent = native(Failure::out_of_range("native exact").with_exception_parent(""));
    let message = native(Failure::out_of_range("native changed"));
    assert_ne!(first, parent.replay_bytes());
    assert_ne!(first, message.replay_bytes());
}
#[test]
fn caller_revision_and_signed_zero_request_bits_remain_bound() {
    let mut original = request(2);
    let first = project_request(original.clone()).replay_bytes();
    original.sources[0].revision.0 += 1;
    let revision = project_request(original.clone()).replay_bytes();
    original.sources[0].world_poles[0][0] = -0.;
    let sign = project_request(original).replay_bytes();
    assert_ne!(first, revision);
    assert_ne!(revision, sign);
}
