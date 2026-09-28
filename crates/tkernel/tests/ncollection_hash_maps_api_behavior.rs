//! Behaviour of the hash-map members completed for the NCollection API ledger:
//! BaseMap queries, OCCT-style cursors, allocator-replacing clears, MapAlgo and quirks.
use rustycad_tkernel::ncollection::{
    BaseAllocator, BaseMap, DataMap, DataMapIterator, DoubleMap, DoubleMapIterator, HashKey,
    IndexedDataMap, IndexedDataMapIterator, IndexedMap, IndexedMapIterator, Map, MapIterator,
    buckets_from_int, map_algo, next_map_prime,
};

#[derive(Clone, Default)]
struct Salted(usize);
impl rustycad_tkernel::ncollection::HashPolicy<i32> for Salted {
    fn hash(&self, key: &i32) -> usize {
        (*key as usize).wrapping_add(self.0)
    }
    fn equal(&self, first: &i32, second: &i32) -> bool {
        first == second
    }
}

fn keys_of(map: &Map<i32>) -> Vec<i32> {
    map.iter().copied().collect()
}

#[test]
fn prime_table_wraps_like_native_size_t_arithmetic() {
    assert_eq!(next_map_prime(0), 101);
    assert_eq!(next_map_prime(2038431744), 2038431745);
    assert_eq!(next_map_prime(2038431745), 2038431746);
    assert_eq!(next_map_prime(usize::MAX), 101);
}

#[test]
fn int_bucket_counts_reject_negatives_with_the_value() {
    assert_eq!(buckets_from_int(0).unwrap(), 0);
    let failure = buckets_from_int(-3).unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_OutOfRange");
    assert!(failure.message().contains("-3"));
}

#[test]
fn pointer_keys_hash_their_address() {
    let value = 5_i32;
    let pointer: *const i32 = &value;
    assert_eq!(pointer.occt_hash(), pointer as usize);
    let mut map = Map::<*const i32>::default();
    assert!(map.add(pointer) && map.contains(&pointer));
}

#[test]
fn base_map_queries_narrow_like_native_ints() {
    let mut map = DataMap::<i32, i32>::default();
    assert_eq!((BaseMap::nb_buckets(&map), BaseMap::extent(&map)), (1, 0));
    assert!(BaseMap::is_empty(&map));
    map.bind(1, 2);
    assert_eq!((BaseMap::size(&map), BaseMap::length(&map)), (1, 1));
    assert_eq!(BaseMap::nb_buckets(&map), 101);
    assert!(BaseMap::allocator(&map) == &BaseAllocator::common());
    let mut double = DoubleMap::<i32, i32>::default();
    double.bind(1, 1).unwrap();
    assert_eq!(BaseMap::extent(&double), 1);
}

#[test]
fn data_map_cursor_walks_bucket_order_and_reports_exhaustion() {
    let mut map = DataMap::<i32, i32>::default();
    for key in [3, 104, 1] {
        map.bind(key, key * 10);
    }
    let mut cursor = map.cursor();
    let mut seen = Vec::new();
    while cursor.more() {
        seen.push((*cursor.key().unwrap(), *cursor.value().unwrap()));
        cursor.next();
    }
    let expected: Vec<(i32, i32)> = map.items().map(|(k, v)| (*k, *v)).collect();
    assert_eq!(seen, expected);
    assert_eq!(seen[1..], [(104, 1040), (3, 30)]);
    assert_eq!(
        cursor.value().unwrap_err().exception_type(),
        "Standard_NoSuchObject"
    );
    assert!(cursor.is_equal(&DataMapIterator::default()));
    cursor.reset();
    assert_eq!(cursor.key().unwrap(), &1);
    let other = DataMap::<i32, i32>::default();
    cursor.initialize(&other);
    assert!(!cursor.more());
}

#[test]
fn data_map_mutable_cursor_changes_items_in_place() {
    let mut map = DataMap::<i32, i32>::default();
    map.bind(1, 1);
    map.bind(2, 2);
    let mut cursor = map.cursor_mut();
    while cursor.more() {
        *cursor.change_value().unwrap() += 100;
        cursor.next();
    }
    assert!(cursor.change_value().is_err());
    cursor.reset();
    assert!(cursor.more());
    assert_eq!((map[&1], map[&2]), (101, 102));
    map[&2] = 7;
    assert_eq!(map.seek(&2), Some(&7));
}

#[test]
fn data_map_queries_and_exchange() {
    let mut map = DataMap::<i32, i32>::default();
    assert_eq!(*map.bound(1, 10), 10);
    assert_eq!(*map.bound(1, 20), 20);
    assert_eq!(map.contained(&1), Some((&1, &20)));
    assert_eq!(map.contained(&2), None);
    *map.change_seek(&1).unwrap() = 30;
    let mut out = 0;
    assert!(map.find_into(&1, &mut out) && out == 30);
    assert!(!map.find_into(&2, &mut out) && out == 30);
    let mut other = DataMap::<i32, i32>::default();
    map.exchange(&mut other);
    assert!(map.is_empty() && other.is_bound(&1));
}

#[test]
#[should_panic(expected = "Standard_NoSuchObject")]
fn data_map_index_raises_for_unbound_keys() {
    let map = DataMap::<i32, i32>::default();
    let _ = map[&1];
}

#[test]
fn clears_with_allocator_release_only_on_change() {
    let arena = BaseAllocator::incremental(1024);
    let mut map = DataMap::<i32, i32>::default();
    map.bind(1, 1);
    map.clear_with_allocator(Some(BaseAllocator::common()));
    assert_eq!(map.bucket_count(), 101);
    map.clear_with_allocator(Some(arena.clone()));
    assert!(map.allocator() == &arena && map.is_empty());
    let mut keys = Map::<i32>::default();
    keys.add(1);
    keys.clear_with_allocator(Some(arena.clone()));
    assert!(keys.allocator() == &arena);
    let mut indexed = IndexedDataMap::<i32, i32>::default();
    indexed.add(1, 1);
    indexed.clear_with_allocator(None);
    assert!(indexed.is_empty() && indexed.allocator() == &BaseAllocator::common());
}

#[test]
fn map_cursor_and_default_cursor() {
    let mut map = Map::<i32>::default();
    map.add(4);
    map.add(105);
    let mut cursor = map.cursor();
    let mut seen = Vec::new();
    while cursor.more() {
        assert_eq!(cursor.value().unwrap(), cursor.key().unwrap());
        seen.push(*cursor.key().unwrap());
        cursor.next();
    }
    assert_eq!(seen, keys_of(&map));
    assert!(MapIterator::<i32>::default().is_equal(&cursor));
    assert!(MapIterator::<i32>::default().key().is_err());
}

#[test]
fn indexed_map_cursor_indices_and_extras() {
    let mut map = IndexedMap::<i32>::default();
    assert_eq!(map.try_emplace(7), 1);
    assert_eq!(map.try_emplaced(8), &8);
    assert_eq!(map.contained(&8), Some(&8));
    let mut cursor = map.cursor();
    assert_eq!((cursor.value().unwrap(), cursor.index()), (&7, 1));
    cursor.next();
    cursor.next();
    assert!(!cursor.more() && cursor.index() == 3);
    assert!(cursor.value().is_err());
    let default = IndexedMapIterator::<i32>::default();
    assert_eq!(default.index(), 1);
    assert!(default.is_equal(&cursor));
    assert!(!map.cursor().is_equal(&default));
    map.clear_with_allocator(None);
    assert!(map.is_empty());
}

#[test]
fn indexed_data_map_cursor_and_extras() {
    let mut map = IndexedDataMap::<i32, i32>::default();
    assert!(map.try_bind(1, 10));
    assert!(!map.try_bind(1, 20));
    map.add(2, 20);
    assert_eq!(map.contained(&2), Some((&2, &20)));
    *map.contained_mut(&2).unwrap().1 = 21;
    let cursor = map.cursor();
    assert_eq!((cursor.key().unwrap(), cursor.value().unwrap()), (&1, &10));
    assert!(
        IndexedDataMapIterator::<i32, i32>::default()
            .value()
            .is_err()
    );
    let mut writer = map.cursor_mut();
    writer.next();
    *writer.change_value().unwrap() += 1;
    assert_eq!(writer.index(), 2);
    writer.next();
    assert!(writer.key().is_err());
    for (_, value, index) in map.indexed_items_mut() {
        *value += index as i32;
    }
    assert_eq!((map[1], map[2]), (11, 24));
}

#[test]
fn double_map_are_bound_follows_the_native_first_key_link() {
    let mut map = DoubleMap::<i32, i32>::default();
    map.bind(1, 5).unwrap();
    map.bind(2, 106).unwrap();
    assert!(map.are_bound(&2, &106));
    assert!(!map.are_bound(&1, &5));
    assert!(!map.are_bound(&1, &106));
    assert!(!map.are_bound(&3, &5));
    map.bind(102, 7).unwrap(); // shares key 1's first-key bucket: 102 % 101 == 1
    assert!(map.are_bound(&102, &7));
}

#[test]
fn double_map_cursor_finds_and_clear() {
    let mut map = DoubleMap::<i32, i32>::default();
    map.bind(1, 5).unwrap();
    let mut cursor = map.cursor();
    assert_eq!((cursor.key1().unwrap(), cursor.key2().unwrap()), (&1, &5));
    assert_eq!(cursor.value().unwrap(), &5);
    cursor.next();
    assert!(cursor.key1().is_err() && cursor.is_equal(&DoubleMapIterator::default()));
    cursor.reset();
    assert!(cursor.more());
    let (mut first, mut second) = (0, 0);
    assert!(map.find1_into(&1, &mut second) && map.find2_into(&5, &mut first));
    assert_eq!((first, second), (1, 5));
    let arena = BaseAllocator::incremental(512);
    map.clear_with_allocator(Some(arena.clone()));
    assert!(map.is_empty() && map.allocator() == &arena);
}

#[test]
fn map_algo_in_place_forms_match_native_results() {
    let mut left = Map::<i32>::default();
    let mut right = Map::<i32>::default();
    for key in [1, 2, 3] {
        left.add(key);
    }
    for key in [3, 4] {
        right.add(key);
    }
    assert!(map_algo::has_intersection(&left, &right));
    assert!(!map_algo::is_equal(&left, &right));
    let mut united = left.clone();
    assert!(map_algo::unite(&mut united, &right) && united.len() == 4);
    assert!(!map_algo::unite(&mut united, &right));
    let mut intersected = left.clone();
    assert!(map_algo::intersect(&mut intersected, &right));
    assert_eq!(keys_of(&intersected), [3]);
    let mut subtracted = left.clone();
    assert!(map_algo::subtract(&mut subtracted, &right));
    assert_eq!(subtracted.len(), 2);
    let mut differed = left.clone();
    // {1,2,3} xor {3,4} = {1,2,4}: the extent is unchanged, so native Differ reports false.
    assert!(!map_algo::differ(&mut differed, &right));
    assert_eq!(differed.len(), 3);
    assert!(!map_algo::intersect(&mut Map::<i32>::default(), &right));
}

#[test]
fn map_algo_rebuild_resets_the_hasher_like_native_exchange() {
    let mut map = Map::with_hasher(Salted(1), 1);
    map.add(1);
    map.add(2);
    let mut other = Map::with_hasher(Salted(1), 1);
    other.add(2);
    assert!(map_algo::intersect(&mut map, &other));
    assert_eq!(map.hasher().0, 0);
    let mut sym = Map::with_hasher(Salted(3), 1);
    sym.add(1);
    map_algo::differ(&mut sym, &other);
    assert_eq!(sym.hasher().0, 0);
}
