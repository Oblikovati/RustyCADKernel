// SPDX-License-Identifier: GPL-2.0-only
use std::process::Command;
const CHILD: &str = "exact_roots::bezier_regularity::replay::regularity_replay_child";
const MARKER: &str = "MARTENSITE_REGULARITY_REPLAY_CHILD";
#[test]
fn regularity_replay_child() {
    if std::env::var_os(MARKER).is_none() {
        return;
    }
    println!(
        "REGULARITY_BEGIN\n{}REGULARITY_END",
        String::from_utf8(super::ownership::replay_bytes()).unwrap()
    );
}
fn child_bytes() -> Vec<u8> {
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", CHILD, "--nocapture", "--test-threads=1"])
        .env(MARKER, "1")
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    stdout
        .split_once("REGULARITY_BEGIN\n")
        .unwrap()
        .1
        .split_once("REGULARITY_END")
        .unwrap()
        .0
        .as_bytes()
        .to_vec()
}
#[test]
fn independent_process_replay_matches_serial_bytes() {
    let first = child_bytes();
    assert_eq!(first, child_bytes());
    assert_eq!(first, super::ownership::replay_bytes());
}
