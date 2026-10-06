//! Native BaseMap inheritance and generic iterator construction over the actual ordered graph.
#![allow(unsafe_code)]
#[path = "ordered_base_bridge/fixture.rs"]
mod fixture;
use fixture::*;
use rustycad_tkernel::ncollection::{
    BaseAllocator, BaseMap, BasicMapIteratorSource, HashPolicy, OrderedDataMap, OrderedMap,
};
#[test]
fn set_base_queries_retain_native_empty_full_and_clear_states() {
    let mut map = Set::new(0);
    queries("set_empty", &map);
    fill_set(&mut map);
    queries("set_full", &map);
    map.resize(200);
    let buckets = map.buckets();
    map.clear(false);
    queries("set_retained", &map);
    let slots = map.basic_bucket_slots().unwrap();
    assert_eq!(slots.len(), 1009 + 1);
    native_row(
        "set_zeroed",
        format!(
            "set_zeroed {} {}",
            u8::from(buckets == map.buckets()),
            u8::from(slots.iter().all(|node| node.is_null()))
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
    assert!(
        map.basic_bucket_slots()
            .unwrap()
            .iter()
            .all(|node| node.is_null())
    );
    map.clear(true);
    queries("data_released", &map);
}
#[test]
fn native_set_iterators_traverse_actual_order_and_bucket_graphs() {
    let mut map = Set::new(101);
    fill_set(&mut map);
    set_order("set_order", SetOrder::new(&map));
    set_buckets("set_buckets", SetBucket::new(&map));
}
#[test]
fn native_data_iterators_traverse_actual_order_and_bucket_graphs() {
    let mut map = Map::new(101);
    fill_map(&mut map);
    data_order("data_order", DataOrder::new(&map));
    data_buckets("data_buckets", DataBucket::new(&map));
}
#[test]
fn native_set_value_aliases_key_in_real_allocator_owned_nodes() {
    let arena = BaseAllocator::fresh_heap();
    let mut map = Set::with_allocator(BridgeHash, 0, arena.clone());
    fill_set(&mut map);
    let buckets = map.buckets_mut();
    let iterator = SetOrder::new(&map);
    // SAFETY: the borrowed iterator holds this live map, and both shared references alias its key.
    let aliases = unsafe { std::ptr::eq(iterator.key(), iterator.value()) };
    let _: &i32 = unsafe { iterator.value() };
    native_row(
        "set_identity",
        format!(
            "set_identity {} {} {} {} 1",
            u8::from(aliases),
            u8::from(iterator.node() == map.basic_first_node()),
            u8::from(buckets.cast_const() == map.buckets()),
            u8::from(BaseMap::allocator(&map) == &arena)
        ),
    );
}
#[test]
fn native_mutable_data_iterator_edits_only_items_and_retains_node_identity() {
    let mut map = Map::new(101);
    fill_map(&mut map);
    let first = map.basic_first_node();
    let buckets = map.buckets_mut();
    let mut iterator = DataMutable::new(&mut map);
    assert_eq!(iterator.node(), first);
    while iterator.more() {
        // SAFETY: the exclusive map borrow and sequential short payload borrows exclude aliases.
        unsafe {
            let key = *iterator.key();
            *iterator.value() += key;
            iterator.advance();
        }
    }
    data_order("data_mutated", DataOrder::new(&map));
    native_row(
        "data_identity",
        format!(
            "data_identity {} {} 1 1",
            u8::from(first == map.basic_first_node()),
            u8::from(buckets.cast_const() == map.buckets())
        ),
    );
    assert_eq!(map.first_node().unwrap().value(), &44);
}
#[test]
fn actual_bucket_tables_change_on_resize_while_nodes_stay_put() {
    let mut set = Set::new(101);
    fill_set(&mut set);
    let (node, buckets) = (set.basic_first_node(), set.buckets());
    set.resize(200);
    native_row(
        "set_resize",
        format!(
            "set_resize {} {}",
            u8::from(node == set.basic_first_node()),
            u8::from(buckets != set.buckets())
        ),
    );
    let mut map = Map::new(101);
    fill_map(&mut map);
    let (node, buckets) = (map.basic_first_node(), map.buckets());
    map.resize(200);
    native_row(
        "data_resize",
        format!(
            "data_resize {} {}",
            u8::from(node == map.basic_first_node()),
            u8::from(buckets != map.buckets())
        ),
    );
}
#[test]
fn mutable_bucket_accessor_projects_writable_table_without_copying() {
    let mut map = Map::new(0);
    assert!(map.buckets_mut().is_null());
    fill_map(&mut map);
    let buckets = map.buckets_mut();
    // SAFETY: bucket zero remains null; writing the same pointer preserves graph and registry invariants.
    unsafe {
        assert!((*buckets).is_null());
        *buckets = std::ptr::null_mut();
    }
    assert_eq!(
        buckets.cast_const(),
        map.basic_bucket_slots().unwrap().as_ptr()
    );
    data_order("data_order", DataOrder::new(&map));
}
#[test]
fn native_bridge_coexists_with_safe_cursor_and_allocator_validation() {
    let arena = BaseAllocator::incremental(1024);
    let mut map = Map::with_allocator(BridgeHash, 101, arena.clone());
    fill_map(&mut map);
    let cursor = map.cursor();
    let first = map.basic_first_node();
    assert_eq!(DataOrder::new(&map).node(), first);
    assert_eq!(cursor.value(&map), Ok(&40));
    map.unbind(&4);
    assert!(cursor.value(&map).is_err());
    assert_ne!(map.basic_first_node(), first);
    assert!(arena.reset(false).is_err());
    map.clear(false);
    assert!(arena.reset(false).is_ok());
    assert!(!DataOrder::new(&map).more());
    assert!(!DataBucket::new(&map).more());
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
    assert!(!map.basic_first_node().is_null());
    assert_eq!(map.basic_bucket_slots().unwrap().as_ptr(), map.buckets());
}
