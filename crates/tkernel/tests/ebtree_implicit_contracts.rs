//! Native derived map destruction precedes base-tree payload destruction.
#[path = "support/ubtree_allocation_trace.rs"]
pub mod allocation_trace;
use allocation_trace::*;
use rustycad_tkernel::ncollection::{BaseAllocator, EBTree, HashKey};
impl HashKey for Payload<'O'> {
    fn occt_hash(&self) -> usize {
        self.0.occt_hash()
    }
}
type Editable = EBTree<Payload<'O'>, Payload<'B'>>;
fn populate(tree: &mut Editable, count: i32) {
    for index in 1..=count {
        tree.add_ref(&Payload(index), &Payload(index * 10));
    }
}
fn native_row(phase: &str, count: i32) -> &'static str {
    let prefix = format!("{phase} {count} ");
    include_str!("../../../tools/oracle/tkernel_ebtree_implicit_contracts.expected")
        .lines()
        .find(|line| line.starts_with(&prefix))
        .unwrap()
}
fn implicit_case(count: i32) {
    let allocator = BaseAllocator::accumulating(65536).unwrap();
    let mut tree = Editable::with_allocator(allocator.clone());
    populate(&mut tree, count);
    start(&allocator, ('X', 'X', 0));
    drop(tree);
    let events = finish();
    let row = format!("drop {count} {events} {}", allocations(&allocator));
    assert_eq!(row, native_row("drop", count));
}
#[test]
fn implicit_empty_tree_releases_no_payloads() {
    implicit_case(0);
}
#[test]
fn implicit_root_destroys_map_key_before_tree_members() {
    implicit_case(1);
}
#[test]
fn implicit_branches_destroy_entire_map_before_native_tree_preorder() {
    implicit_case(2);
    implicit_case(3);
}
#[test]
fn broader_editable_convenience_construction_gap_remains_explicit() {
    let allocator = BaseAllocator::accumulating(65536).unwrap();
    let mut tree = Editable::with_allocator(allocator.clone());
    let input = (Payload(1), Payload(10));
    start(&allocator, ('X', 'X', 0));
    tree.add_ref(&input.0, &input.1);
    assert_eq!(finish(), "CO1:0,CB10:0,CO1:1,");
    assert_eq!(
        native_row("construct", 1),
        "construct 1 CB10:1,CO1:1,CO1:1, 1"
    );
}

fn native_construction_case(count: i32) {
    let allocator = BaseAllocator::accumulating(65536).unwrap();
    let mut tree = Editable::with_allocator(allocator.clone());
    let objects = [Payload(1), Payload(2), Payload(3)];
    let bounds = [Payload(10), Payload(20), Payload(30)];
    start(&allocator, ('X', 'X', 0));
    for index in 0..usize::try_from(count).unwrap() {
        tree.add_native_ref(&objects[index], &bounds[index]);
    }
    let row = format!("construct {count} {} {}", finish(), allocations(&allocator));
    assert_eq!(row, native_row("construct", count));
}
#[test]
fn native_borrowed_adapter_matches_all_previously_unmatched_construction_rows() {
    for count in 0..=3 {
        native_construction_case(count);
    }
}
