//! Invalid hash stability is a logic error, never permission for unsafe graph traversal.
use rustycad_tkernel::ncollection::{HashPolicy, IndexedDataMap, IndexedMap};
use std::{cell::Cell, rc::Rc};
struct ShiftPolicy(Rc<Cell<usize>>);
impl HashPolicy<i32> for ShiftPolicy {
    fn hash(&self, key: &i32) -> usize {
        *key as usize + self.0.get()
    }
    fn equal(&self, first: &i32, second: &i32) -> bool {
        first == second
    }
}
#[test]
fn changed_hash_route_cannot_make_indexed_set_removal_dereference_null() {
    let shift = Rc::new(Cell::new(0));
    let mut map = IndexedMap::with_hasher(ShiftPolicy(shift.clone()), 101);
    map.add(1);
    map.add(2);
    shift.set(9);
    map.remove_last().unwrap();
    assert_eq!((map.len(), map.find_key(1)), (1, Ok(&1)));
    map.substitute(1, 7).unwrap();
    assert_eq!(map.find_index(&7), 1);
    map.remove_last().unwrap();
    assert!(map.is_empty());
}
#[test]
fn changed_hash_route_cannot_make_indexed_data_removal_dereference_null() {
    let shift = Rc::new(Cell::new(0));
    let mut map = IndexedDataMap::with_hasher(ShiftPolicy(shift.clone()), 101);
    map.add(1, 10);
    map.add(2, 20);
    shift.set(9);
    map.remove_last().unwrap();
    assert_eq!((map.len(), map.find_from_index(1)), (1, Ok(&10)));
    map.substitute(1, 7, 70).unwrap();
    assert_eq!(map.find_index(&7), 1);
    map.remove_last().unwrap();
    assert!(map.is_empty());
}
#[derive(Debug)]
struct MutableKey(Cell<usize>);
struct MutablePolicy;
impl HashPolicy<MutableKey> for MutablePolicy {
    fn hash(&self, key: &MutableKey) -> usize {
        key.0.get()
    }
    fn equal(&self, first: &MutableKey, second: &MutableKey) -> bool {
        first.0.get() == second.0.get()
    }
}
#[test]
fn interior_key_mutation_keeps_owned_node_reachable_for_safe_removal() {
    let mut map = IndexedDataMap::with_hasher(MutablePolicy, 101);
    map.add(MutableKey(Cell::new(1)), 10);
    map.find_key(1).unwrap().0.set(9);
    map.remove_last().unwrap();
    assert!(map.is_empty());
    assert!(
        map.observer()
            .snapshot()
            .buckets
            .iter()
            .all(|head| *head == 0)
    );
}
