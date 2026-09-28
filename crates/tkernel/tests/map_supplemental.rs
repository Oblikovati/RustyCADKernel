//! Stable key replacement and injected policy ownership beyond the legacy examples.
use rustycad_tkernel::ncollection::{BaseAllocator, HashPolicy, Map};
#[derive(Debug, Clone)]
struct Key(i32, &'static str);
#[derive(Clone)]
struct Policy(usize);
impl HashPolicy<Key> for Policy {
    fn hash(&self, key: &Key) -> usize {
        (key.0 as usize).wrapping_add(self.0)
    }
    fn equal(&self, left: &Key, right: &Key) -> bool {
        left.0 == right.0
    }
}
#[test]
fn equivalent_keys_replace_only_on_emplacement_and_keep_their_address() {
    let mut map = Map::with_hasher(Policy(0), 1);
    let address = map.added(Key(1, "first")) as *const Key;
    assert!(!map.add(Key(1, "ignored")));
    assert!(!map.try_emplace(Key(1, "ignored")));
    assert_eq!(map.try_emplaced(Key(1, "ignored")).1, "first");
    assert_eq!(map.emplaced(Key(1, "replacement")) as *const Key, address);
    assert_eq!(map.contained(&Key(1, "lookup")).unwrap().1, "replacement");
    map.resize(1000);
    assert_eq!(
        map.contained(&Key(1, "lookup")).unwrap() as *const Key,
        address
    );
    let mut iterator = map.iter();
    let copy = iterator.clone();
    assert!(iterator == copy);
    iterator.next();
    assert!(iterator == map.end());
}
#[test]
fn assignment_keeps_policy_and_exchange_moves_policy_and_allocator() {
    let arena = BaseAllocator::incremental(128);
    let mut source = Map::with_hasher(Policy(42), 1);
    source.add(Key(1, "source"));
    let mut destination = Map::with_allocator(Policy(99), 1, arena.clone());
    destination.assign(&source);
    assert_eq!(destination.hasher().0, 99);
    assert_eq!(destination.allocator(), &arena);
    assert!(destination.contains(&Key(1, "lookup")));
    destination.exchange(&mut source);
    assert_eq!(source.hasher().0, 99);
    assert_eq!(source.allocator(), &arena);
    assert_eq!(destination.hasher().0, 42);
    source.clear(false);
    arena.reset(true).unwrap();
}
