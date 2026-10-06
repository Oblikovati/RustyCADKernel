#![allow(unsafe_code)]
//! Native BaseDoubleMap and BaseIndexedMap subclass probes define these contracts.
use rustycad_tkernel::ncollection::{
    BaseAllocator, BaseDoubleMapStorage, BaseIndexedMapStorage, BaseMap, MapLinkNode,
};

fn resize_double(map: &mut BaseDoubleMapStorage, extent: usize) {
    let replacement = map.begin_resize(extent).unwrap();
    map.end_resize(replacement);
}
fn resize_indexed(map: &mut BaseIndexedMapStorage, extent: usize) {
    let replacement = map.base().begin_resize(extent).unwrap();
    if replacement.is_resize() {
        map.resize_index_table(replacement.bucket_count()).unwrap();
        map.base_mut().end_resize(replacement);
    }
}

#[test]
fn double_table_pair_is_contiguous_and_zeroed() {
    let mut map = BaseDoubleMapStorage::new(1009, None);
    assert!(map.buckets().is_null() && map.buckets_mut().is_null());
    assert!(map.second_buckets().is_null() && map.second_buckets_mut().is_null());
    assert!(map.bucket_slots().is_empty() && map.second_slots().is_empty());
    assert!(map.resizable() && map.is_empty());
    resize_double(&mut map, 0);
    assert_eq!(map.nb_buckets(), 1009);
    assert_eq!(
        (map.bucket_slots().len(), map.second_slots().len()),
        (1010, 1010)
    );
    // SAFETY: both pointers belong to the one contiguous Standard bucket allocation.
    assert_eq!(
        unsafe { map.second_buckets().offset_from(map.buckets()) },
        1010
    );
    assert!(
        map.bucket_slots()
            .iter()
            .chain(map.second_slots())
            .all(|p| p.is_null())
    );
    let noop = map.begin_resize(0).unwrap();
    assert!(!noop.is_resize());
    assert_eq!(noop.bucket_count(), 101);
    let original = map.buckets();
    map.end_resize(noop);
    assert_eq!(map.buckets(), original);
}

#[test]
fn double_plan_relinks_both_tables_before_commit() {
    let mut node = MapLinkNode::new(std::ptr::null_mut());
    let pointer = std::ptr::from_mut(&mut node);
    let mut map = BaseDoubleMapStorage::new(101, None);
    let mut replacement = map.begin_resize(0).unwrap();
    replacement.bucket_slots_mut()[5] = pointer;
    replacement.second_slots_mut()[7] = pointer;
    map.end_resize(replacement);
    assert_eq!(map.bucket_slots()[5], pointer);
    assert_eq!(map.second_slots()[7], pointer);
    let mut noop = map.begin_resize(0).unwrap();
    assert!(noop.bucket_slots_mut().is_empty() && noop.second_slots_mut().is_empty());
    drop(noop);
    drop(map.begin_resize(102).unwrap());
    assert_eq!(map.bucket_slots()[5], pointer);
}

#[test]
fn double_destroy_clears_second_table_even_with_zero_size() {
    let mut node = MapLinkNode::new(std::ptr::null_mut());
    let pointer = std::ptr::from_mut(&mut node);
    let mut map = BaseDoubleMapStorage::new(101, None);
    resize_double(&mut map, 0);
    map.second_slots_mut()[3] = pointer;
    // SAFETY: no nodes are visited for a zero-size map.
    unsafe { map.destroy(|_, _| panic!("zero-size traversal"), false) };
    assert!(map.second_slots()[3].is_null());
    map.bucket_slots_mut()[4] = pointer;
    map.second_slots_mut()[3] = pointer;
    map.increment();
    let mut visits = 0;
    // SAFETY: the one stack node outlives this destruction walk, and its link is null.
    unsafe {
        map.destroy(
            |current, _| {
                visits += 1;
                assert_eq!(current, pointer);
            },
            false,
        )
    };
    assert_eq!((visits, map.size()), (1, 0));
    assert!(map.bucket_slots()[4].is_null() && map.second_slots()[3].is_null());
    // SAFETY: the prior walk emptied all chains.
    unsafe { map.destroy(|_, _| panic!("duplicate traversal"), true) };
    assert!(map.buckets().is_null() && map.second_buckets().is_null());
    assert_eq!(map.nb_buckets(), 101);
}

#[test]
fn double_exchange_and_count_forwarding_preserve_allocator() {
    let allocator = BaseAllocator::fresh_common();
    let mut map = BaseDoubleMapStorage::new(1009, Some(allocator.clone()));
    resize_double(&mut map, 0);
    let pointers = (map.buckets(), map.second_buckets());
    assert_eq!((map.increment(), map.decrement()), (1, 0));
    let mut other = BaseDoubleMapStorage::new(5, None);
    map.exchange(&mut other);
    assert_eq!((other.buckets(), other.second_buckets()), pointers);
    assert_eq!(other.allocator(), &allocator);
    assert_eq!(other.base().nb_buckets(), 1009);
    assert_eq!(map.nb_buckets(), 5);
    assert!(map.second_buckets().is_null());
}

#[test]
fn independent_index_table_reinitializes_without_hash_storage() {
    let mut node = MapLinkNode::new(std::ptr::null_mut());
    let pointer = std::ptr::from_mut(&mut node);
    let mut map = BaseIndexedMapStorage::new(101, None);
    assert!(map.index_buckets().is_null() && map.index_buckets_mut().is_null());
    assert!(map.index_slots().is_empty() && map.index_slots_mut().is_empty());
    map.clear_index_table(true);
    map.resize_index_table(101).unwrap();
    map.index_slots_mut()[3] = pointer;
    map.resize_index_table(101).unwrap();
    assert!(map.index_slots().iter().all(|pointer| pointer.is_null()));
    assert_eq!(map.index_buckets(), map.index_buckets_mut().cast_const());
}

#[test]
fn index_growth_keeps_prefix_and_zeroes_suffix_then_clear_preserves_table() {
    let mut node = MapLinkNode::new(std::ptr::null_mut());
    let pointer = std::ptr::from_mut(&mut node);
    let mut map = BaseIndexedMapStorage::new(101, None);
    resize_indexed(&mut map, 0);
    map.index_slots_mut()[3] = pointer;
    resize_indexed(&mut map, 102);
    assert_eq!(map.nb_buckets(), 1009);
    assert_eq!(map.index_slots()[3], pointer);
    assert!(map.index_slots()[102..].iter().all(|p| p.is_null()));
    let original = map.index_buckets();
    map.clear_index_table(false);
    assert_eq!(map.index_buckets(), original);
    assert!(map.index_slots().iter().all(|p| p.is_null()));
}

#[test]
fn index_exchange_release_and_checked_allocation_keep_hash_table() {
    let allocator = BaseAllocator::fresh_common();
    let mut map = BaseIndexedMapStorage::new(101, Some(allocator.clone()));
    resize_indexed(&mut map, 0);
    map.base_mut().increment();
    let pointers = (map.index_buckets(), map.base().buckets());
    let mut other = BaseIndexedMapStorage::new(7, None);
    map.exchange(&mut other);
    assert_eq!((other.index_buckets(), other.base().buckets()), pointers);
    assert_eq!((other.size(), other.nb_buckets()), (1, 101));
    assert_eq!(other.allocator(), &allocator);
    assert!(map.index_buckets().is_null());
    other.clear_index_table(true);
    assert!(other.index_buckets().is_null());
    assert_eq!(other.base().buckets(), pointers.1);
    assert!(other.resize_index_table(usize::MAX).is_err());
}

#[test]
fn shrinking_index_storage_preserves_valid_prefix() {
    let mut node = MapLinkNode::new(std::ptr::null_mut());
    let pointer = std::ptr::from_mut(&mut node);
    let mut map = BaseIndexedMapStorage::new(101, None);
    resize_indexed(&mut map, 0);
    map.index_slots_mut()[1] = pointer;
    map.resize_index_table(5).unwrap();
    assert_eq!(map.index_slots().len(), 6);
    assert_eq!(map.index_slots()[1], pointer);
    // Native callers must commit a matching modulus before clearIndexTable; drop releases
    // the allocation without using the temporarily mismatched base modulus.
}

#[test]
fn double_resize_output_pair_stays_owned_on_noop() {
    let mut map = BaseDoubleMapStorage::new(1009, None);
    resize_double(&mut map, 0);
    let mut pending = Some(map.begin_resize(1010).unwrap());
    let pointer = pending.as_mut().unwrap().bucket_slots_mut().as_mut_ptr();
    let mut count = 0;
    assert!(!map.begin_resize_into(0, &mut count, &mut pending).unwrap());
    assert_eq!(count, 101);
    assert_eq!(
        pending.as_mut().unwrap().bucket_slots_mut().as_mut_ptr(),
        pointer
    );
    assert!(
        map.begin_resize_into(2004, &mut count, &mut pending)
            .unwrap()
    );
    assert_eq!(pending.unwrap().bucket_count(), count);
}

#[path = "base_derived_map_storage_contracts/oracle_rows.rs"]
mod oracle_rows;

#[test]
fn native_derived_base_map_probe_rows_match() {
    oracle_rows::compare_all_rows();
}
