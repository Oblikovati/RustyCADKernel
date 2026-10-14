// SPDX-License-Identifier: GPL-2.0-only
//! Independent little-endian fixture and field perturbations prevent partial request serialization.
use super::super::test_fixture::request;
use super::*;
#[test]
fn complete_request_matches_independent_explicit_field_fixture() {
    let bytes = encode_owned_prism_request(&request());
    assert_eq!(bytes, include_bytes!("fixtures/request-v1.bin"));
    assert!(!bytes.is_empty());
}
pub(super) fn changed(edit: impl FnOnce(&mut OwnedPolynomialPrismRequest)) {
    let original = request();
    let mut altered = original.clone();
    edit(&mut altered);
    assert_ne!(
        encode_owned_prism_request(&altered),
        encode_owned_prism_request(&original)
    );
}
#[test]
fn caller_identity_revision_multiplicity_and_pole_bits_are_all_bound() {
    let edits: [fn(&mut OwnedPolynomialPrismRequest); 10] = [
        |r| r.operation.0 += 1,
        |r| r.sources[0].source.0 += 1,
        |r| r.sources[0].revision.0 += 1,
        |r| r.sources[0].closure.0 += 1,
        |r| r.sources[0].native_start.0 += 1,
        |r| r.sources[0].native_end.0 += 1,
        |r| r.sources[0].reversed_traversal = true,
        |r| r.sources[0].world_poles[1][0] = -0.,
        |r| r.sources.push(r.sources[0].clone()),
        |r| r.sources.clear(),
    ];
    for edit in edits {
        changed(edit);
    }
}
#[test]
fn refused_nan_payload_and_signed_zero_are_not_normalized() {
    let mut first = request();
    let mut second = first.clone();
    first.sources[0].world_poles[1][0] = f64::from_bits(0x7ff8_0000_0000_0001);
    second.sources[0].world_poles[1][0] = f64::from_bits(0x7ff8_0000_0000_0002);
    assert_ne!(
        encode_owned_prism_request(&first),
        encode_owned_prism_request(&second)
    );
    first.sources[0].world_poles[1][0] = 0.;
    second.sources[0].world_poles[1][0] = -0.;
    assert_ne!(
        encode_owned_prism_request(&first),
        encode_owned_prism_request(&second)
    );
}
#[test]
fn independent_threads_keep_complete_request_bytes() {
    let expected = encode_owned_prism_request(&request());
    let workers: Vec<_> = (0..8)
        .map(|_| std::thread::spawn(|| encode_owned_prism_request(&request())))
        .collect();
    for worker in workers {
        assert_eq!(worker.join().unwrap(), expected);
    }
}
