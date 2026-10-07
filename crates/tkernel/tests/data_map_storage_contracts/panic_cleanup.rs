use super::fixture::{CollisionHash, DropTrace, DropValue, caught, keys, populated, table_of};
use rustycad_tkernel::ncollection::{BaseAllocator, DataMap, HashPolicy};
fn dropping_map(
    trace: &DropTrace,
    arena: &BaseAllocator,
) -> DataMap<i32, DropValue, CollisionHash> {
    let mut map = DataMap::with_allocator(CollisionHash::default(), 1, arena.clone());
    for key in 1..=3 {
        map.bind(key, trace.value(key));
    }
    map
}
#[test]
fn hash_panic_leaves_old_canonical_chains_intact() {
    let mut map = populated(1);
    let table = table_of(&map);
    map.hasher().rejected.set(Some(2));
    assert!(caught(|| map.resize(200)));
    map.hasher().rejected.set(None);
    assert_eq!(
        (keys(&map), map.len(), map.bucket_count()),
        ("3,2,1,".into(), 3, 101)
    );
    assert_eq!(table, table_of(&map));
    assert_eq!(map.find(&2), Ok(&20));
    map.resize(200);
    assert_eq!(keys(&map), "1,2,3,");
}
#[test]
fn constructor_panic_keeps_existing_value_and_count() {
    let mut map = populated(1);
    assert!(caught(|| {
        map.emplace(2, || panic!("rejected value"));
    }));
    assert_eq!((map.len(), map.find(&2)), (3, Ok(&20)));
}
#[test]
fn unset_drop_panic_unlinks_and_decrements_before_resumption() {
    let trace = DropTrace::default();
    let arena = BaseAllocator::incremental(512);
    let mut map = dropping_map(&trace, &arena);
    trace.reject.set(Some(2));
    assert!(caught(|| {
        map.unbind(&2);
    }));
    assert_eq!((map.len(), keys(&map)), (2, "3,1,".into()));
    assert!(!map.is_bound(&2));
    map.clear(true);
    assert_eq!(*trace.events.borrow(), [2, 3, 1]);
    assert!(arena.reset(false).is_ok());
}
#[test]
fn overwritten_value_drop_panic_keeps_replacement_live() {
    let trace = DropTrace::default();
    let arena = BaseAllocator::incremental(512);
    let mut map = dropping_map(&trace, &arena);
    trace.reject.set(Some(2));
    assert!(caught(|| {
        map.bind(2, trace.value(20));
    }));
    assert_eq!((map.len(), map.find(&2).unwrap().number), (3, 20));
    map.clear(true);
    assert!(arena.reset(false).is_ok());
}
#[test]
fn allocator_replacement_clears_with_old_resource_and_preserves_modulus() {
    let first = BaseAllocator::incremental(512);
    let second = BaseAllocator::incremental(512);
    let mut map = DataMap::<i32, i32>::with_allocator(Default::default(), 2003, first.clone());
    map.bind(1, 7);
    map.clear_with_allocator(Some(second.clone()));
    assert_eq!((map.len(), map.bucket_count()), (0, 2003));
    assert!(table_of(&map).is_none());
    assert_eq!(map.allocator(), &second);
    assert!(first.reset(false).is_ok());
    map.bind(2, 8);
    // Native IncAllocator::Reset never refuses; entries hold no arena lease (plan D1 B6).
    assert!(second.reset(false).is_ok());
    assert_eq!(map.find(&2), Ok(&8));
}
struct NonCloneHash;
impl HashPolicy<i32> for NonCloneHash {
    fn hash(&self, key: &i32) -> usize {
        *key as usize
    }
    fn equal(&self, a: &i32, b: &i32) -> bool {
        a == b
    }
}
#[test]
fn native_assign_does_not_require_copying_the_hasher() {
    let mut source = DataMap::with_hasher(NonCloneHash, 1);
    source.bind(1, 7);
    let mut target = DataMap::with_hasher(NonCloneHash, 1);
    target.assign(&source);
    assert_eq!(target.find(&1), Ok(&7));
}
