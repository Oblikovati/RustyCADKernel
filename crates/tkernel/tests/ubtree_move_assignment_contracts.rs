//! Native destination-clear-before-transfer ordering and allocator-handle exchange.
#[path = "ubtree_move_assignment_contracts/failures.rs"]
mod failures;
#[path = "ubtree_kill_contracts/fixture.rs"]
mod fixture;
use fixture::{Trace, Tree, populated};
use rustycad_tkernel::ncollection::BaseAllocator;
const DROP_TRACE: &str = "DO0,DB10,DO0,DB30,DO1,DB1,DO3,DB3,DO0,DB40,DO2,DB2,DO4,DB4,";
fn optional_tree(full: bool, trace: &Trace, allocator: BaseAllocator) -> Tree {
    if full {
        populated(trace, allocator)
    } else {
        Tree::with_allocator(allocator)
    }
}
fn assignment_case(target_full: bool, source_full: bool) {
    let trace = Trace::default();
    let source_allocator = BaseAllocator::incremental(1024);
    let mut source = optional_tree(source_full, &trace, source_allocator.clone());
    let mut target = optional_tree(target_full, &trace, BaseAllocator::common());
    let root = source.root().map(std::ptr::from_ref);
    let last = source.last_node().map(std::ptr::from_ref);
    let address = std::ptr::from_ref(&target);
    assert_eq!(target.move_assign(&mut source) as *const _, address);
    assert_eq!(trace.events(), if target_full { DROP_TRACE } else { "" });
    assert_eq!(target.root().map(std::ptr::from_ref), root);
    assert_eq!(target.last_node().map(std::ptr::from_ref), last);
    assert!(source.is_empty());
    assert!(source.last_node().is_none());
    assert_eq!(target.allocator(), &source_allocator);
    assert_eq!(source.allocator(), &BaseAllocator::common());
    assert!(trace.addresses().is_empty());
}
#[test]
fn move_assignment_handles_all_empty_and_nonempty_combinations() {
    for target_full in [false, true] {
        for source_full in [false, true] {
            assignment_case(target_full, source_full);
        }
    }
}
#[test]
fn self_move_is_callback_free_and_preserves_storage() {
    for full in [false, true] {
        let trace = Trace::default();
        let mut tree = optional_tree(full, &trace, BaseAllocator::common());
        let root = tree.root().map(std::ptr::from_ref);
        let address = std::ptr::from_ref(&tree);
        assert_eq!(tree.move_assign_self() as *const _, address);
        assert_eq!(tree.root().map(std::ptr::from_ref), root);
        assert_eq!(trace.events(), "");
    }
}
#[test]
fn assignment_destroys_destination_payloads_at_original_addresses() {
    let trace = Trace::default();
    let mut target = populated(&trace, BaseAllocator::common());
    let expected: Vec<usize> = [0, 1, 3, 4, 2, 5, 6]
        .into_iter()
        .flat_map(|id| {
            let node = target.node_by_id(id).unwrap();
            [
                std::ptr::from_ref(node.object()).addr(),
                std::ptr::from_ref(node.bounds()).addr(),
            ]
        })
        .collect();
    let mut source = Tree::default();
    target.move_assign(&mut source);
    assert_eq!(trace.destructor_addresses(), expected);
}
#[test]
fn move_constructor_transfers_live_storage_without_callbacks() {
    for full in [false, true] {
        let trace = Trace::default();
        let allocator = BaseAllocator::incremental(1024);
        let mut source = optional_tree(full, &trace, allocator.clone());
        let root = source.root().map(std::ptr::from_ref);
        let moved = Tree::move_from(&mut source);
        assert_eq!(moved.root().map(std::ptr::from_ref), root);
        assert!(source.is_empty());
        assert_eq!(moved.allocator(), &allocator);
        assert_eq!(source.allocator(), &allocator);
        assert_eq!(trace.events(), "");
    }
}
