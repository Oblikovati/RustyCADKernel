// SPDX-License-Identifier: GPL-2.0-only
use super::{test_support::*, *};
fn portable() -> Vec<u8> {
    let join = JoinFixture::new().compose(|| false).ok().unwrap();
    complete(&join).replay_bytes()
}
#[test]
fn ordered_validation_process_receipt() {
    let bytes = portable();
    let hex: String = bytes.iter().map(|byte| format!("{byte:02x}")).collect();
    println!("ordered-validation-v1:{hex}");
}
#[test]
fn complete_receipts_match_after_prior_work_across_workers_and_processes() {
    let serial = portable();
    let workers: Vec<_> = (0..8)
        .map(|_| std::thread::spawn(|| (0..4).map(|_| portable()).collect::<Vec<_>>()))
        .collect();
    for worker in workers {
        for bytes in worker.join().unwrap() {
            assert_eq!(bytes, serial);
        }
    }
    assert_eq!(portable(), serial);
    for _ in 0..2 {
        assert_eq!(child(), serial);
    }
}
fn child() -> Vec<u8> {
    let result = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "sketch_region_prism::frozen::ordered_validation::replay_tests::ordered_validation_process_receipt", "--nocapture"])
        .output().unwrap();
    assert!(result.status.success());
    decode_receipt(&String::from_utf8(result.stdout).unwrap())
}
fn decode_receipt(stdout: &str) -> Vec<u8> {
    let rows: Vec<_> = stdout
        .lines()
        .filter_map(|row| row.strip_prefix("ordered-validation-v1:"))
        .collect();
    assert_eq!(rows.len(), 1);
    let (pairs, remainder) = rows[0].as_bytes().as_chunks::<2>();
    assert!(remainder.is_empty());
    pairs
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}
#[test]
fn shared_published_join_can_be_read_concurrently_without_live_sources() {
    let join = std::sync::Arc::new(JoinFixture::new().compose(|| false).ok().unwrap());
    let serial = complete(&join).replay_bytes();
    let workers: Vec<_> = (0..8)
        .map(|_| {
            let joined = join.clone();
            std::thread::spawn(move || complete(&joined).replay_bytes())
        })
        .collect();
    for worker in workers {
        assert_eq!(worker.join().unwrap(), serial);
    }
}
