//! EBTree virtual Clear ordering, allocator/map exchange and panic-safe ownership recovery.
#[path = "support/bounding_tree_trace.rs"]
mod trace;
mod ebtree_move_assignment_contracts {
    pub mod failures;
    pub mod fixture;
}
use ebtree_move_assignment_contracts::fixture::{Editable, editable_populated, optional_editable};
use rustycad_tkernel::ncollection::BaseAllocator;
use trace::Trace;
const CLEAR_TRACE: &str = "DO1,DO2,DO3,DO0,DB2,DO0,DB3,DO1,DB1,DO3,DB3,DO2,DB2,";
fn assignment_case(target_full: bool, source_full: bool) {
    let trace = Trace::default();
    let old = BaseAllocator::incremental(1024);
    let donor = BaseAllocator::incremental(2048);
    let mut target = optional_editable(target_full, &trace, old.clone(), 1);
    let mut source = optional_editable(source_full, &trace, donor.clone(), 7);
    let root = source.as_ubtree().root().map(std::ptr::from_ref);
    let target_address = std::ptr::from_ref(&target);
    assert_eq!(target.move_assign(&mut source) as *const _, target_address);
    assert_eq!(trace.events(), if target_full { CLEAR_TRACE } else { "" });
    assert_eq!(target.as_ubtree().root().map(std::ptr::from_ref), root);
    assert_eq!(target.as_ubtree().allocator(), &donor);
    assert_eq!(source.as_ubtree().allocator(), &old);
    assert!(source.is_empty());
    assert!(trace.addresses().is_empty());
    verify_bindings(&target, &source, &trace, source_full);
}
fn verify_bindings(target: &Editable, source: &Editable, trace: &Trace, full: bool) {
    assert!(!target.contains(&trace.item(1)));
    assert_eq!(target.contains(&trace.item(7)), full);
    assert!(!source.contains(&trace.item(7)));
    if full {
        assert_eq!(target.find_node(&trace.item(7)).unwrap().object().value, 7);
    }
}
#[test]
fn editable_move_assignment_handles_all_empty_and_nonempty_combinations() {
    for target in [false, true] {
        for source in [false, true] {
            assignment_case(target, source);
        }
    }
}
#[test]
fn editable_clear_destroys_index_keys_before_native_tree_preorder() {
    let trace = Trace::default();
    let mut tree = editable_populated(&trace, BaseAllocator::common(), 1);
    tree.clear();
    assert_eq!(trace.events(), CLEAR_TRACE);
    assert!(tree.is_empty());
    assert!(!tree.contains(&trace.item(1)));
}
#[test]
fn editable_self_move_preserves_storage_without_callbacks() {
    for full in [false, true] {
        let trace = Trace::default();
        let mut tree = optional_editable(full, &trace, BaseAllocator::common(), 1);
        let root = tree.as_ubtree().root().map(std::ptr::from_ref);
        let address = std::ptr::from_ref(&tree);
        assert_eq!(tree.move_assign_self() as *const _, address);
        assert_eq!(tree.as_ubtree().root().map(std::ptr::from_ref), root);
        assert_eq!(trace.events(), "");
    }
}
#[test]
fn editable_clear_keeps_node_destructors_at_original_addresses() {
    let trace = Trace::default();
    let mut tree = editable_populated(&trace, BaseAllocator::common(), 1);
    let expected: Vec<usize> = [0, 1, 3, 4, 2]
        .into_iter()
        .flat_map(|id| {
            let node = tree.as_ubtree().node_by_id(id).unwrap();
            [
                std::ptr::from_ref(node.object()).addr(),
                std::ptr::from_ref(node.bounds()).addr(),
            ]
        })
        .collect();
    tree.clear();
    assert_eq!(&trace.destructor_addresses()[3..], expected);
}
#[test]
fn editable_move_reuses_source_with_destination_old_allocator() {
    let trace = Trace::default();
    let old = BaseAllocator::incremental(1024);
    let mut target = editable_populated(&trace, old.clone(), 1);
    let mut source = editable_populated(&trace, BaseAllocator::incremental(2048), 7);
    target.move_assign(&mut source);
    old.reset(true).unwrap();
    source.add(trace.item(3), trace.item(3));
    assert!(old.reset(true).is_err());
    assert_eq!(source.find_node(&trace.item(3)).unwrap().object().value, 3);
    source.clear();
    old.reset(true).unwrap();
}
#[test]
fn editable_clear_replaces_allocator_only_after_successful_cleanup() {
    for full in [false, true] {
        let trace = Trace::default();
        let old = BaseAllocator::incremental(1024);
        let replacement = BaseAllocator::incremental(2048);
        let mut tree = optional_editable(full, &trace, old.clone(), 1);
        tree.clear_with_allocator(Some(replacement.clone()));
        assert_eq!(trace.events(), if full { CLEAR_TRACE } else { "" });
        assert_eq!(tree.as_ubtree().allocator(), &replacement);
        assert!(tree.is_empty());
        old.reset(true).unwrap();
    }
}
