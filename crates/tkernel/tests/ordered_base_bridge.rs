//! Native BaseMap queries and the insertion-order and bucket traversals of the ordered maps,
//! read through their public cursors and the `Buckets()` view. The native generic
//! `BaseMap::BasicIterator` the probe also drove is a protected member (plan D1 class B6).
#[path = "ordered_base_bridge/fixture.rs"]
mod fixture;
use fixture::*;
use rustycad_tkernel::ncollection::{
    BaseAllocator, BaseMap, HashPolicy, OrderedDataMap, OrderedMap,
};
#[test]
fn set_base_queries_retain_native_empty_full_and_clear_states() {
    let mut map = Set::new(0);
    queries("set_empty", &map);
    fill_set(&mut map);
    queries("set_full", &map);
    map.resize(200);
    let buckets = map.buckets().map(|table| table.identity());
    map.clear(false);
    queries("set_retained", &map);
    let table = map.buckets().unwrap();
    assert_eq!(table.len(), 1009 + 1);
    native_row(
        "set_zeroed",
        format!(
            "set_zeroed {} {}",
            u8::from(buckets == Some(table.identity())),
            u8::from(bucket_pairs(&table).is_empty())
        ),
    );
    map.clear(true);
    queries("set_released", &map);
}
#[test]
fn data_base_queries_retain_native_empty_full_and_clear_states() {
    let mut map = Map::new(0);
    queries("data_empty", &map);
    fill_map(&mut map);
    queries("data_full", &map);
    map.resize(200);
    map.clear(false);
    queries("data_retained", &map);
    assert!(bucket_pairs(&map.buckets().unwrap()).is_empty());
    map.clear(true);
    queries("data_released", &map);
}
#[test]
fn set_traversals_follow_insertion_order_and_bucket_chains() {
    let mut map = Set::new(101);
    fill_set(&mut map);
    record("set_order", map.iter().map(|key| (key, key)));
    let table = map.buckets().unwrap();
    record(
        "set_buckets",
        bucket_pairs(&table).into_iter().map(|(key, _)| (key, key)),
    );
}
#[test]
fn data_traversals_follow_insertion_order_and_bucket_chains() {
    let mut map = Map::new(101);
    fill_map(&mut map);
    record("data_order", map.items());
    record(
        "data_buckets",
        bucket_pairs(&map.buckets().unwrap()).into_iter(),
    );
}
#[test]
fn set_cursor_value_aliases_its_key() {
    let arena = BaseAllocator::fresh_heap();
    let mut map = Set::with_allocator(BridgeHash, 0, arena.clone());
    fill_set(&mut map);
    let buckets = map.buckets().map(|table| table.identity());
    let cursor = map.iter();
    let aliases = std::ptr::eq(cursor.key().unwrap(), cursor.value().unwrap());
    let first = map.first_node().unwrap();
    native_row(
        "set_identity",
        format!(
            "set_identity {} {} {} {} 1",
            u8::from(aliases),
            u8::from(std::ptr::eq(cursor.key().unwrap(), first.key())),
            u8::from(buckets == map.buckets().map(|table| table.identity())),
            u8::from(BaseMap::allocator(&map) == &arena)
        ),
    );
}
#[test]
fn mutable_data_traversal_edits_only_items_and_keeps_the_table() {
    let mut map = Map::new(101);
    fill_map(&mut map);
    let first = map.first_node().map(|node| *node.key());
    let buckets = map.buckets().map(|table| table.identity());
    for (key, value) in map.items_mut() {
        *value += key;
    }
    record("data_mutated", map.items());
    native_row(
        "data_identity",
        format!(
            "data_identity {} {} 1 1",
            u8::from(first == map.first_node().map(|node| *node.key())),
            u8::from(buckets == map.buckets().map(|table| table.identity()))
        ),
    );
    assert_eq!(map.first_node().unwrap().value(), &44);
}
#[test]
fn bucket_tables_change_on_resize_while_order_stays() {
    let mut set = Set::new(101);
    fill_set(&mut set);
    let (first, buckets) = (
        *set.first().unwrap(),
        set.buckets().map(|table| table.identity()),
    );
    set.resize(200);
    native_row(
        "set_resize",
        format!(
            "set_resize {} {}",
            u8::from(first == *set.first().unwrap()),
            u8::from(buckets != set.buckets().map(|table| table.identity()))
        ),
    );
    let mut map = Map::new(101);
    fill_map(&mut map);
    let (first, buckets) = (
        *map.first().unwrap(),
        map.buckets().map(|table| table.identity()),
    );
    map.resize(200);
    native_row(
        "data_resize",
        format!(
            "data_resize {} {}",
            u8::from(first == *map.first().unwrap()),
            u8::from(buckets != map.buckets().map(|table| table.identity()))
        ),
    );
}
#[test]
fn detached_cursor_reports_a_removed_entry() {
    let arena = BaseAllocator::incremental(1024);
    let mut map = Map::with_allocator(BridgeHash, 101, arena);
    fill_map(&mut map);
    let cursor = map.cursor();
    assert_eq!(cursor.value(&map), Ok(&40));
    map.unbind(&4);
    assert!(cursor.value(&map).is_err());
    assert_eq!(map.first(), Ok(&1));
    map.clear(false);
    assert!(map.items().next().is_none());
    assert!(bucket_pairs(&map.buckets().unwrap()).is_empty());
}
struct OpaqueKey(i32);
struct OpaquePolicy;
impl HashPolicy<OpaqueKey> for OpaquePolicy {
    fn hash(&self, key: &OpaqueKey) -> usize {
        key.0 as usize
    }
    fn equal(&self, a: &OpaqueKey, b: &OpaqueKey) -> bool {
        a.0 == b.0
    }
}
#[test]
fn base_bridges_need_no_payload_debug_clone_or_hasher_clone() {
    let mut set = OrderedMap::with_hasher(OpaquePolicy, 0);
    set.add(OpaqueKey(1));
    assert_eq!(
        (
            BaseMap::extent(&set),
            BaseMap::length(&set),
            BaseMap::size(&set)
        ),
        (1, 1, 1)
    );
    let mut map = OrderedDataMap::with_hasher(OpaquePolicy, 0);
    map.bind(OpaqueKey(1), OpaqueKey(2));
    assert_eq!(BaseMap::size(&map), 1);
    assert!(map.first_node().is_some());
    assert!(map.buckets().is_some());
}
