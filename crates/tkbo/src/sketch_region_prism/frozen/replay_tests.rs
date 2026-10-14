// SPDX-License-Identifier: GPL-2.0-only
//! Process harness runs this same owned-byte test twice; workers build separate local sources.
use super::tests::fresh_projection;

#[test]
fn frozen_prism_owned_bytes_replay_across_threads_and_repeated_operations() {
    let serial = repeated_bytes();
    std::thread::scope(|scope| {
        let workers: Vec<_> = (0..8).map(|_| scope.spawn(repeated_bytes)).collect();
        for worker in workers {
            assert_eq!(worker.join().unwrap(), serial);
        }
    });
    let hex: String = serial.iter().map(|byte| format!("{byte:02x}")).collect();
    println!("frozen-prism-v1:{hex}");
}
fn repeated_bytes() -> Vec<u8> {
    let first = fresh_projection().replay_bytes();
    for _ in 0..3 {
        assert_eq!(fresh_projection().replay_bytes(), first);
    }
    first
}
#[test]
fn shared_published_projection_reads_are_lock_free_and_deterministic() {
    let projection = fresh_projection();
    let serial = projection.replay_bytes();
    std::thread::scope(|scope| {
        let workers: Vec<_> = (0..8)
            .map(|_| {
                scope.spawn(|| {
                    for _ in 0..32 {
                        assert_eq!(projection.replay_bytes(), serial);
                    }
                })
            })
            .collect();
        for worker in workers {
            worker.join().unwrap();
        }
    });
}

#[test]
fn frozen_containment_v1_matches_owned_pre_composition_bytes() {
    // ADR0035: structural sharing and new proof joins must preserve selectable V1 replay.
    assert_eq!(
        fresh_projection().replay_bytes(),
        include_bytes!("fixtures/containment-v1.bin")
    );
}
