use super::{
    CLEAR_TRACE,
    fixture::{Trace, caught, populated},
};
use rustycad_tkernel::ncollection::BaseAllocator;
fn destructor_failure(kind: char) {
    let trace = Trace::default();
    let original = BaseAllocator::incremental(1024);
    let replacement = BaseAllocator::incremental(1024);
    let mut tree = populated(&trace, original.clone());
    trace.reject('D', kind);
    assert!(caught(|| tree.clear_with_allocator(Some(replacement))));
    assert_eq!(trace.events(), CLEAR_TRACE);
    assert!(tree.is_empty());
    assert!(tree.last_node().is_none());
    assert_eq!(tree.allocator(), &original);
    original.reset(true).unwrap();
    tree.add(trace.item(9), trace.item(9));
    assert_eq!(tree.root().unwrap().object().value, 9);
    assert_eq!(tree.root_id(), Some(0));
    tree.clear();
    original.reset(true).unwrap();
}
#[test]
fn object_destructor_panic_drains_tree_and_retains_original_allocator() {
    destructor_failure('O');
}
#[test]
fn bounds_destructor_panic_drains_tree_and_retains_original_allocator() {
    destructor_failure('B');
}
