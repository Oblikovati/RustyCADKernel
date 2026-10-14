// SPDX-License-Identifier: GPL-2.0-only
use super::super::traversal_tests::ordered_projection as fresh_projection;
use super::{tests::*, *};
#[test]
fn exact_topology_report_replays_across_threads_and_processes() {
    let source = fresh_projection();
    let serial = certified(&source).replay_bytes();
    std::thread::scope(|scope| {
        let workers: Vec<_> = (0..8)
            .map(|_| {
                scope.spawn(|| {
                    for _ in 0..16 {
                        assert_eq!(certified(&source).replay_bytes(), serial);
                    }
                })
            })
            .collect();
        for worker in workers {
            worker.join().unwrap();
        }
    });
    let hex: String = serial.iter().map(|byte| format!("{byte:02x}")).collect();
    println!("frozen-topology-v1:{hex}");
}
#[test]
fn independent_operation_history_does_not_affect_certificate_bytes() {
    let first = certified(&fresh_projection()).replay_bytes();
    let invalid = super::fixtures::NamedCellComplex::double_tetrahedron(true).projection();
    rejected(&invalid, TopologyInvariant::CyclicVertexLinks);
    let second = certified(&fresh_projection()).replay_bytes();
    assert_eq!(first, second);
}
#[test]
fn immutable_reports_and_operation_types_are_send_sync() {
    use static_assertions::assert_impl_all;
    assert_impl_all!(FrozenTopologyCertificate: Send, Sync);
    assert_impl_all!(FrozenTopologyOutcome: Send, Sync);
    assert_impl_all!(TopologyFailure: Send, Sync);
    assert_impl_all!(TopologyWitness: Send, Sync);
    assert_impl_all!(TopologyActual: Send, Sync);
    assert_impl_all!(TopologyCheckpoint: Send, Sync);
    assert_impl_all!(TopologyInvariant: Send, Sync);
    assert_impl_all!(TopologyNumericalError: Send, Sync);
    assert_impl_all!(TopologyEvidenceKind: Send, Sync);
    assert_impl_all!(super::model::TopologyIndex<'static>: Send, Sync);
    assert_impl_all!(super::execution::TopologyWork<'static>: Send);
}
