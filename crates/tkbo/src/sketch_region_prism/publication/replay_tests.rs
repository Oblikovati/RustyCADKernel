// SPDX-License-Identifier: GPL-2.0-only
//! Full portable result bytes, including historical findings and all policy operands.
use super::fixture::*;
fn result_bytes() -> Vec<u8> {
    PublicationFixture::new()
        .publish(&mut PublicationObserver::default())
        .replay_bytes()
        .to_vec()
}
#[test]
fn publication_process_replay() {
    let bytes = result_bytes();
    println!(
        "named-prism-publication-v1={}",
        bytes.iter().map(|b| format!("{b:02x}")).collect::<String>()
    );
}
#[test]
fn independent_operations_and_lock_free_shared_reads_match_serial_result() {
    let expected = result_bytes();
    let workers: Vec<_> = (0..4).map(|_| std::thread::spawn(result_bytes)).collect();
    for worker in workers {
        assert_eq!(worker.join().unwrap(), expected);
    }
    let result =
        std::sync::Arc::new(PublicationFixture::new().publish(&mut PublicationObserver::default()));
    let readers: Vec<_> = (0..4)
        .map(|_| {
            let owned = result.clone();
            std::thread::spawn(move || owned.replay_bytes().to_vec())
        })
        .collect();
    for reader in readers {
        assert_eq!(reader.join().unwrap(), expected);
    }
    assert_eq!(result_bytes(), expected);
}
