//! Rust unwind extensions complete cleanup without changing native nonpanicking callbacks.
use rustycad_tkernel::ncollection::{HashPolicy, IndexedMap};
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
#[test]
fn borrowed_substitute_uses_assignment_in_place() {
    let mut map = IndexedMap::with_hasher(Policy, 1);
    let original = key(7);
    let assignments = original.assignments.clone();
    map.add(original);
    map.substitute_ref(1, &key(9)).unwrap();
    assert_eq!(assignments.get(), 1);
    assert_eq!(map.find_key(1).unwrap().value, 9);
    assert_eq!(map.find_index(&key(9)), 1);
}
