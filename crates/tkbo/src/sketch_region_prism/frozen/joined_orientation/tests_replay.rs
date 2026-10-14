// SPDX-License-Identifier: GPL-2.0-only
use super::*;
fn portable() -> Vec<u8> {
    let (_, join) = fixture();
    certified(&join).replay_bytes()
}
#[test]
fn joined_orientation_process_receipt() {
    let bytes = portable();
    let hex: String = bytes.iter().map(|byte| format!("{byte:02x}")).collect();
    println!("joined-orientation-v1:{hex}");
}
#[test]
fn independent_workers_and_two_processes_match_complete_owned_bytes() {
    let serial = portable();
    let workers: Vec<_> = (0..8)
        .map(|_| std::thread::spawn(|| (0..4).map(|_| portable()).collect::<Vec<_>>()))
        .collect();
    for worker in workers {
        for bytes in worker.join().unwrap() {
            assert_eq!(bytes, serial);
        }
    }
    for _ in 0..2 {
        assert_eq!(child(), serial);
    }
}
fn child() -> Vec<u8> {
    let result = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "sketch_region_prism::frozen::joined_orientation::tests::replay_tests::joined_orientation_process_receipt", "--nocapture"])
        .output().unwrap();
    assert!(result.status.success());
    let stdout = String::from_utf8(result.stdout).unwrap();
    let rows: Vec<_> = stdout
        .lines()
        .filter_map(|row| row.strip_prefix("joined-orientation-v1:"))
        .collect();
    assert_eq!(rows.len(), 1);
    rows[0]
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}
