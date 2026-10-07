#![allow(unsafe_code)]
use crate::allocation_trace::*;
use rustycad_tkernel::ncollection::BaseAllocator;
pub fn populated(count: usize) -> (Tree, BaseAllocator) {
    let allocator = BaseAllocator::accumulating(65536).unwrap();
    let mut tree = Tree::with_allocator(allocator.clone());
    for index in 1..=count {
        let index = i32::try_from(index).unwrap();
        tree.add_ref(&Payload(index), &Payload(index * 10));
    }
    (tree, allocator)
}
fn native_row(count: usize, rejection: &str) -> &'static str {
    let prefix = format!("add {count} {rejection} ");
    include_str!("../../../../tools/oracle/tkernel_ubtree_add_contracts.expected")
        .lines()
        .find(|line| line.starts_with(&prefix))
        .unwrap()
}
fn apply(tree: &mut Tree, allocator: &BaseAllocator, rejection: &str) -> (bool, String) {
    let inputs = (Payload(9), Payload(90));
    start(allocator, rejection_tuple(rejection));
    let failed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        assert!(tree.add_ref(&inputs.0, &inputs.1));
    }))
    .is_err();
    (failed, finish())
}
fn result_row(tree: &Tree, failed: bool, events: &str, count: usize, rejection: &str) -> String {
    let last = tree.last_node().map_or(-1, |node| node.object().0);
    let bound = tree.root().map_or(-1, |node| node.bounds().0);
    let live = allocations(tree.allocator());
    format!(
        "add {count} {rejection} {events} {} {last} {bound} {live}",
        u8::from(failed)
    )
}
pub fn exact_case(count: usize, rejection: &str) {
    let (mut tree, allocator) = populated(count);
    let (failed, events) = apply(&mut tree, &allocator, rejection);
    assert_eq!(
        result_row(&tree, failed, &events, count, rejection),
        native_row(count, rejection)
    );
    tree.clear();
    assert_eq!(allocations(&allocator), 0);
}
