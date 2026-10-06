use super::{
    fixture::{Trace, caught, populated},
    verify_links,
};
use rustycad_tkernel::ncollection::BaseAllocator;
fn assignment_failure(kind: char, expected: &str) {
    let trace = Trace::default();
    let mut tree = populated(&trace, BaseAllocator::common());
    trace.reject('A', kind);
    assert!(caught(|| tree.kill(1, 0)));
    assert_eq!(trace.events(), expected);
    let node = tree.node_by_id(1).unwrap();
    assert_eq!(node.bounds().value, 3);
    assert_eq!(node.object().value, if kind == 'B' { 0 } else { 3 });
    assert_eq!(node.child_id(0), Some(3));
    assert_eq!(node.child_id(1), Some(4));
    assert_eq!(tree.node_by_id(4).unwrap().parent_id(), Some(1));
}
#[test]
fn bounds_assignment_failure_preserves_original_live_links() {
    assignment_failure('B', "AB3,");
}
#[test]
fn object_assignment_failure_preserves_original_live_links() {
    assignment_failure('O', "AB3,AO3,");
}
fn destructor_failure(kind: char) {
    let trace = Trace::default();
    let allocator = BaseAllocator::incremental(1024);
    let mut tree = populated(&trace, allocator.clone());
    trace.reject('D', kind);
    assert!(caught(|| tree.kill(0, 0)));
    assert_eq!(
        trace.events(),
        "AB40,AO0,DO0,DB40,DO0,DB30,DO1,DB1,DO3,DB3,"
    );
    verify_links(&tree, [5, 6]);
    tree.clear();
    assert!(allocator.reset(false).is_ok());
}
#[test]
fn object_destructor_panic_drains_detached_nodes_and_releases_leases() {
    destructor_failure('O');
}
#[test]
fn bounds_destructor_panic_drains_detached_nodes_and_releases_leases() {
    destructor_failure('B');
}
