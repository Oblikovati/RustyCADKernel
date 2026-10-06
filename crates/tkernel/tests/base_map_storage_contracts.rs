#![allow(unsafe_code)]
//! Custom native subclasses exercise these protected base APIs; no direct upstream GTest exists.
use rustycad_tkernel::ncollection::{
    BaseAllocator, BaseMap, BaseMapBucketIterator, BaseMapStorage, MapLinkNode, buckets_from_int,
    next_map_prime,
};
use std::fmt::Write;

fn resize(map: &mut BaseMapStorage, extent: usize) {
    let plan = map.begin_resize(extent).unwrap();
    map.end_resize(plan);
}

#[test]
fn constructor_raw_tables_and_legacy_bool() {
    let mut map = BaseMapStorage::new(5, None);
    assert_eq!(
        (map.nb_buckets(), map.size(), map.extent(), map.length()),
        (5, 0, 0, 0)
    );
    assert!(map.is_empty() && map.resizable() && map.buckets().is_null());
    assert!(map.buckets_mut().is_null() && map.bucket_slots().is_empty());
    assert!(map.bucket_slots_mut().is_empty());
    assert_eq!(map.allocator(), &BaseAllocator::common());
    let legacy = BaseMapStorage::new_legacy(7, false, None);
    assert_eq!(legacy.nb_buckets(), 7);
}

#[test]
fn resize_keeps_computed_noop_modulus_and_owns_uncommitted_table() {
    let mut map = BaseMapStorage::new(1009, None);
    let mut plan = map.begin_resize(0).unwrap();
    assert!(plan.is_resize());
    assert_eq!(
        (plan.bucket_count(), plan.bucket_slots().len()),
        (1009, 1010)
    );
    assert_eq!(plan.buckets(), plan.buckets_mut().cast_const());
    assert!(plan.bucket_slots().iter().all(|pointer| pointer.is_null()));
    let marker = MapLinkNode::new(std::ptr::null_mut());
    plan.bucket_slots_mut()[2] = std::ptr::from_ref(&marker).cast_mut();
    map.end_resize(plan);
    let original = map.buckets();
    let mut noop = map.begin_resize(0).unwrap();
    assert!(!noop.is_resize());
    assert_eq!(noop.bucket_count(), 101);
    assert!(noop.buckets().is_null() && noop.buckets_mut().is_null());
    assert!(noop.bucket_slots().is_empty() && noop.bucket_slots_mut().is_empty());
    map.end_resize(noop);
    assert_eq!(map.buckets(), original);
    drop(map.begin_resize(1010).unwrap());
    assert_eq!(
        map.bucket_slots()[2],
        std::ptr::from_ref(&marker).cast_mut()
    );
}

#[test]
fn legacy_iterator_includes_bucket_and_advances_exhausted_state() {
    let mut map = BaseMapStorage::new(5, None);
    let default = BaseMapBucketIterator::default();
    let mut empty = BaseMapBucketIterator::new(&map);
    // SAFETY: no current node exists, so advance only inspects cursor metadata.
    unsafe { empty.advance() };
    assert_eq!(default, empty);
    resize(&mut map, 0);
    let mut allocated = BaseMapBucketIterator::new(&map);
    let prior = allocated.clone();
    assert_eq!(allocated.bucket_index(), 102);
    assert!(!allocated.more() && allocated.node().is_null());
    assert_ne!(allocated, default);
    // SAFETY: the cursor is exhausted, so no node is dereferenced.
    unsafe { allocated.advance() };
    assert_eq!(allocated.bucket_index(), 103);
    assert_ne!(allocated, prior);
    allocated.reset();
    assert_eq!(allocated, prior);
}

#[test]
fn iterator_and_destroy_cache_links_before_callback() {
    let mut first = MapLinkNode::new(std::ptr::null_mut());
    let first_ptr = std::ptr::from_mut(&mut first);
    let mut second = MapLinkNode::new(first_ptr);
    let second_ptr = std::ptr::from_mut(&mut second);
    let mut last = MapLinkNode::new(std::ptr::null_mut());
    let last_ptr = std::ptr::from_mut(&mut last);
    let mut map = BaseMapStorage::new(5, None);
    resize(&mut map, 0);
    map.bucket_slots_mut()[0] = second_ptr;
    map.bucket_slots_mut()[4] = last_ptr;
    for _ in 0..3 {
        map.increment();
    }
    let mut cursor = BaseMapBucketIterator::default();
    cursor.initialize(&map);
    let mut visited = Vec::new();
    while cursor.more() {
        visited.push((cursor.node(), cursor.bucket_index()));
        // SAFETY: all three stack nodes outlive the cursor and have valid acyclic links.
        unsafe { cursor.advance() };
    }
    assert_eq!(visited, [(second_ptr, 0), (first_ptr, 0), (last_ptr, 4)]);
    destroy_and_assert(&mut map, &[second_ptr, first_ptr, last_ptr]);
}

fn destroy_and_assert(map: &mut BaseMapStorage, expected: &[*mut MapLinkNode]) {
    let mut destroyed = Vec::new();
    // SAFETY: caller supplies the same three live stack nodes from the acyclic traversal above.
    unsafe {
        map.destroy(
            |node, _| {
                destroyed.push(node);
                // Native caches Next before callbacks can mutate or free this node.
                *(*node).next_mut() = std::ptr::null_mut();
            },
            false,
        )
    };
    assert_eq!(destroyed, expected);
    assert_eq!(map.size(), 0);
    assert!(map.bucket_slots().iter().all(|node| node.is_null()));
    // SAFETY: size is zero, so the callback is never invoked and no nodes are dereferenced.
    unsafe { map.destroy(|_, _| panic!("empty map must not destroy a node"), true) };
    assert!(map.buckets().is_null());
    assert_eq!(map.nb_buckets(), 101);
}

#[test]
fn zero_size_destroy_skips_first_table_and_retains_native_links() {
    let mut node = MapLinkNode::new(std::ptr::null_mut());
    let pointer = std::ptr::from_mut(&mut node);
    let mut map = BaseMapStorage::new(1, None);
    resize(&mut map, 0);
    map.bucket_slots_mut()[1] = pointer;
    // SAFETY: size0 means native destroy ignores the first table, independent of link contents.
    unsafe { map.destroy(|_, _| panic!("unexpected node deletion"), false) };
    assert_eq!(map.bucket_slots()[1], pointer);
}

#[test]
fn move_exchange_allocator_and_unsigned_size_rules() {
    let allocator = BaseAllocator::incremental(4096);
    let mut source = BaseMapStorage::new(101, Some(allocator.clone()));
    resize(&mut source, 0);
    source.increment();
    let pointer = source.buckets();
    let mut moved = BaseMapStorage::move_from(&mut source);
    assert_eq!((source.nb_buckets(), source.size()), (0, 0));
    assert!(source.buckets().is_null());
    assert_eq!(source.allocator(), moved.allocator());
    let mut other = BaseMapStorage::new(7, None);
    moved.exchange(&mut other);
    assert_eq!(other.buckets(), pointer);
    assert_eq!((other.size(), other.nb_buckets()), (1, 101));
    assert_eq!(other.allocator(), &allocator);
    assert_eq!(moved.decrement(), usize::MAX);
    assert!(moved.resizable());
    assert_eq!(moved.increment(), 0);
}

#[test]
fn link_prefix_is_nonowning_and_mutable() {
    let mut first = MapLinkNode::new(std::ptr::null_mut());
    let mut link = MapLinkNode::new(std::ptr::null_mut());
    *link.next_mut() = std::ptr::from_mut(&mut first);
    assert_eq!(link.next(), std::ptr::from_mut(&mut first));
    assert_eq!(size_of::<MapLinkNode>(), size_of::<*mut MapLinkNode>());
    assert_eq!(align_of::<MapLinkNode>(), align_of::<*mut MapLinkNode>());
}

#[test]
fn unaddressable_bucket_tables_are_rejected_without_wrapping_allocation() {
    let map = BaseMapStorage::new(usize::MAX, None);
    let result = map.begin_resize(0);
    assert!(result.is_err());
    let message = result.err().unwrap();
    assert_eq!(message.exception_type(), "Standard_OutOfMemory");
    assert!(message.message().contains(&usize::MAX.to_string()));
}

#[path = "base_map_storage_contracts/oracle_rows.rs"]
mod oracle_rows;

#[test]
fn native_base_map_probe_rows_match() {
    oracle_rows::compare_all_rows();
}

#[test]
fn legacy_resize_outputs_clear_second_even_when_noop() {
    let mut map = BaseMapStorage::new(1009, None);
    let mut pending = None;
    let mut count = 0;
    assert!(map.begin_resize_into(0, &mut count, &mut pending).unwrap());
    assert_eq!(count, 1009);
    map.end_resize_legacy(usize::MAX, pending.take().unwrap(), std::ptr::null_mut());
    pending = Some(map.begin_resize(1010).unwrap());
    let previous = pending.as_ref().unwrap().buckets();
    let mut second = map.buckets_mut();
    assert!(
        !map.begin_resize_legacy_into(0, &mut count, &mut pending, &mut second)
            .unwrap()
    );
    assert_eq!(count, 101);
    assert_eq!(pending.as_ref().unwrap().buckets(), previous);
    assert!(second.is_null());
}

#[test]
fn resize_failure_still_sets_native_count_and_obsolete_second() {
    let map = BaseMapStorage::new(usize::MAX, None);
    let mut count = 0;
    let mut pending = None;
    let mut slot = std::ptr::null_mut();
    let mut second = std::ptr::from_mut(&mut slot);
    assert!(
        map.begin_resize_legacy_into(0, &mut count, &mut pending, &mut second)
            .is_err()
    );
    assert_eq!(count, usize::MAX);
    assert!(second.is_null() && pending.is_none());
}
