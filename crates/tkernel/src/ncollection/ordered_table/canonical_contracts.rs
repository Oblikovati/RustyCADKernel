//! White-box joins prove the live container uses native allocation/layout and node links.
use super::*;
use crate::ncollection::{BaseMap, OrderedMapNode};
use std::{cell::RefCell, rc::Rc};
#[derive(Clone, Default)]
struct CanonicalHash(Rc<RefCell<String>>);
impl HashPolicy<i32> for CanonicalHash {
    fn hash(&self, key: &i32) -> usize {
        self.0.borrow_mut().push_str(&format!("{key},"));
        (*key as usize) % 2
    }
    fn equal(&self, left: &i32, right: &i32) -> bool {
        left == right
    }
}
fn native_row(label: &str, observed: String) {
    let expected = include_str!("../../../../../tools/oracle/tkernel_ordered_canonical.expected");
    assert_eq!(
        observed,
        expected
            .lines()
            .find(|line| line.starts_with(&format!("{label} ")))
            .unwrap()
    );
}
fn graph<V, N: StorageNode<i32, V>>(label: &str, map: &OrderedTable<i32, V, CanonicalHash, N>) {
    let mut observed = format!("{label} ");
    let mut previous = std::ptr::null_mut();
    for slot in map.items().map(|(key, _)| map.lookup(key).unwrap()) {
        let node = map.node_pointer(slot);
        unsafe {
            assert_eq!(*N::previous(node), previous);
            let next = (*node).next();
            let key = if next.is_null() { 0 } else { *N::key(next) };
            observed.push_str(&format!("{}:{key},", map.key(slot)));
        }
        previous = node;
    }
    observed.push_str(" 1");
    native_row(label, observed);
}
fn exercise<V: Default, N: StorageNode<i32, V>>(
    prefix: &str,
    map: &mut OrderedTable<i32, V, CanonicalHash, N>,
) {
    for key in [1, 2, 3, 6, 5] {
        map.place(key, V::default, false, false);
    }
    assert!(map.remove(&1));
    map.place(9, V::default, false, false);
    graph(&format!("{prefix}_before"), map);
    let old = map.node_pointer(map.first.unwrap());
    map.hasher.0.borrow_mut().clear();
    map.resize(200);
    native_row(
        &format!("{prefix}_rehash"),
        format!(
            "{prefix}_rehash {} {}",
            map.hasher.0.borrow(),
            u8::from(old == map.node_pointer(map.first.unwrap()))
        ),
    );
    graph(&format!("{prefix}_after"), map);
    map.clear(false);
    native_row(
        &format!("{prefix}_clear"),
        format!(
            "{prefix}_clear {} {}",
            map.resource
                .accumulating_chain()
                .iter()
                .map(|block| block.alloc_count)
                .sum::<i32>(),
            map.bucket_count
        ),
    );
    assert_eq!(map.base.size(), 0);
    assert!(!map.base.buckets().is_null());
}
fn allocation_span<V: Default, N: StorageNode<i32, V>>(
    map: &mut OrderedTable<i32, V, CanonicalHash, N>,
) -> usize {
    let node = map.node_pointer(map.first.unwrap());
    let before = map
        .resource
        .accumulating_block(node.addr())
        .unwrap()
        .free_size;
    map.place(100, V::default, false, false);
    let after = map
        .resource
        .accumulating_block(node.addr())
        .unwrap()
        .free_size;
    assert!(map.remove(&100));
    before - after
}
fn reservation_matches_native<N>(label: &str, span: usize) {
    native_row(label, format!("{label} {}", size_of::<N>()));
    // Exact cursor padding closes the measured +4 byte adaptation for these aligned nodes.
    assert_eq!(span, size_of::<N>());
}
#[test]
fn native_set_storage_layout_allocations_links_and_rehash() {
    let arena = BaseAllocator::accumulating(1024).unwrap();
    let mut map =
        OrderedTable::<i32, (), _, OrderedMapNode<i32>>::new(CanonicalHash::default(), 101, arena);
    let (_, slot) = map.place(4, || (), false, false);
    let node = map.node_pointer(slot);
    let span = allocation_span(&mut map);
    let key = map.key(slot) as *const i32;
    native_row(
        "set_layout",
        format!(
            "set_layout {} {} {} {} {}",
            size_of::<OrderedMapNode<i32>>(),
            size_of::<OrderedMapNode<i32>>(),
            key.addr() - node.addr(),
            key.addr() - node.addr(),
            map.resource
                .accumulating_block(node.addr())
                .unwrap()
                .alloc_count
        ),
    );
    reservation_matches_native::<OrderedMapNode<i32>>("set_reservation", span);
    exercise("set", &mut map);
}
#[test]
fn native_data_storage_layout_allocations_links_and_rehash() {
    let arena = BaseAllocator::accumulating(1024).unwrap();
    let mut map = OrderedTable::<i32, i32, _>::new(CanonicalHash::default(), 101, arena);
    let (_, slot) = map.place(4, || 40, false, false);
    let node = map.node_pointer(slot);
    let span = allocation_span(&mut map);
    native_row(
        "data_layout",
        format!(
            "data_layout {} {} {} {} {}",
            size_of::<OrderedDataMapNode<i32, i32>>(),
            size_of::<OrderedDataMapNode<i32, i32>>(),
            (map.key(slot) as *const i32).addr() - node.addr(),
            (map.value(slot) as *const i32).addr() - node.addr(),
            map.resource
                .accumulating_block(node.addr())
                .unwrap()
                .alloc_count
        ),
    );
    reservation_matches_native::<OrderedDataMapNode<i32, i32>>("data_reservation", span);
    exercise("data", &mut map);
}
#[test]
fn identity_slots_reuse_without_owning_or_relinking_nodes() {
    let mut map = OrderedTable::<i32, i32, _>::new(
        CanonicalHash::default(),
        101,
        BaseAllocator::incremental(512),
    );
    let (_, first) = map.place(1, || 10, false, false);
    let old = map.node_pointer(first);
    let (_, second) = map.place(2, || 20, false, false);
    map.remove(&1);
    let (_, reused) = map.place(3, || 30, false, false);
    assert_eq!(reused, first);
    assert_ne!(old, map.node_pointer(reused));
    assert_eq!(map.next_slot(second), Some(reused));
    assert_eq!(map.slot_of(old), None);
    map.clear(true);
    assert!(map.resource.reset(false).is_ok());
    assert!(map.base.buckets().is_null());
}
