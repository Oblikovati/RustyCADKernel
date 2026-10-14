// SPDX-License-Identifier: GPL-2.0-only
use super::{joined_test_support::*, *};
fn fresh_receipt() -> Vec<u8> {
    let join = JoinFixture::new().compose(|| false).ok().unwrap();
    let geometry = conformance(&join);
    tolerance(&join, &geometry, StoredToleranceCap::report(0.).unwrap())
        .ok()
        .unwrap()
        .replay_bytes()
}
#[test]
fn exact_tolerance_bytes_replay_across_threads_and_operations() {
    let expected = fresh_receipt();
    for _ in 0..3 {
        assert_eq!(fresh_receipt(), expected);
    }
    std::thread::scope(|scope| {
        let workers: Vec<_> = (0..4).map(|_| scope.spawn(fresh_receipt)).collect();
        for worker in workers {
            assert_eq!(worker.join().unwrap(), expected);
        }
    });
    let hex: String = expected.iter().map(|byte| format!("{byte:02x}")).collect();
    println!("prism-exact-tolerance-v2:{hex}");
}
#[test]
fn immutable_certificate_types_are_send_sync() {
    fn send_sync<T: Send + Sync>() {}
    send_sync::<JoinedPrismConformance>();
    send_sync::<JoinedPrismTolerance>();
    send_sync::<JoinedConformanceFailure>();
    send_sync::<JoinedToleranceFailure>();
    send_sync::<JoinedConformanceCause>();
    send_sync::<JoinedToleranceCause>();
}
#[test]
fn replay_distinguishes_explicit_caps_signed_zero_and_option_tags() {
    let join = JoinFixture::new().compose(|| false).ok().unwrap();
    let geometry = conformance(&join);
    distinct_cap_receipts(&join, &geometry);
}
#[test]
fn independently_owned_proofs_survive_live_body_metadata_mutation() {
    let fixture = JoinFixture::new();
    let join = fixture.compose(|| false).ok().unwrap();
    let geometry = conformance(&join);
    let metadata = tolerance(&join, &geometry, StoredToleranceCap::NoLimit)
        .ok()
        .unwrap();
    let before = metadata.replay_bytes();
    mutate_body_metadata(&fixture);
    assert_eq!(metadata.replay_bytes(), before);
    assert!(fixture.rebind(&join).is_err());
    assert!(geometry.binds(&join));
}
#[test]
fn every_optional_limit_numeric_bit_and_presence_is_retained() {
    let join = JoinFixture::new().compose(|| false).ok().unwrap();
    let mut limits = *join.projection().limits();
    let encode = |value| {
        let mut bytes = Vec::new();
        super::replay_limits::limits(&value, &mut bytes);
        bytes
    };
    limits.achieved_error = Some(0.);
    let positive = encode(limits);
    limits.achieved_error = Some(-0.);
    assert_ne!(positive, encode(limits));
    limits.achieved_error = None;
    assert_ne!(positive, encode(limits));
}

fn mutate_body_metadata(fixture: &JoinFixture) {
    let vertex = fixture
        .receipt
        .diagnostic
        .as_ref()
        .unwrap()
        .bindings
        .iter()
        .find(|row| {
            row.context.occurrence.shape_type().ok()
                == Some(rustycad_tkg3d::top_abs::ShapeEnum::Vertex)
        })
        .unwrap();
    rustycad_tkbrep::brep::access::vertex_tshape(
        &vertex.context.occurrence,
        "exact tolerance isolation test",
    )
    .unwrap()
    .set_tolerance(0.125);
}

fn distinct_cap_receipts(join: &JoinedPrismProof, geometry: &JoinedPrismConformance) {
    let encode = |cap| tolerance(join, geometry, cap).ok().unwrap().replay_bytes();
    assert_ne!(
        encode(StoredToleranceCap::report(0.).unwrap()),
        encode(StoredToleranceCap::report(-0.).unwrap())
    );
    assert_ne!(
        encode(StoredToleranceCap::report(1.).unwrap()),
        encode(StoredToleranceCap::refuse(1.).unwrap())
    );
    assert_ne!(
        encode(StoredToleranceCap::NoLimit),
        encode(StoredToleranceCap::report(1.).unwrap())
    );
}
