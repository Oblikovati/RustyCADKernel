// SPDX-License-Identifier: GPL-2.0-only
use super::super::names::reservations::inspect_reservations;
use super::{fixture::*, *};

fn complete_semantic_bytes() -> Vec<u8> {
    let invoked = invoked_prism_fixture(super::super::test_fixture::request());
    let names = body_map::issue_resolved_prism_names(&invoked).unwrap();
    inspect_reservations(&invoked, &names)
        .unwrap()
        .replay_bytes()
        .to_vec()
}
#[test]
fn actual_resolver_process_replay() {
    let bytes = complete_semantic_bytes();
    println!(
        "actual-semantic-resolver-v2={}",
        bytes.iter().map(|b| format!("{b:02x}")).collect::<String>()
    );
}
#[test]
fn independent_thread_operations_and_shared_owned_readers_match_serial_bytes() {
    let expected = complete_semantic_bytes();
    let workers: Vec<_> = (0..8)
        .map(|_| std::thread::spawn(complete_semantic_bytes))
        .collect();
    for worker in workers {
        assert_eq!(worker.join().unwrap(), expected);
    }
    let result = std::sync::Arc::new(default_resolved_prism_fixture());
    let readers: Vec<_> = (0..8)
        .map(|_| {
            let result = result.clone();
            std::thread::spawn(move || result.names.replay_bytes().to_vec())
        })
        .collect();
    for reader in readers {
        assert_eq!(reader.join().unwrap(), result.names.replay_bytes());
    }
    assert_eq!(complete_semantic_bytes(), expected);
}
