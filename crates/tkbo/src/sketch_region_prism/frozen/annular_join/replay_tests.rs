// SPDX-License-Identifier: GPL-2.0-only
use super::operation_tests::joined;
use super::*;
use std::sync::Arc;

#[test]
fn annular_frozen_join_owned_process_receipt() {
    let joined = joined();
    println!(
        "named-annular-frozen-domain-v2={}",
        joined
            .replay_bytes()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    );
}
#[test]
fn independent_annular_join_threads_and_lock_free_shared_reads_match_single_thread() {
    let expected = joined().replay_bytes().to_vec();
    let workers: Vec<_> = (0..3)
        .map(|_| std::thread::spawn(|| joined().replay_bytes().to_vec()))
        .collect();
    for worker in workers {
        assert_eq!(worker.join().unwrap(), expected);
    }
    let shared = Arc::new(joined());
    let workers: Vec<_> = (0..4)
        .map(|_| {
            let shared = shared.clone();
            std::thread::spawn(move || shared.replay_bytes().to_vec())
        })
        .collect();
    for worker in workers {
        assert_eq!(worker.join().unwrap(), expected);
    }
}
#[test]
fn owned_annular_types_have_language_enforced_send_sync_without_native_owners() {
    static_assertions::assert_impl_all!(AnnularFrozenDomain: Send, Sync);
    static_assertions::assert_impl_all!(AnnularFrozenOptions: Send, Sync);
    static_assertions::assert_impl_all!(AnnularNameCensus: Send, Sync);
    static_assertions::assert_impl_all!(NamedAnnularFace: Send, Sync);
    static_assertions::assert_impl_all!(super::operation::AnnularJoinWork<'static>: Send);
    static_assertions::assert_impl_all!(super::super::topology::AnnularTopologyCertificate: Send, Sync);
    static_assertions::assert_impl_all!(super::super::topology::AnnularTopologyOutcome: Send, Sync);
}
