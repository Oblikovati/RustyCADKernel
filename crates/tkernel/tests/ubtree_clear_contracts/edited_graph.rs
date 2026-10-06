//! Public mutable nodes can copy links or detach descendants; ownership still belongs to the arena.
use super::fixture::{Trace, caught, populated};
use rustycad_tkernel::ncollection::{BaseAllocator, BoundingNode};
#[test]
fn clear_drains_detached_descendants_even_if_an_orphan_destructor_panics() {
    let trace = Trace::default();
    let allocator = BaseAllocator::incremental(1024);
    let mut tree = populated(&trace, allocator.clone());
    let quiet = Trace::default();
    *tree.change_node_by_id(0).unwrap() = BoundingNode::new(quiet.item(77), quiet.item(77));
    trace.clear();
    trace.reject('D', 'O');
    assert!(caught(|| tree.clear()));
    assert_eq!(
        trace.events(),
        "DO0,DB30,DO0,DB40,DO1,DB1,DO3,DB3,DO2,DB2,DO4,DB4,"
    );
    assert!(tree.is_empty());
    allocator.reset(true).unwrap();
}
#[test]
fn clear_terminates_after_public_link_copy_creates_cycle_and_duplicate_edges() {
    let trace = Trace::default();
    let allocator = BaseAllocator::incremental(1024);
    let mut tree = populated(&trace, allocator.clone());
    let root = tree.root().unwrap().clone();
    tree.change_node_by_id(1).unwrap().clone_from(&root);
    drop(root);
    trace.clear();
    tree.clear();
    assert_eq!(
        trace.events(),
        "DO0,DB10,DO0,DB10,DO0,DB40,DO2,DB2,DO4,DB4,DO1,DB1,DO3,DB3,"
    );
    allocator.reset(true).unwrap();
}
#[test]
fn clear_handles_foreign_arena_child_ids_without_losing_local_ownership() {
    let trace = Trace::default();
    let mut source = populated(&trace, BaseAllocator::common());
    source.gemmate(0, trace.item(80), trace.item(8), trace.item(8));
    let mut target = populated(&trace, BaseAllocator::common());
    target
        .change_node_by_id(0)
        .unwrap()
        .clone_from(source.root().unwrap());
    trace.clear();
    target.clear();
    assert_eq!(
        trace.events(),
        "DO0,DB80,DO0,DB30,DO0,DB40,DO1,DB1,DO3,DB3,DO2,DB2,DO4,DB4,"
    );
    assert!(target.is_empty());
    assert!(!source.is_empty());
}
