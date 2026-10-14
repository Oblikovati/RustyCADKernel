// SPDX-License-Identifier: GPL-2.0-only
use super::*;

fn replay() -> Vec<u8> {
    let fixture = JoinFixture::new();
    let join = fixture.compose(|| false).ok().unwrap();
    measured(&join).replay_bytes()
}
#[test]
fn joined_measure_process_replay() {
    let bytes = replay();
    // Full address-free bytes, not only scalar endpoints or a digest, are compared by the owner runner.
    println!(
        "joined-measures-v1={}",
        bytes
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    );
}
#[test]
fn independent_workers_and_shared_reads_match_serial_full_replay() {
    let expected = replay();
    let workers: Vec<_> = (0..4).map(|_| std::thread::spawn(replay)).collect();
    for worker in workers {
        assert_eq!(worker.join().unwrap(), expected);
    }
    let join = std::sync::Arc::new(JoinFixture::new().compose(|| false).ok().unwrap());
    let readers: Vec<_> = (0..4)
        .map(|_| {
            let shared = join.clone();
            std::thread::spawn(move || measured(&shared).replay_bytes())
        })
        .collect();
    for reader in readers {
        assert_eq!(reader.join().unwrap(), expected);
    }
}
