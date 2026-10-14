// SPDX-License-Identifier: GPL-2.0-only
use std::process::Command;
const CHILD: &str = "exact_roots::bezier_line::directed_process_replay::directed_replay_child";
const MARKER: &str = "MARTENSITE_DIRECTED_REPLAY_CHILD";

#[test]
fn directed_replay_child() {
    if std::env::var_os(MARKER).is_none() {
        return;
    }
    let bytes = super::directed_ownership::replay_bytes();
    println!(
        "DIRECTED_REPLAY_BEGIN\n{}DIRECTED_REPLAY_END",
        String::from_utf8(bytes).unwrap()
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
        .split_once("DIRECTED_REPLAY_BEGIN\n")
        .unwrap()
        .1
        .split_once("DIRECTED_REPLAY_END")
        .unwrap()
        .0
        .as_bytes()
        .to_vec()
}
#[test]
fn independent_processes_replay_serial_certificate_bytes() {
    let first = child_bytes();
    assert_eq!(first, child_bytes());
    assert_eq!(first, super::directed_ownership::replay_bytes());
}
