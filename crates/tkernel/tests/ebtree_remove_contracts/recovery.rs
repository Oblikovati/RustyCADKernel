//! Rust unwind/invalid-hash safety extensions; no throwing-destructor OCCT claim.
use super::fixture::*;
use rustycad_tkernel::ncollection::{BaseAllocator, EBTree, HashKey};
use std::{cell::Cell, rc::Rc};
fn destructor_failure(count: usize) {
    let trace = RemoveTrace::default();
    let allocator = BaseAllocator::incremental(1024);
    let mut tree = populated_with_allocator(&trace, 4, allocator.clone());
    let parent = ancestor_addresses(&tree)[0];
    trace.reset('D', count);
    assert!(caught(|| {
        tree.remove(&1);
    }));
    assert_eq!(trace.events(), "A16,D16,D2,");
    assert_eq!(
        std::ptr::from_ref(tree.find_node(&4).unwrap().bounds()).addr(),
        parent
    );
    assert!(!tree.contains(&1));
    trace.reset('\0', 0);
    tree.clear();
    allocator.reset(true).unwrap();
}
#[test]
fn survivor_destructor_panic_repairs_index_before_resuming() {
    destructor_failure(1);
}
#[test]
fn removed_destructor_panic_repairs_index_before_resuming() {
    destructor_failure(2);
}
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
fn survivor_hash_panic_repairs_index_without_repeating_user_hash() {
    hash_failure(1);
}
#[test]
fn changed_hash_route_cannot_leave_survivor_index_in_released_slot() {
    hash_failure(2);
}

#[test]
fn destructor_and_hash_panics_restore_index_and_resume_first_payload() {
    let trace = RemoveTrace::default();
    let mode = Rc::new(Cell::new(0));
    let mut tree = routed_tree(&trace, &mode);
    let removed = RoutedKey::new(1, &mode);
    let address = routed_parent_address(&tree, &removed);
    trace.reset('D', 1);
    mode.set(1);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| tree.remove(&removed)));
    assert!(
        result
            .unwrap_err()
            .downcast::<String>()
            .unwrap()
            .contains("bounds D")
    );
    mode.set(0);
    assert_routed_survivor(&tree, &RoutedKey::new(4, &mode), address);
}
