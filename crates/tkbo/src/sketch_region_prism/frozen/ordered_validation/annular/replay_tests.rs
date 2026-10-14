// SPDX-License-Identifier: GPL-2.0-only
use super::{tests::*, *};
use crate::sketch_region_prism::frozen::annular_join::operation_tests::joined;
#[test]
fn ordered_annular_replay_probe() {
    let bytes = complete(&joined()).replay_bytes();
    println!("ANNULAR_ORDERED_REPLAY={bytes:?}");
}
#[test]
fn complete_child_blocks_and_tolerance_policy_survive_full_replay() {
    let join = joined();
    let proof = complete(&join);
    let replay = proof.replay_bytes();
    assert_child_blocks(&proof, &replay);
    let requested = AnnularValidationOptions {
        version: 2,
        stored_cap: StoredToleranceCap::report(1.).unwrap(),
    };
    let other = validate_annular_prism(&join, requested, &mut RecordedAnnularProgress::default())
        .ok()
        .unwrap();
    assert_ne!(other.replay_bytes(), replay);
}
fn assert_child_blocks(proof: &AnnularValidationCertificate, replay: &[u8]) {
    for child in [
        proof.orientation().replay_bytes(),
        proof.conformance().replay_bytes(),
        proof.intersection().replay_bytes(),
        proof.tolerance().replay_bytes(),
    ] {
        assert!(replay.windows(child.len()).any(|slice| slice == child));
    }
}
#[test]
fn results_outlive_source_and_concurrent_operations_match_serial_bytes() {
    let report = complete(&joined());
    let expected = report.replay_bytes();
    let workers: Vec<_> = (0..4)
        .map(|_| std::thread::spawn(|| complete(&joined()).replay_bytes()))
        .collect();
    for worker in workers {
        assert_eq!(worker.join().unwrap(), expected);
    }
    assert_shared_reads(&report, &expected);
}
fn assert_shared_reads(report: &AnnularValidationCertificate, expected: &[u8]) {
    std::thread::scope(|scope| {
        for _ in 0..8 {
            scope.spawn(move || {
                for _ in 0..4 {
                    assert_eq!(report.replay_bytes(), *expected);
                }
            });
        }
    });
}
