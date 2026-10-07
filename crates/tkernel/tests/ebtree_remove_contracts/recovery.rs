//! Rust unwind/invalid-hash safety extensions; no throwing-destructor OCCT claim.
use super::fixture::*;
use rustycad_tkernel::ncollection::{EBTree, HashKey};
use std::{cell::Cell, rc::Rc};
#[derive(Clone, Debug, Default)]
struct RoutedKey {
    value: i32,
    mode: Rc<Cell<u8>>,
}
impl RoutedKey {
    fn new(value: i32, mode: &Rc<Cell<u8>>) -> Self {
        Self {
            value,
            mode: mode.clone(),
        }
    }
}
impl PartialEq for RoutedKey {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}
impl HashKey for RoutedKey {
    fn occt_hash(&self) -> usize {
        if self.value != 4 {
            return self.value as usize;
        }
        match self.mode.get() {
            1 => panic!("key 4 hash rejected; expected stable non-panicking policy"),
            2 => 104,
            _ => self.value as usize,
        }
    }
}
fn routed_tree(trace: &RemoveTrace, mode: &Rc<Cell<u8>>) -> EBTree<RoutedKey, RemoveBounds> {
    let mut tree = EBTree::default();
    for value in 1..=4 {
        tree.add(RoutedKey::new(value, mode), trace.bounds(1 << value));
    }
    trace.reset('\0', 0);
    tree
}
fn hash_failure(mode_value: u8) {
    let trace = RemoveTrace::default();
    let mode = Rc::new(Cell::new(0));
    let mut tree = routed_tree(&trace, &mode);
    let removed = RoutedKey::new(1, &mode);
    let address = routed_parent_address(&tree, &removed);
    mode.set(mode_value);
    assert_eq!(
        caught(|| {
            tree.remove(&removed);
        }),
        mode_value == 1
    );
    mode.set(0);
    let survivor = RoutedKey::new(4, &mode);
    assert_routed_survivor(&tree, &survivor, address);
    assert!(!tree.contains(&removed));
}
fn routed_parent_address(tree: &EBTree<RoutedKey, RemoveBounds>, removed: &RoutedKey) -> usize {
    let parent = tree
        .as_ubtree()
        .parent(tree.find_node(removed).unwrap())
        .unwrap();
    std::ptr::from_ref(parent).addr()
}
fn assert_routed_survivor(
    tree: &EBTree<RoutedKey, RemoveBounds>,
    survivor: &RoutedKey,
    address: usize,
) {
    let retained = tree.find_node(survivor).unwrap();
    assert_eq!(std::ptr::from_ref(retained).addr(), address);
    assert_eq!(retained.object(), survivor);
}

#[test]
fn changed_hash_route_cannot_leave_survivor_index_in_released_slot() {
    hash_failure(2);
}
