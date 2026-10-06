use super::fixture::{caught, keys, populated};
use rustycad_tkernel::ncollection::{BaseAllocator, HashPolicy, Map};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
#[derive(Clone, Debug, Default)]
struct KeyDropTrace {
    events: Rc<RefCell<Vec<i32>>>,
    reject: Rc<Cell<Option<i32>>>,
}
#[derive(Debug)]
struct OwnedKey {
    identity: i32,
    payload: i32,
    trace: KeyDropTrace,
}
impl Drop for OwnedKey {
    fn drop(&mut self) {
        self.trace.events.borrow_mut().push(self.payload);
        if self.trace.reject.get() == Some(self.payload) {
            self.trace.reject.set(None);
            panic!(
                "key payload {} rejected; expected accepted key",
                self.payload
            );
        }
    }
}
impl KeyDropTrace {
    fn key(&self, identity: i32, payload: i32) -> OwnedKey {
        OwnedKey {
            identity,
            payload,
            trace: self.clone(),
        }
    }
}
struct KeyHash;
impl HashPolicy<OwnedKey> for KeyHash {
    fn hash(&self, _: &OwnedKey) -> usize {
        0
    }
    fn equal(&self, a: &OwnedKey, b: &OwnedKey) -> bool {
        a.identity == b.identity
    }
}
fn dropping_map(trace: &KeyDropTrace, arena: &BaseAllocator) -> Map<OwnedKey, KeyHash> {
    let mut map = Map::with_allocator(KeyHash, 1, arena.clone());
    for key in 1..=3 {
        map.add(trace.key(key, key));
    }
    map
}
#[test]
fn hasher_panic_keeps_old_links_and_stable_keys() {
    let mut map = populated(1);
    let table = map.buckets();
    let key = std::ptr::from_ref(map.contained(&2).unwrap());
    map.hasher().rejected.set(Some(2));
    assert!(caught(|| map.resize(200)));
    map.hasher().rejected.set(None);
    assert_eq!(
        (map.len(), keys(&map), map.bucket_count()),
        (3, "3,2,1,".into(), 101)
    );
    assert_eq!(map.buckets(), table);
    assert_eq!(unsafe { *key }, 2);
    map.resize(200);
    assert_eq!(keys(&map), "1,2,3,");
}
#[test]
fn clear_drop_panic_drains_every_map_node_and_restores_count() {
    let trace = KeyDropTrace::default();
    let arena = BaseAllocator::incremental(512);
    let mut map = dropping_map(&trace, &arena);
    map.resize(200);
    trace.reject.set(Some(1));
    assert!(caught(|| map.clear(false)));
    assert_eq!(*trace.events.borrow(), [1, 2, 3]);
    assert_eq!(map.len(), 0);
    assert!(!map.buckets().is_null());
    assert!(arena.reset(false).is_ok());
    map.add(trace.key(7, 7));
    assert_eq!(map.len(), 1);
}
#[test]
fn removal_drop_panic_unlinks_real_node_and_decrements_count() {
    let trace = KeyDropTrace::default();
    let arena = BaseAllocator::incremental(512);
    let mut map = dropping_map(&trace, &arena);
    let lookup = trace.key(2, 200);
    trace.reject.set(Some(2));
    assert!(caught(|| {
        map.remove(&lookup);
    }));
    assert_eq!(map.len(), 2);
    assert!(!map.contains(&lookup));
    assert_eq!(*trace.events.borrow(), [2]);
    map.clear(true);
    assert_eq!(*trace.events.borrow(), [2, 3, 1]);
    assert!(arena.reset(false).is_ok());
}
#[test]
fn owner_drop_panic_releases_every_key_allocation_once() {
    let trace = KeyDropTrace::default();
    let arena = BaseAllocator::incremental(512);
    let map = dropping_map(&trace, &arena);
    trace.reject.set(Some(3));
    assert!(caught(|| drop(map)));
    assert_eq!(*trace.events.borrow(), [3, 2, 1]);
    assert!(arena.reset(false).is_ok());
}
#[test]
fn emplace_drop_panic_leaves_replacement_key_initialized() {
    let trace = KeyDropTrace::default();
    let arena = BaseAllocator::incremental(512);
    let mut map = dropping_map(&trace, &arena);
    trace.reject.set(Some(2));
    assert!(caught(|| {
        map.emplace(trace.key(2, 20));
    }));
    assert_eq!(map.len(), 3);
    assert_eq!(
        map.iter().find(|key| key.identity == 2).unwrap().payload,
        20
    );
    map.clear(true);
    assert_eq!(*trace.events.borrow(), [2, 3, 20, 1]);
    assert!(arena.reset(false).is_ok());
}
#[test]
fn replacement_allocator_clears_actual_old_nodes_first() {
    let first = BaseAllocator::incremental(512);
    let second = BaseAllocator::incremental(512);
    let mut map = Map::<i32>::with_allocator(Default::default(), 2003, first.clone());
    map.add(7);
    map.clear_with_allocator(Some(second.clone()));
    assert_eq!(map.allocator(), &second);
    assert!(map.buckets().is_null());
    assert_eq!(map.bucket_count(), 2003);
    assert!(first.reset(false).is_ok());
    map.add(9);
    assert!(second.reset(false).is_err());
    map.clear(true);
    assert!(second.reset(false).is_ok());
}
#[test]
fn rust_clone_from_keeps_replacement_semantics_and_source_resource() {
    let first = BaseAllocator::incremental(512);
    let second = BaseAllocator::incremental(512);
    let mut source = Map::<i32>::with_allocator(Default::default(), 1009, first.clone());
    source.add(7);
    let mut target = Map::<i32>::with_allocator(Default::default(), 2003, second.clone());
    target.add(9);
    target.clone_from(&source);
    assert_eq!(target.iter().copied().collect::<Vec<_>>(), [7]);
    assert_eq!((target.bucket_count(), target.allocator()), (1009, &first));
    assert!(second.reset(false).is_ok());
}
#[test]
fn same_allocator_clear_keeps_real_bucket_table() {
    let arena = BaseAllocator::incremental(512);
    let mut map = Map::<i32>::with_allocator(Default::default(), 1, arena.clone());
    map.add(7);
    let table = map.buckets();
    map.clear_with_allocator(Some(arena.clone()));
    assert_eq!(table, map.buckets());
    assert_eq!(map.len(), 0);
    assert!(arena.reset(false).is_ok());
}
