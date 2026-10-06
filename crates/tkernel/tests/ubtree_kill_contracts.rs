//! Native Kill member assignment and subtree destruction; pair allocation remains separate.
#[path = "ubtree_kill_contracts/failures.rs"]
mod failures;
#[path = "ubtree_kill_contracts/fixture.rs"]
mod fixture;
use fixture::{Trace, Tree, populated};
use rustycad_tkernel::ncollection::BaseAllocator;
fn verify_links(tree: &Tree, children: [usize; 2]) {
    let root = tree.root().unwrap();
    assert!(root.is_root());
    for (slot, child) in children.into_iter().enumerate() {
        assert_eq!(root.child_id(slot as i32), Some(child));
        assert_eq!(tree.node_by_id(child).unwrap().parent_id(), Some(0));
    }
}
#[test]
fn kill_zero_assigns_in_place_then_destroys_survivor_and_preorder_branch() {
    let trace = Trace::default();
    let mut tree = populated(&trace, BaseAllocator::common());
    let root = tree.root().unwrap();
    let addresses = [
        std::ptr::from_ref(root.bounds()).addr(),
        std::ptr::from_ref(root.object()).addr(),
    ];
    tree.kill(0, 0);
    assert_eq!(
        trace.events(),
        "AB40,AO0,DO0,DB40,DO0,DB30,DO1,DB1,DO3,DB3,"
    );
    assert_eq!(trace.addresses(), addresses);
    verify_links(&tree, [5, 6]);
    for dead in [1, 2, 3, 4] {
        assert!(tree.node_by_id(dead).is_none());
    }
}
#[test]
fn kill_one_uses_child_zero_survivor_and_child_zero_first_destruction() {
    let trace = Trace::default();
    let mut tree = populated(&trace, BaseAllocator::common());
    tree.kill(0, 1);
    assert_eq!(
        trace.events(),
        "AB30,AO0,DO0,DB30,DO0,DB40,DO2,DB2,DO4,DB4,"
    );
    verify_links(&tree, [3, 4]);
    for dead in [1, 2, 5, 6] {
        assert!(tree.node_by_id(dead).is_none());
    }
}
#[test]
fn kill_leaf_and_checked_invalid_arguments_have_no_callbacks() {
    let trace = Trace::default();
    let mut tree = populated(&trace, BaseAllocator::common());
    for (node, index) in [(3, 0), (0, -1), (0, 2), (usize::MAX, 0)] {
        tree.kill(node, index);
    }
    assert_eq!(trace.events(), "");
    assert_eq!(tree.root().unwrap().child_id(0), Some(1));
}
#[test]
fn nested_kill_preserves_parent_and_stable_leaf() {
    let trace = Trace::default();
    let mut tree = populated(&trace, BaseAllocator::common());
    tree.kill(1, 0);
    assert_eq!(trace.events(), "AB3,AO3,DO3,DB3,DO1,DB1,");
    let survivor = tree.node_by_id(1).unwrap();
    assert_eq!(survivor.parent_id(), Some(0));
    assert!(survivor.is_leaf());
    assert_eq!(survivor.object().value, 3);
}

#[test]
fn removed_payloads_are_destroyed_at_original_allocation_addresses() {
    let trace = Trace::default();
    let mut tree = populated(&trace, BaseAllocator::common());
    let expected: Vec<usize> = [2, 1, 3, 4]
        .into_iter()
        .flat_map(|id| {
            let node = tree.node_by_id(id).unwrap();
            [
                std::ptr::from_ref(node.object()).addr(),
                std::ptr::from_ref(node.bounds()).addr(),
            ]
        })
        .collect();
    tree.kill(0, 0);
    assert_eq!(trace.destructor_addresses(), expected);
}
