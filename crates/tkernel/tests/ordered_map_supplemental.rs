//! Linked order survives slot reuse and destination policy ownership.
use rustycad_tkernel::ncollection::{BaseAllocator, HashPolicy, OrderedMap, OrderedMapIter};
#[derive(Clone)]
struct Policy(usize);
impl HashPolicy<i32> for Policy {
    fn hash(&self, _key: &i32) -> usize {
        self.0
    }
    fn equal(&self, left: &i32, right: &i32) -> bool {
        left == right
    }
}
#[test]
fn linked_nodes_keep_addresses_and_reused_slots_append_at_tail() {
    let mut map = OrderedMap::<i32>::default();
    let address = map.added(10) as *const i32;
    map.add(20);
    map.add(30);
    assert!(map.remove(&20));
    map.add(40);
    map.resize(1000);
    assert_eq!(map.contained(&10).unwrap() as *const i32, address);
    assert_eq!(map.iter().copied().collect::<Vec<_>>(), [10, 30, 40]);
    assert!(map.iter() == map.iter().clone());
    let mut cursor = map.iter();
    while cursor.more() {
        cursor.advance();
    }
    assert!(cursor.key().is_err());
    assert!(cursor == OrderedMapIter::default());
}
#[test]
fn assignment_and_move_preserve_allocator_ownership_and_expected_policy() {
    let arena = BaseAllocator::incremental(128);
    let mut source = OrderedMap::with_hasher(Policy(42), 10);
    source.add(30);
    source.add(10);
    let mut destination = OrderedMap::with_allocator(Policy(99), 1, arena.clone());
    destination.assign(&source);
    assert_eq!(destination.hasher().0, 99);
    assert_eq!(destination.allocator(), &arena);
    assert_eq!(destination.iter().copied().collect::<Vec<_>>(), [30, 10]);
    let mut moved = OrderedMap::move_from(&mut destination);
    assert!(destination.is_empty());
    assert_eq!(destination.allocator(), &arena);
    assert_eq!(moved.allocator(), &arena);
    assert_eq!(destination.bucket_count(), moved.bucket_count());
    moved.clear(true);
    arena.reset(true).unwrap();
}
