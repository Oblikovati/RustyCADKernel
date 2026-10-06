use super::fixture::{Editable, editable_populated, editable_with_traces};
use crate::{
    CLEAR_TRACE,
    trace::{Trace, caught},
};
use rustycad_tkernel::ncollection::BaseAllocator;
fn verify_empty(tree: &Editable, allocator: &BaseAllocator, trace: &Trace) {
    assert!(tree.is_empty());
    assert!(!tree.contains(&trace.item(1)));
    assert_eq!(tree.as_ubtree().allocator(), allocator);
    allocator.reset(true).unwrap();
}
fn clear_failure(kind: char) {
    let trace = Trace::default();
    let allocator = BaseAllocator::incremental(1024);
    let mut tree = editable_populated(&trace, allocator.clone(), 1);
    trace.reject('D', kind);
    let replacement = BaseAllocator::incremental(2048);
    assert!(caught(|| tree.clear_with_allocator(Some(replacement))));
    assert_eq!(trace.events(), CLEAR_TRACE);
    verify_empty(&tree, &allocator, &trace);
    tree.add(trace.item(11), trace.item(11));
    assert_eq!(tree.find_node(&trace.item(11)).unwrap().object().value, 11);
    tree.clear();
    allocator.reset(true).unwrap();
}
#[test]
fn map_key_destructor_panic_cleans_both_structures_and_preserves_allocator() {
    clear_failure('O');
}
#[test]
fn bounds_destructor_panic_cleans_both_structures_and_preserves_allocator() {
    clear_failure('B');
}
fn assignment_failure(kind: char) {
    let trace = Trace::default();
    let source_trace = Trace::default();
    let old = BaseAllocator::incremental(1024);
    let donor = BaseAllocator::incremental(2048);
    let mut target = editable_populated(&trace, old.clone(), 1);
    let mut source = editable_populated(&source_trace, donor.clone(), 7);
    let root = std::ptr::from_ref(source.as_ubtree().root().unwrap());
    trace.reject('D', kind);
    assert!(caught(|| {
        target.move_assign(&mut source);
    }));
    assert_eq!(trace.events(), CLEAR_TRACE);
    verify_empty(&target, &old, &trace);
    assert_eq!(std::ptr::from_ref(source.as_ubtree().root().unwrap()), root);
    assert_eq!(source.as_ubtree().allocator(), &donor);
    assert_eq!(source_trace.events(), "");
    assert!(source.contains(&source_trace.item(7)));
}
#[test]
fn map_key_destructor_panic_prevents_source_transfer() {
    assignment_failure('O');
}
#[test]
fn tree_bounds_destructor_panic_prevents_source_transfer() {
    assignment_failure('B');
}
#[test]
fn independent_map_and_tree_panics_drain_both_and_resume_first_failure() {
    let object = Trace::default();
    let bounds = Trace::default();
    let allocator = BaseAllocator::incremental(1024);
    let mut tree = editable_with_traces(&object, &bounds, allocator.clone(), 1);
    object.reject('D', 'O');
    bounds.reject('D', 'B');
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| tree.clear()));
    let panic = result.unwrap_err().downcast::<String>().unwrap();
    assert!(panic.contains("payload O"));
    assert_eq!(object.events(), "DO1,DO2,DO3,DO0,DO0,DO1,DO3,DO2,");
    assert_eq!(bounds.events(), "DB2,DB3,DB1,DB3,DB2,");
    verify_empty(&tree, &allocator, &object);
}
