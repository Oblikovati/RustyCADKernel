//! Explicit Clear traversal and allocator replacement, independent of implicit owner destruction.
#[path = "ubtree_clear_contracts/edited_graph.rs"]
mod edited_graph;
#[path = "ubtree_clear_contracts/failures.rs"]
mod failures;
#[path = "ubtree_kill_contracts/fixture.rs"]
mod fixture;
use fixture::{Trace, populated};
use rustycad_tkernel::ncollection::BaseAllocator;
const CLEAR_TRACE: &str = "DO0,DB10,DO0,DB30,DO1,DB1,DO3,DB3,DO0,DB40,DO2,DB2,DO4,DB4,";
fn allocator_policies() -> [BaseAllocator; 5] {
    [
        BaseAllocator::common(),
        BaseAllocator::global_heap(),
        BaseAllocator::incremental(1024),
        BaseAllocator::accumulating(1024).unwrap(),
        BaseAllocator::aligned(32),
    ]
}
#[test]
fn clear_uses_native_preorder_with_every_supported_allocator_policy() {
    for allocator in allocator_policies() {
        let trace = Trace::default();
        let mut tree = populated(&trace, allocator.clone());
        tree.clear();
        assert_eq!(trace.events(), CLEAR_TRACE);
        assert_eq!(tree.allocator(), &allocator);
        assert!(tree.is_empty());
        assert!(tree.last_node().is_none());
        assert!(trace.addresses().is_empty());
        for id in 0..7 {
            assert!(tree.node_by_id(id).is_none());
        }
    }
}
#[test]
fn clear_destroys_every_payload_at_its_original_address() {
    let trace = Trace::default();
    let mut tree = populated(&trace, BaseAllocator::common());
    let expected: Vec<usize> = [0, 1, 3, 4, 2, 5, 6]
        .into_iter()
        .flat_map(|id| {
            let node = tree.node_by_id(id).unwrap();
            [
                std::ptr::from_ref(node.object()).addr(),
                std::ptr::from_ref(node.bounds()).addr(),
            ]
        })
        .collect();
    tree.clear();
    assert_eq!(trace.destructor_addresses(), expected);
}
#[test]
fn clear_releases_old_incremental_leases_then_adopts_requested_allocator() {
    let trace = Trace::default();
    let original = BaseAllocator::incremental(1024);
    let replacement = BaseAllocator::incremental(1024);
    let mut tree = populated(&trace, original.clone());
    assert!(original.reset(true).is_err());
    tree.clear_with_allocator(Some(replacement.clone()));
    assert_eq!(trace.events(), CLEAR_TRACE);
    assert_eq!(tree.allocator(), &replacement);
    original.reset(true).unwrap();
    tree.add(trace.item(9), trace.item(9));
    assert_eq!(tree.root_id(), Some(0));
    assert!(replacement.reset(true).is_err());
    tree.clear();
    replacement.reset(true).unwrap();
}
#[test]
fn repeated_empty_clear_preserves_or_replaces_allocator_without_callbacks() {
    let trace = Trace::default();
    let mut tree = populated(&trace, BaseAllocator::common());
    tree.clear();
    trace.clear();
    tree.clear();
    assert_eq!(tree.allocator(), &BaseAllocator::common());
    let replacement = BaseAllocator::incremental(1024);
    tree.clear_with_allocator(Some(replacement.clone()));
    assert_eq!(tree.allocator(), &replacement);
    assert_eq!(trace.events(), "");
}

#[test]
fn clear_replacement_preserves_native_trace_for_each_allocator_policy() {
    for original in allocator_policies() {
        let trace = Trace::default();
        let mut tree = populated(&trace, original);
        let replacement = BaseAllocator::incremental(1024);
        tree.clear_with_allocator(Some(replacement.clone()));
        assert_eq!(trace.events(), CLEAR_TRACE);
        assert_eq!(tree.allocator(), &replacement);
        assert!(tree.is_empty());
    }
}
#[test]
fn clear_traverses_only_live_nodes_after_prior_kill() {
    let trace = Trace::default();
    let mut tree = populated(&trace, BaseAllocator::common());
    tree.kill(0, 0);
    trace.clear();
    tree.clear();
    assert_eq!(trace.events(), "DO0,DB40,DO2,DB2,DO4,DB4,");
    assert!(tree.is_empty());
}
