//! Rust unwind extensions complete cleanup without changing native nonpanicking callbacks.
use rustycad_tkernel::ncollection::{BaseAllocator, HashPolicy, IndexedMap};
use std::{cell::Cell, rc::Rc};
#[derive(Debug)]
struct Key {
    value: i32,
    reject_assign: Rc<Cell<bool>>,
    reject_drop: Rc<Cell<bool>>,
    assignments: Rc<Cell<usize>>,
}
impl Clone for Key {
    fn clone(&self) -> Self {
        Self {
            value: self.value,
            reject_assign: self.reject_assign.clone(),
            reject_drop: self.reject_drop.clone(),
            assignments: self.assignments.clone(),
        }
    }
    fn clone_from(&mut self, other: &Self) {
        self.assignments.set(self.assignments.get() + 1);
        self.value = other.value;
        assert!(
            !self.reject_assign.replace(false),
            "injected partial assignment panic"
        );
    }
}
impl Drop for Key {
    fn drop(&mut self) {
        assert!(!self.reject_drop.replace(false), "injected key Drop panic");
    }
}
struct Policy;
impl HashPolicy<Key> for Policy {
    fn hash(&self, key: &Key) -> usize {
        key.value as usize
    }
    fn equal(&self, a: &Key, b: &Key) -> bool {
        a.value == b.value
    }
}
fn key(value: i32) -> Key {
    Key {
        value,
        reject_assign: Rc::new(Cell::new(false)),
        reject_drop: Rc::new(Cell::new(false)),
        assignments: Rc::new(Cell::new(0)),
    }
}
fn caught(action: impl FnOnce()) -> bool {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(action)).is_err()
}
#[test]
fn borrowed_substitute_uses_assignment_and_retains_node() {
    let mut map = IndexedMap::with_hasher(Policy, 1);
    let original = key(7);
    let assignments = original.assignments.clone();
    map.add(original);
    let identity = map.observer().snapshot().indices[0];
    map.substitute_ref(1, &key(9)).unwrap();
    assert_eq!(assignments.get(), 1);
    assert_eq!(map.find_key(1).unwrap().value, 9);
    assert_eq!(map.observer().snapshot().indices[0], identity);
}
#[test]
fn partially_mutating_assignment_panic_clears_owned_graph() {
    let arena = BaseAllocator::incremental(4096);
    let mut map = IndexedMap::with_allocator(Policy, 1, arena.clone());
    let original = key(7);
    original.reject_assign.set(true);
    map.add(original);
    map.add(key(8));
    assert!(caught(|| {
        map.substitute_ref(1, &key(9)).unwrap();
    }));
    assert_eq!(map.len(), 0);
    assert!(arena.reset(false).is_ok());
    assert!(map.observer().snapshot().indices.iter().all(|p| *p == 0));
}
#[test]
fn remove_last_drop_panic_finishes_metadata_and_deallocation() {
    let arena = BaseAllocator::incremental(4096);
    let mut map = IndexedMap::with_allocator(Policy, 1, arena.clone());
    let original = key(7);
    original.reject_drop.set(true);
    map.add(original);
    assert!(caught(|| {
        map.remove_last().unwrap();
    }));
    assert_eq!(map.len(), 0);
    assert!(arena.reset(false).is_ok());
}
#[test]
fn clear_drop_panic_drains_all_chains_then_resumes() {
    let arena = BaseAllocator::incremental(4096);
    let mut map = IndexedMap::with_allocator(Policy, 1, arena.clone());
    let original = key(7);
    original.reject_drop.set(true);
    map.add(original);
    map.add(key(8));
    assert!(caught(|| map.clear(false)));
    assert_eq!(map.len(), 0);
    assert!(arena.reset(false).is_ok());
}
