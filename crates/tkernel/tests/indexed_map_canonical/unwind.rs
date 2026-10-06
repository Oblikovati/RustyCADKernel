use rustycad_tkernel::ncollection::{HashPolicy, IndexedMap, IndexedMapIter};
use std::{cell::Cell, rc::Rc};
#[derive(Clone)]
struct CountPolicy {
    calls: Rc<Cell<usize>>,
    reject: Rc<Cell<usize>>,
}
impl HashPolicy<i32> for CountPolicy {
    fn hash(&self, _: &i32) -> usize {
        let next = self.calls.get() + 1;
        self.calls.set(next);
        assert_ne!(next, self.reject.get(), "injected hash panic");
        0
    }
    fn equal(&self, a: &i32, b: &i32) -> bool {
        a == b
    }
}
fn policy() -> CountPolicy {
    CountPolicy {
        calls: Rc::new(Cell::new(0)),
        reject: Rc::new(Cell::new(usize::MAX)),
    }
}
fn caught(action: impl FnOnce()) -> bool {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(action)).is_err()
}
#[test]
fn owned_add_hashes_once_swap_never_hashes_empty_lookup_never_hashes() {
    let policy = policy();
    let mut map = IndexedMap::with_hasher(policy.clone(), 1);
    assert!(!map.contains(&7));
    assert_eq!(policy.calls.get(), 0);
    map.add(7);
    assert_eq!(policy.calls.get(), 1);
    map.add(8);
    assert_eq!(policy.calls.get(), 2);
    map.swap(1, 2).unwrap();
    assert_eq!(policy.calls.get(), 2);
}
#[test]
fn resize_hash_panic_restores_links_and_live_nodes() {
    let policy = policy();
    let mut map = IndexedMap::with_hasher(policy.clone(), 1);
    for key in 1..=3 {
        map.add(key);
    }
    let before = map.observer().snapshot();
    policy.reject.set(policy.calls.get() + 2);
    assert!(caught(|| map.resize(200)));
    policy.reject.set(usize::MAX);
    assert_eq!(map.observer().snapshot().bucket_table, before.bucket_table);
    for key in 1..=3 {
        assert_eq!(map.find_index(&key), key as usize);
    }
    assert_eq!(
        &map.observer().snapshot().indices[..3],
        &before.indices[..3]
    );
}
#[test]
fn remove_last_hash_panic_restores_cleared_slot() {
    let policy = policy();
    let mut map = IndexedMap::with_hasher(policy.clone(), 1);
    map.add(7);
    let before = map.observer().snapshot();
    policy.reject.set(policy.calls.get() + 1);
    assert!(caught(|| {
        map.remove_last().unwrap();
    }));
    assert_eq!(map.observer().snapshot(), before);
}
fn send_sync<T: Send + Sync>() {}
fn unpin<T: Unpin>() {}
#[derive(Debug)]
struct PinnedKey(std::marker::PhantomPinned);
struct PinnedPolicy;
impl HashPolicy<PinnedKey> for PinnedPolicy {
    fn hash(&self, _: &PinnedKey) -> usize {
        0
    }
    fn equal(&self, _: &PinnedKey, _: &PinnedKey) -> bool {
        true
    }
}
#[test]
fn owning_and_borrowed_auto_traits_preserve_payload_bounds() {
    send_sync::<IndexedMap<i32>>();
    send_sync::<IndexedMapIter<'_, i32>>();
    unpin::<IndexedMap<PinnedKey, PinnedPolicy>>();
    let mut map = IndexedMap::<i32>::default();
    map.add(7);
    map.add(8);
    let mut iter = map.iter();
    let first = iter.next().unwrap();
    let second = iter.next().unwrap();
    assert_eq!((*first, *second), (7, 8));
}
#[test]
fn native_constructor_adapter_resizes_before_constructing_key() {
    let mut map = IndexedMap::<i32>::default();
    let observer = map.observer();
    map.emplace_with(|| {
        assert_ne!(observer.snapshot().bucket_table, 0);
        7
    });
    assert_eq!(map.try_emplace_with(|| 7), 1);
}
