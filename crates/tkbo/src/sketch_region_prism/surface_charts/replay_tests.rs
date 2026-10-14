// SPDX-License-Identifier: GPL-2.0-only
//! Compare owned chart proof bytes; compatibility topology is rebuilt inside each worker.
use super::super::{tests::*, *};
use super::{expected_tests::*, tests::*, *};

#[test]
fn chart_proofs_replay_across_independent_requests_and_threads() {
    let serial = proof_bytes();
    std::thread::scope(|scope| {
        let workers: Vec<_> = (0..8).map(|_| scope.spawn(repeated_proof_bytes)).collect();
        for worker in workers {
            assert_eq!(worker.join().unwrap(), serial);
        }
    });
    assert_eq!(proof_bytes(), serial);
    println!("chart-proof-v1:{serial}");
}
fn repeated_proof_bytes() -> String {
    let first = proof_bytes();
    for _ in 0..4 {
        assert_eq!(proof_bytes(), first);
    }
    first
}
fn proof_bytes() -> String {
    let receipt = cubic_receipt([1., 1.], &mut progress(None, None));
    assert_cubic_collection(inspection(&receipt));
    assert_body_incomplete(&receipt);
    assert_expected_caps(&receipt);
    let side = entry_by_role(&receipt, PrismImageRole::Generated);
    assert_expected_side(side);
    assert_expected_side_trims(side);
    [
        PrismImageRole::First,
        PrismImageRole::Last,
        PrismImageRole::Generated,
    ]
    .map(|role| {
        let entry = entry_by_role(&receipt, role);
        let parents: Vec<_> = entry.parents.iter().map(|parent| parent.source.0).collect();
        format!("{role:?}:{:?}:{parents:?}", entry.proof)
    })
    .join(";")
}
