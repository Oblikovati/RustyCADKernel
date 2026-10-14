// SPDX-License-Identifier: GPL-2.0-only
use super::{tests::*, *};
use crate::sketch_region_prism::{
    exact_tolerance::StoredToleranceCap, frozen::annular_join::operation_tests::joined,
};
fn fresh() -> Vec<u8> {
    let join = joined();
    let geometry = conformance(&join);
    certify_annular_prism_tolerances(
        &join,
        &geometry,
        StoredToleranceCap::report(0.).unwrap(),
        &mut Continue,
    )
    .ok()
    .unwrap()
    .replay_bytes()
}
#[test]
fn annular_exact_tolerance_replays_across_processes_threads_and_owned_input_drop() {
    let expected = fresh();
    assert_eq!(fresh(), expected);
    std::thread::scope(|scope| {
        let workers: Vec<_> = (0..4).map(|_| scope.spawn(fresh)).collect();
        for worker in workers {
            assert_eq!(worker.join().unwrap(), expected);
        }
    });
    let hex: String = expected.iter().map(|byte| format!("{byte:02x}")).collect();
    println!("annular-exact-tolerance-v2:{hex}");
}
#[test]
fn cap_modes_and_signed_zero_keep_distinct_complete_replay() {
    let join = joined();
    let geometry = conformance(&join);
    let encode = |cap| {
        certify_annular_prism_tolerances(&join, &geometry, cap, &mut Continue)
            .ok()
            .unwrap()
            .replay_bytes()
    };
    let positive = encode(StoredToleranceCap::report(0.).unwrap());
    assert_ne!(positive, encode(StoredToleranceCap::report(-0.).unwrap()));
    assert_ne!(positive, encode(StoredToleranceCap::NoLimit));
    assert_ne!(
        encode(StoredToleranceCap::report(1.).unwrap()),
        encode(StoredToleranceCap::refuse(1.).unwrap())
    );
}
