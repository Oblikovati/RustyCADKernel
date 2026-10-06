//! Canonical graph ownership, disjoint borrows, safe cursor adaptation and Rust unwind guarantees.
use rustycad_tkernel::ncollection::{
    BaseAllocator, DefaultHashPolicy, HashPolicy, OrderedDataMap, OrderedMap,
};
use std::{cell::Cell, rc::Rc};
#[path = "fixtures/native_map_node_trace.rs"]
mod trace;
use trace::{NodeItem, NodeTrace, node_caught};
struct TraceHash;
impl HashPolicy<NodeItem> for TraceHash {
    fn hash(&self, key: &NodeItem) -> usize {
        key.kind as usize
    }
    fn equal(&self, left: &NodeItem, right: &NodeItem) -> bool {
        left.kind == right.kind
    }
}
#[test]
fn mutable_iteration_keeps_every_previously_yielded_pair_live() {
    let mut map = OrderedDataMap::<i32, i32>::default();
    for key in 0..20 {
        map.bind(key, key * 10);
    }
    let mut pairs = map.items_mut().collect::<Vec<_>>();
    for (key, value) in &mut pairs {
        **value += **key;
    }
    assert_eq!(
        pairs
            .iter()
            .map(|(key, value)| (**key, **value))
            .collect::<Vec<_>>(),
        (0..20).map(|key| (key, key * 11)).collect::<Vec<_>>()
    );
}
#[test]
fn mutable_views_project_disjoint_key_and_value_fields() {
    let mut map = OrderedDataMap::<i32, i32>::default();
    map.bind(1, 10);
    map.bind(2, 20);
    let (key, value) = map.contained_mut(&1).unwrap();
    *value += *key;
    let mut view = map.first_node_mut().unwrap();
    assert_eq!(view.key(), &1);
    *view.change_value() = 12;
    let mut next = view.order_next().unwrap();
    assert_eq!(next.key(), &2);
    *next.change_value() = 21;
    assert_eq!(
        map.items().map(|(k, v)| (*k, *v)).collect::<Vec<_>>(),
        [(1, 12), (2, 21)]
    );
}
#[test]
fn detached_cursor_rejects_erasure_foreign_owner_and_recycled_identity_slot() {
    let arena = BaseAllocator::incremental(512);
    let mut map = OrderedDataMap::with_allocator(DefaultHashPolicy, 101, arena.clone());
    map.bind(1, 10);
    let mut cursor = map.cursor();
    map.unbind(&1);
    assert!(cursor.value(&map).is_err());
    map.bind(2, 20);
    assert!(cursor.value(&map).is_err());
    cursor.initialize(&map);
    let other = OrderedDataMap::<i32, i32>::default();
    assert!(cursor.value(&other).is_err());
    cursor.reset(&map);
    assert_eq!(cursor.value(&map), Ok(&20));
    drop(map);
    assert!(arena.reset(false).is_ok());
}
#[test]
fn clear_finishes_all_native_payloads_and_allocations_before_resuming_panic() {
    let trace = NodeTrace::new();
    let mut map = OrderedDataMap::with_allocator(TraceHash, 101, trace.arena.clone());
    map.bind(trace.item('A'), trace.item('V'));
    map.bind(trace.item('B'), trace.item('W'));
    trace.clear();
    trace.reject_drop.set('A');
    assert!(node_caught(|| map.clear(false)));
    assert!(map.is_empty());
    assert!(!trace.live());
    let events = trace.events();
    assert!(events.contains("DA,DV,"));
    assert!(events.contains("DB,DW,"));
    map.bind(trace.item('C'), trace.item('X'));
    map.clear(true);
    assert!(!trace.live());
}
#[test]
fn remove_unlinks_every_native_chain_before_payload_panic() {
    let trace = NodeTrace::new();
    let key = trace.item('A');
    let mut map = OrderedDataMap::with_allocator(TraceHash, 101, trace.arena.clone());
    map.bind(trace.item('A'), trace.item('V'));
    map.bind(trace.item('B'), trace.item('W'));
    trace.reject_drop.set('A');
    assert!(node_caught(|| {
        map.unbind(&key);
    }));
    assert_eq!(map.len(), 1);
    assert_eq!(map.first().unwrap().kind, 'B');
    assert_eq!(map.items().count(), 1);
    map.clear(false);
    assert!(!trace.live());
}
#[derive(Clone)]
struct FailingHash(Rc<Cell<Option<i32>>>);
impl HashPolicy<i32> for FailingHash {
    fn hash(&self, key: &i32) -> usize {
        assert_ne!(
            self.0.get(),
            Some(*key),
            "rejected key {key}; expected an enabled hash callback"
        );
        (*key as usize) % 2
    }
    fn equal(&self, left: &i32, right: &i32) -> bool {
        left == right
    }
}
#[test]
fn resize_hash_panic_restores_original_bucket_graph_without_rehashing() {
    let reject = Rc::new(Cell::new(None));
    let mut map = OrderedMap::with_hasher(FailingHash(Rc::clone(&reject)), 101);
    for key in [4, 1, 2, 3, 6, 5] {
        map.add(key);
    }
    let first = std::ptr::from_ref(map.first().unwrap());
    reject.set(Some(2));
    assert!(node_caught(|| map.resize(200)));
    reject.set(None);
    assert_eq!(map.bucket_count(), 101);
    assert_eq!(map.iter().copied().collect::<Vec<_>>(), [4, 1, 2, 3, 6, 5]);
    for key in [4, 1, 2, 3, 6, 5] {
        assert!(map.contains(&key));
    }
    map.resize(200);
    assert_eq!(first, std::ptr::from_ref(map.first().unwrap()));
}
#[test]
fn allocator_change_releases_old_native_nodes_and_adopts_new_provenance() {
    let old = BaseAllocator::incremental(512);
    let new = BaseAllocator::incremental(512);
    let mut map = OrderedMap::with_allocator(DefaultHashPolicy, 101, old.clone());
    map.add(1);
    assert!(old.reset(false).is_err());
    map.clear_with_allocator(Some(new.clone()));
    assert!(old.reset(false).is_ok());
    map.add(2);
    assert!(new.reset(false).is_err());
    map.clear(false);
    assert!(new.reset(false).is_ok());
    assert_eq!(map.allocator(), &new);
}
fn send_sync<T: Send + Sync>() {}
fn unpin<T: Unpin>() {}
#[test]
fn map_and_borrowed_iterator_auto_traits_match_owned_payloads() {
    use rustycad_tkernel::ncollection::{
        OrderedDataItems, OrderedDataItemsMut, OrderedDataValues, OrderedDataValuesMut,
        OrderedMapIter, OrderedNodeMut, OrderedNodeRef,
    };
    send_sync::<OrderedMap<i32>>();
    send_sync::<OrderedDataMap<i32, i32>>();
    send_sync::<OrderedMapIter<'_, i32>>();
    send_sync::<OrderedDataItems<'_, i32, i32>>();
    send_sync::<OrderedDataItemsMut<'_, i32, i32>>();
    send_sync::<OrderedDataValues<'_, i32, i32>>();
    send_sync::<OrderedDataValuesMut<'_, i32, i32>>();
    send_sync::<OrderedNodeRef<'_, i32, i32>>();
    send_sync::<OrderedNodeMut<'_, i32, i32>>();
    unpin::<OrderedMap<std::marker::PhantomPinned>>();
    unpin::<OrderedDataMap<std::marker::PhantomPinned, std::marker::PhantomPinned>>();
}
fn shorten_map<'a>(
    map: OrderedMap<&'static str, StrHash>,
    _: &'a str,
) -> OrderedMap<&'a str, StrHash> {
    map
}
fn shorten_data<'a>(
    map: OrderedDataMap<&'static str, &'static str, StrHash>,
    _: &'a str,
) -> OrderedDataMap<&'a str, &'a str, StrHash> {
    map
}
fn shorten_iterator<'a>(
    iter: rustycad_tkernel::ncollection::OrderedMapIter<'a, &'static str>,
) -> rustycad_tkernel::ncollection::OrderedMapIter<'a, &'a str> {
    iter
}
#[test]
fn native_graph_ownership_and_shared_iteration_remain_covariant() {
    let owned = String::from("short");
    let mut map = OrderedMap::with_hasher(StrHash, 101);
    map.add("key");
    assert_eq!(shorten_iterator(map.iter()).next(), Some(&"key"));
    let map = shorten_map(map, &owned);
    assert_eq!(map.first(), Ok(&"key"));
    let mut bindings = OrderedDataMap::with_hasher(StrHash, 101);
    bindings.bind("key", "value");
    assert_eq!(shorten_data(bindings, &owned).seek(&"key"), Some(&"value"));
}

struct StrHash;
impl HashPolicy<&str> for StrHash {
    fn hash(&self, key: &&str) -> usize {
        key.len()
    }
    fn equal(&self, left: &&str, right: &&str) -> bool {
        left == right
    }
}

macro_rules! assert_not {
    ($ty:ty,$bound:path) => {{
        trait Ambiguous<A> {}
        impl<T: ?Sized> Ambiguous<()> for T {}
        struct Forbidden;
        impl<T: ?Sized + $bound> Ambiguous<Forbidden> for T {}
        fn check<T: ?Sized + Ambiguous<A>, A>() {}
        check::<$ty, _>();
    }};
}
#[test]
fn canonical_ownership_never_grants_forbidden_payload_thread_traits() {
    use rustycad_tkernel::ncollection::{OrderedDataItemsMut, OrderedMapIter, OrderedNodeMut};
    assert_not!(OrderedMap<Rc<i32>>, Send);
    assert_not!(OrderedMap<Cell<i32>>, Sync);
    assert_not!(OrderedDataMap<i32,Rc<i32>>,Send);
    assert_not!(OrderedDataMap<i32,Cell<i32>>,Sync);
    assert_not!(OrderedMapIter<'static, Cell<i32>>, Send);
    assert_not!(OrderedDataItemsMut<'static, Cell<i32>, i32>, Send);
    assert_not!(OrderedNodeMut<'static, i32, Rc<i32>>, Send);
    assert_not!(OrderedDataMap<i32,i32,Rc<i32>>,Send);
    assert_not!(OrderedMap<i32,std::marker::PhantomPinned>,Unpin);
    assert_not!(OrderedDataMap<i32,i32,std::marker::PhantomPinned>,Unpin);
}
