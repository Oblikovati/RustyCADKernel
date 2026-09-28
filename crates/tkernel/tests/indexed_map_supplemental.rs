//! Collisions, stable storage, bounds and allocator ownership beyond the legacy cases.
use rustycad_tkernel::ncollection::{BaseAllocator, HashPolicy, IndexedMap};
#[derive(Clone)]
struct CollisionPolicy(usize);
impl HashPolicy<i32> for CollisionPolicy {
    fn hash(&self, _key: &i32) -> usize {
        self.0
    }
    fn equal(&self, left: &i32, right: &i32) -> bool {
        left == right
    }
}
#[test]
fn colliding_indices_keep_stable_nodes_across_swap_remove_and_substitute() {
    let mut map = IndexedMap::with_hasher(CollisionPolicy(42), 1);
    let address = map.added(30) as *const i32;
    map.add(20);
    map.add(10);
    map.swap(1, 3).unwrap();
    assert_eq!(map.find_key(3).unwrap() as *const i32, address);
    map.remove_from_index(1).unwrap();
    assert_eq!(map.find_key(1).unwrap() as *const i32, address);
    assert_eq!(map.find_index(&30), 1);
    assert_eq!(map.find_index(&20), 2);
    assert_eq!(map.find_index(&10), 0);
    map.substitute(2, 25).unwrap();
    map.substitute(2, 25).unwrap();
    assert_eq!(map.find_index(&20), 0);
    assert_eq!(map.find_index(&25), 2);
    map.resize(1000);
    assert_eq!(map.find_key(1).unwrap() as *const i32, address);
    assert!(map.iter() == map.iter().clone());
    assert!(map.indexed_items() == map.indexed_items().clone());
}
#[test]
fn failed_indices_and_duplicate_substitution_leave_keys_unchanged() {
    let mut map = IndexedMap::<i32>::default();
    assert!(map.remove_last().is_err());
    map.add(10);
    map.add(20);
    assert!(map.find_key(0).is_err());
    assert!(map.find_key(3).is_err());
    assert!(map.swap(0, 1).is_err());
    assert!(map.swap(1, 3).is_err());
    assert!(map.substitute(0, 30).is_err());
    assert!(map.substitute(1, 20).is_err());
    assert!(map.remove_from_index(3).is_err());
    map.swap(1, 1).unwrap();
    assert_eq!(map.iter().copied().collect::<Vec<_>>(), [10, 20]);
}
#[test]
fn assignment_keeps_destination_allocator_and_bucket_policy() {
    let arena = BaseAllocator::incremental(128);
    let mut source = IndexedMap::with_hasher(CollisionPolicy(42), 1);
    source.add(30);
    source.add(20);
    let mut destination = IndexedMap::with_allocator(CollisionPolicy(99), 10, arena.clone());
    assert_eq!(destination.bucket_count(), 10);
    destination.assign(&source);
    assert_eq!(destination.hasher().0, 99);
    assert_eq!(destination.allocator(), &arena);
    assert_eq!(destination.find_index(&30), 1);
    assert_eq!(destination.find_index(&20), 2);
    destination.clear(false);
    arena.reset(true).unwrap();
    assert!(destination.is_empty());
    destination.add(40);
    assert_eq!(destination.find_index(&40), 1);
}
