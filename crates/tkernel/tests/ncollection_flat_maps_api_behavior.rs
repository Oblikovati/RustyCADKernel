//! Members completing the flat, ordered and packed map APIs: optional lookups, default
//! iterators, signed bucket counts, allocator-switching clears, node traversal, the swapping
//! ordered move assignment and the deprecated packed set algebra.
#![allow(deprecated)] // the deprecated PackedMap members are exercised on purpose
use rustycad_tkernel::ncollection::{
    BaseAllocator, DefaultHashPolicy, FlatDataItems, FlatDataMap, FlatDataValues, FlatMap,
    FlatMapIter, OrderedDataCursor, OrderedDataMap, OrderedMap, OrderedMapIter, PackedMap,
    PackedMapIter,
};

#[test]
fn flat_map_contained_and_move_assign_empty_the_source() {
    let mut map = FlatMap::<i32>::default();
    map.add(3);
    assert_eq!(map.contained(&3), Some(&3));
    assert_eq!(map.contained(&4), None);
    let mut target = FlatMap::<i32>::new(40);
    target.add(1000);
    target.move_assign(&mut map);
    assert_eq!(target.iter().copied().collect::<Vec<_>>(), [3]);
    assert_eq!((map.len(), map.capacity()), (0, 0));
}

#[test]
fn default_flat_iterators_are_exhausted_ends() {
    let map = FlatMap::<i32>::default();
    assert_eq!(FlatMapIter::<i32>::default().next(), None);
    assert!(FlatMapIter::<i32>::default() == map.iter());
    let mut filled = FlatDataMap::<i32, i32>::default();
    filled.bind(1, 2);
    assert!(FlatDataItems::<i32, i32>::default() == filled.items_end());
    assert!(FlatDataValues::<i32, i32>::default() != filled.iter());
}

#[test]
fn flat_data_contained_returns_the_stored_pair() {
    let mut map = FlatDataMap::<i32, i32>::default();
    map.bind(5, 50);
    assert_eq!(map.contained(&5), Some((&5, &50)));
    *map.contained_mut(&5).unwrap().1 = 51;
    assert_eq!(map.find(&5).unwrap(), &51);
    assert!(map.contained(&6).is_none() && map.contained_mut(&6).is_none());
}

#[test]
fn ordered_signed_bucket_counts_reject_negatives_and_keep_zero() {
    let common = BaseAllocator::common;
    let error = OrderedMap::<i32>::with_int_buckets(DefaultHashPolicy, -1, common())
        .err()
        .unwrap();
    assert_eq!(error.exception_type(), "Standard_OutOfRange");
    assert!(error.message().contains("-1"));
    let mut map = OrderedMap::<i32>::with_int_buckets(DefaultHashPolicy, 0, common()).unwrap();
    assert_eq!(map.bucket_count(), 0);
    assert!(map.resize_int(-3).is_err());
    map.resize_int(200).unwrap();
    assert_eq!(map.bucket_count(), 1009);
    let data = OrderedDataMap::<i32, i32>::with_int_buckets(DefaultHashPolicy, -2, common());
    assert!(data.is_err());
}

#[test]
fn ordered_move_assignment_swaps_like_the_native_operator() {
    let mut source = OrderedMap::<i32>::new(1);
    source.add(10);
    source.add(20);
    let mut target = OrderedMap::<i32>::new(3);
    target.add(1000);
    target.move_assign(&mut source);
    assert_eq!(target.iter().copied().collect::<Vec<_>>(), [10, 20]);
    assert_eq!(source.iter().copied().collect::<Vec<_>>(), [1000]);
    let mut pairs = OrderedDataMap::<i32, i32>::new(1);
    pairs.bind(1, 2);
    let mut other = OrderedDataMap::<i32, i32>::new(1);
    other.bind(7, 8);
    other.move_assign(&mut pairs);
    assert_eq!(pairs.find(&7).unwrap(), &8);
}

#[test]
fn clear_with_allocator_adopts_the_new_allocator_or_the_common_one() {
    let arena = BaseAllocator::incremental(256);
    let mut map = OrderedMap::<i32>::new(1);
    map.add(1);
    map.clear_with_allocator(Some(arena.clone()));
    assert!(map.is_empty() && map.allocator() == &arena);
    map.add(2);
    map.clear_with_allocator(None);
    assert!(map.allocator() == &BaseAllocator::common());
    let mut data = OrderedDataMap::<i32, i32>::new(1);
    data.bind(1, 1);
    data.clear_with_allocator(None);
    assert!(data.is_empty());
    arena.reset(true).unwrap();
}

#[test]
fn first_node_walks_insertion_order_and_edits_values() {
    let mut map = OrderedMap::<i32>::new(1);
    for key in [5, 3, 9] {
        map.add(key);
    }
    map.remove(&3);
    let mut keys = Vec::new();
    let mut node = map.first_node();
    while let Some(current) = node {
        keys.push(*current.key());
        node = current.order_next();
    }
    assert_eq!(keys, [5, 9]);
    let mut data = OrderedDataMap::<i32, i32>::new(1);
    data.bind(1, 10);
    data.bind(2, 20);
    let mut head = data.first_node_mut().unwrap();
    *head.change_value() = 11;
    let second = head.order_next().unwrap();
    assert_eq!((second.key(), second.value()), (&2, &20));
    assert_eq!(data.first_node().unwrap().value(), &11);
    assert!(OrderedMap::<i32>::new(1).first_node().is_none());
}

#[test]
fn ordered_cursors_reset_to_the_head_or_stay_exhausted() {
    let mut map = OrderedMap::<i32>::new(1);
    map.add(4);
    map.add(6);
    let mut cursor = map.iter();
    cursor.advance();
    cursor.reset();
    assert_eq!(cursor.key().unwrap(), &4);
    let mut empty = OrderedMapIter::<i32>::default();
    empty.reset();
    assert!(!empty.more());
    let mut data = OrderedDataMap::<i32, i32>::new(1);
    data.bind(1, 2);
    data.bind(3, 4);
    let mut detached = data.cursor();
    detached.advance(&data);
    assert!(detached != data.cursor());
    detached.reset(&data);
    assert!(detached == data.cursor());
    let mut unbound = OrderedDataCursor::default();
    unbound.reset(&data);
    assert!(!unbound.more());
    unbound.initialize(&data);
    assert_eq!(unbound.key(&data).unwrap(), &1);
}

#[test]
fn packed_signed_constructor_and_resize_clamp_like_native() {
    assert_eq!(PackedMap::<i32>::from_int(-5).bucket_count(), 1);
    assert_eq!(PackedMap::<i32>::from_int(0).bucket_count(), 1);
    assert_eq!(PackedMap::<i32>::from_int(9).bucket_count(), 9);
    let mut map = PackedMap::<i32>::from_int(1);
    map.resize_int(-1);
    assert_eq!(map.bucket_count(), 101);
    assert!(!PackedMapIter::<i32>::default().more());
    assert_eq!(PackedMapIter::<i32>::default().next(), None);
}

#[test]
fn deprecated_packed_members_forward_to_the_algorithms() {
    let of = |keys: &[i32]| {
        let mut map = PackedMap::<i32>::default();
        keys.iter().for_each(|key| {
            map.add(*key);
        });
        map
    };
    let (left, right) = (of(&[1, 2, 3]), of(&[2, 3, 4]));
    let sorted = |map: &PackedMap<i32>| {
        let mut keys: Vec<i32> = map.iter().collect();
        keys.sort();
        keys
    };
    let mut result = PackedMap::<i32>::default();
    result.union(&left, &right);
    assert_eq!(sorted(&result), [1, 2, 3, 4]);
    result.intersection(&left, &right);
    assert_eq!(sorted(&result), [2, 3]);
    result.subtraction(&left, &right);
    assert_eq!(sorted(&result), [1]);
    result.difference(&left, &right);
    assert_eq!(sorted(&result), [1, 4]);
    let mut work = left.clone();
    assert!(work.unite(&right) && work.intersect(&right) && work.subtract(&of(&[4])));
    assert!(work.differ(&of(&[9])) && sorted(&work) == [2, 3, 9]);
    assert!(!left.is_equal(&right) && of(&[2]).is_subset(&left));
    assert!(left.has_intersection(&right) && left.contains_map(&of(&[1, 3])));
}
