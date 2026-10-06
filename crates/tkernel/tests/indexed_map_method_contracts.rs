//! Supplemental method contracts pinned to NCollection_IndexedMap.hxx.
use rustycad_tkernel::ncollection::{BaseAllocator, DefaultHashPolicy, HashPolicy, IndexedMap};
use std::cell::Cell;
use std::rc::Rc;

struct SaltPolicy(i32);
impl HashPolicy<i32> for SaltPolicy {
    fn hash(&self, key: &i32) -> usize {
        (*key + self.0) as usize
    }
    fn equal(&self, first: &i32, second: &i32) -> bool {
        first == second
    }
}

#[derive(Debug)]
struct CountedKey(i32, Rc<Cell<usize>>);
impl Clone for CountedKey {
    fn clone(&self) -> Self {
        self.1.set(self.1.get() + 1);
        Self(self.0, Rc::clone(&self.1))
    }
}
struct CountedPolicy;
impl HashPolicy<CountedKey> for CountedPolicy {
    fn hash(&self, key: &CountedKey) -> usize {
        key.0 as usize
    }
    fn equal(&self, first: &CountedKey, second: &CountedKey) -> bool {
        first.0 == second.0
    }
}

#[test]
fn indexed_assignment_returns_destination_without_cloning_policy() {
    let arena = BaseAllocator::incremental(4096);
    let mut target = IndexedMap::with_allocator(SaltPolicy(90), 1009, arena.clone());
    target.add(99);
    let mut source = IndexedMap::with_hasher(SaltPolicy(7), 1);
    source.add(3);
    source.add(4);
    let destination = &raw const target;
    assert_eq!(target.assign(&source) as *const _, destination);
    assert_eq!(target.hasher().0, 90);
    assert_eq!(target.allocator(), &arena);
    assert_eq!(target.bucket_count(), 1009);
    assert_eq!(target.iter().copied().collect::<Vec<_>>(), [3, 4]);
    assert_eq!(target.find_index(&4), 2);
}

#[test]
fn indexed_assignment_of_empty_preserves_bucket_modulus() {
    let mut target = IndexedMap::<i32>::new(1009);
    target.add(7);
    let empty = IndexedMap::default();
    assert!(target.assign(&empty).is_empty());
    assert_eq!(target.bucket_count(), 1009);
    assert_eq!(target.add(8), 1);
}

#[test]
fn indexed_move_assignment_exchanges_keys_policies_allocators_and_addresses() {
    let arena = BaseAllocator::incremental(4096);
    let mut source = IndexedMap::with_allocator(SaltPolicy(7), 1, arena.clone());
    source.add(3);
    let mut target = IndexedMap::with_hasher(SaltPolicy(90), 1009);
    target.add(99);
    let source_key = source.find_key(1).unwrap() as *const i32;
    let target_key = target.find_key(1).unwrap() as *const i32;
    let destination = &raw const target;
    assert_eq!(target.move_from(&mut source) as *const _, destination);
    assert_eq!(target.find_key(1).unwrap() as *const i32, source_key);
    assert_eq!(source.find_key(1).unwrap() as *const i32, target_key);
    assert_eq!((target.hasher().0, source.hasher().0), (7, 90));
    assert_eq!((target.bucket_count(), source.bucket_count()), (101, 1009));
    assert_eq!(target.allocator(), &arena);
    assert_eq!((target.find_index(&3), source.find_index(&99)), (1, 1));
}

#[test]
fn indexed_self_assignment_preserves_node_identity() {
    let mut map = IndexedMap::<i32>::default();
    map.add(12);
    let key = map.find_key(1).unwrap() as *const i32;
    let destination = &raw const map;
    assert_eq!(map.assign_self() as *const _, destination);
    assert_eq!(map.find_key(1).unwrap() as *const i32, key);
}

#[test]
fn indexed_added_ref_clones_only_on_miss() {
    let copies = Rc::new(Cell::new(0));
    let borrowed = CountedKey(42, Rc::clone(&copies));
    let mut map = IndexedMap::with_hasher(CountedPolicy, 1);
    let first = map.added_ref(&borrowed) as *const CountedKey;
    assert_eq!(copies.get(), 1);
    assert_eq!(map.added_ref(&borrowed) as *const CountedKey, first);
    assert_eq!(copies.get(), 1);
    let mut other = IndexedMap::with_hasher(CountedPolicy, 1);
    other.assign(&map);
    assert_eq!(copies.get(), 2);
    assert_eq!(other.find_key(1).unwrap().0, 42);
}

#[test]
fn indexed_signed_constructor_rejects_negative_but_resize_clamps() {
    let negative =
        IndexedMap::<i32>::with_int_buckets(DefaultHashPolicy, -7, BaseAllocator::common());
    assert_eq!(
        negative.err().unwrap().exception_type(),
        "Standard_OutOfRange"
    );
    let mut map =
        IndexedMap::<i32>::with_int_buckets(DefaultHashPolicy, 0, BaseAllocator::common()).unwrap();
    assert_eq!(map.bucket_count(), 0);
    map.resize_int(-7);
    assert_eq!(map.bucket_count(), 101);
    map.resize_int(101);
    assert_eq!(map.bucket_count(), 1009);
}

#[test]
fn indexed_signed_indices_reject_negative_zero_and_past_end() {
    let mut map = IndexedMap::<i32>::default();
    map.add(10);
    for index in [-7, 0, 2] {
        assert_eq!(
            map.find_key_int(index).unwrap_err().exception_type(),
            "Standard_OutOfRange"
        );
        assert!(map.substitute_int(index, 20).is_err());
        assert!(map.swap_int(index, 1).is_err());
        assert!(map.swap_int(1, index).is_err());
        assert!(map.remove_from_index_int(index).is_err());
    }
    assert_eq!(map.find_key_int(1), Ok(&10));
}

#[test]
fn indexed_signed_mutations_preserve_dense_indices_and_duplicate_failure() {
    let mut map = IndexedMap::<i32>::default();
    map.add(10);
    map.add(20);
    map.add(30);
    map.substitute_int(1, 11).unwrap();
    assert_eq!(
        map.substitute_int(1, 20).unwrap_err().exception_type(),
        "Standard_DomainError"
    );
    map.swap_int(1, 3).unwrap();
    map.remove_from_index_int(1).unwrap();
    assert_eq!(map.iter().copied().collect::<Vec<_>>(), [11, 20]);
    assert_eq!(map.find_index(&11), 1);
}
