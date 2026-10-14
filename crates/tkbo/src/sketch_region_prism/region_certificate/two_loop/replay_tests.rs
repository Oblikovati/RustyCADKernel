// SPDX-License-Identifier: GPL-2.0-only
use super::{fixture::*, operation_tests::*};
fn receipt() -> Vec<u8> {
    let fixture = TwoLoopFixture::nested();
    accepted(inspect(&fixture, &mut OperationProgress)).replay
}
#[test]
fn two_loop_owned_process_receipt() {
    let bytes = receipt();
    let hex: String = bytes.iter().map(|v| format!("{v:02x}")).collect();
    println!("two-loop-region-domain-v2:{hex}");
}
#[test]
fn two_loop_independent_thread_operations_match_serial_bytes() {
    let expected = receipt();
    let workers: Vec<_> = (0..4)
        .map(|_| std::thread::spawn(|| [receipt(), receipt()]))
        .collect();
    for worker in workers {
        for actual in worker.join().unwrap() {
            assert_eq!(actual, expected);
        }
    }
}
#[test]
fn two_loop_two_independent_processes_match_complete_owned_bytes() {
    let expected: String = receipt().iter().map(|v| format!("{v:02x}")).collect();
    assert_eq!(child_receipt(), expected);
    assert_eq!(child_receipt(), expected);
}
fn child_receipt() -> String {
    let output=std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact","sketch_region_prism::region_certificate::two_loop::replay_tests::two_loop_owned_process_receipt","--nocapture"]).output().unwrap();
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    let rows: Vec<_> = text
        .lines()
        .filter_map(|line| line.strip_prefix("two-loop-region-domain-v2:"))
        .collect();
    assert_eq!(rows.len(), 1);
    rows[0].to_owned()
}
