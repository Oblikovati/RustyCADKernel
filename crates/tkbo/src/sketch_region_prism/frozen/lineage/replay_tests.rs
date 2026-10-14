// SPDX-License-Identifier: GPL-2.0-only
use super::{tests::*, *};
fn fresh_bytes() -> Vec<u8> {
    let (_, receipt, publication, authored) = fixture();
    inspected(run(&receipt, &publication, &authored)).replay_bytes()
}
#[test]
fn prism_copy_lineage_owned_bytes_replay_across_processes_and_threads() {
    let serial = fresh_bytes();
    std::thread::scope(|scope| {
        let workers: Vec<_> = (0..8)
            .map(|_| {
                scope.spawn(|| {
                    let first = fresh_bytes();
                    for _ in 0..3 {
                        assert_eq!(first, fresh_bytes());
                    }
                    first
                })
            })
            .collect();
        for worker in workers {
            assert_eq!(serial, worker.join().unwrap());
        }
    });
    print_replay(&serial);
}
fn print_replay(serial: &[u8]) {
    let hex = serial
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    println!("prism-copy-lineage-v1:{hex}");
}
#[test]
fn owned_lineage_reads_are_shared_without_locks() {
    let (_, receipt, publication, authored) = fixture();
    let result = inspected(run(&receipt, &publication, &authored));
    let before = result.replay_bytes();
    std::thread::scope(|scope| {
        let workers: Vec<_> = (0..8)
            .map(|_| {
                scope.spawn(|| {
                    for _ in 0..32 {
                        assert_eq!(result.replay_bytes(), before);
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
fn observation_and_authored_binding_order_do_not_define_identity() {
    let (_, mut receipt, mut publication, mut authored) = fixture();
    let before = inspected(run(&receipt, &publication, &authored)).replay_bytes();
    receipt.copied.as_mut().unwrap().observations.reverse();
    authored.selected_members.reverse();
    authored.selected_uses.reverse();
    authored.copied_uses.reverse();
    publication.sources.reverse();
    assert_eq!(
        before,
        inspected(run(&receipt, &publication, &authored)).replay_bytes()
    );
}
