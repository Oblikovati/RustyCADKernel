// SPDX-License-Identifier: GPL-2.0-only
use super::super::{
    request_replay::encode_owned_prism_request, resolver::fixture::*, test_fixture,
};
use super::*;
#[test]
fn complete_envelope_binds_full_request_and_each_unchanged_child() {
    let (result, _) = super::operation_tests::accepted();
    assert_eq!(result.version(), 1);
    let bytes = result.replay_bytes();
    let header = b"Martensite.ScopedOwnedPrism/v1\0";
    assert!(bytes.starts_with(header));
    let mut rest = &bytes[header.len()..];
    for child in expected_children(&result) {
        let length = u64::from_le_bytes(rest[..8].try_into().unwrap()) as usize;
        assert_eq!(length, child.len());
        assert_eq!(&rest[8..8 + length], child);
        rest = &rest[8 + length..];
    }
    assert!(rest.is_empty());
    assert_eq!(result.replay_bytes(), bytes);
}
fn expected_children(result: &OwnedCompletePrism) -> Vec<Vec<u8>> {
    vec![
        encode_owned_prism_request(result.request()),
        result.publication().replay_bytes().to_vec(),
        result.names().replay_bytes().to_vec(),
        result.reservations().replay_bytes().to_vec(),
        result.body().replay_bytes(),
        result.volume().replay_bytes(),
    ]
}
#[test]
fn explicit_legacy_publication_policy_keeps_existing_actual_child_bytes() {
    let request = test_fixture::request();
    let legacy = resolved_prism_fixture(request.clone());
    let mut progress = super::operation_tests::CompleteObserver::default();
    let complete = with_owned_prism_operation(request, &mut progress, |result| {
        result.unwrap_or_else(|_| panic!("legacy publication remains supported"))
    });
    assert_eq!(complete.publication().requested().version, 1);
    assert_eq!(
        complete.publication().replay_bytes(),
        legacy.publication.replay_bytes()
    );
    assert_eq!(complete.names().replay_bytes(), legacy.names.replay_bytes());
}
