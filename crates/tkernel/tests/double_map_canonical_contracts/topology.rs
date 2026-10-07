use rustycad_tkernel::ncollection::{BaseAllocator, DoubleMap, HashPolicy};
fn first_chains(map: &super::hash_order::OrderedMap) -> Vec<(usize, i32, i32)> {
    let table = map.buckets().unwrap();
    let mut chains = Vec::new();
    for bucket in 0..table.len() {
        chains.extend(table.chain(bucket).unwrap().map(|(a, b)| (bucket, *a, *b)));
    }
    chains
}
#[test]
fn public_table_is_the_key1_table_and_the_cursor_follows_it() {
    let (mut map, _events) = super::hash_order::ordered_map();
    map.bind(1, 5).unwrap();
    map.bind(2, 6).unwrap();
    assert_eq!(map.buckets().unwrap().len(), map.bucket_count() + 1);
    assert_eq!(first_chains(&map), [(2, 1, 5), (3, 2, 6)]);
    let mut cursor = map.cursor();
    assert_eq!((cursor.key1().unwrap(), cursor.key2().unwrap()), (&1, &5));
    assert_eq!(cursor.value_ptr(), map.seek1_ptr(&1));
    cursor.next();
    assert_eq!((cursor.key1().unwrap(), cursor.value().unwrap()), (&2, &6));
}
#[test]
fn bindings_survive_link_rewrites_resize_and_neighbor_removal() {
    let (mut map, _events) = super::hash_order::ordered_map();
    map.bind(1, 5).unwrap();
    map.bind(2, 6).unwrap();
    map.bind(3, 7).unwrap();
    map.resize(200);
    assert!(map.unbind2(&6));
    assert_eq!((map.seek2(&5), map.seek1(&1)), (Some(&1), Some(&5)));
    assert_eq!((map.seek2(&7), map.seek1(&3)), (Some(&3), Some(&7)));
    assert_eq!(first_chains(&map), [(2, 1, 5), (4, 3, 7)]);
}
#[test]
fn clear_with_same_allocator_releases_both_tables_and_keeps_the_allocator() {
    let arena = BaseAllocator::incremental(512);
    let mut map = DoubleMap::<i32, i32>::with_allocator(
        Default::default(),
        Default::default(),
        1,
        arena.clone(),
    );
    map.bind(1_i32, 5_i32).unwrap();
    map.clear_with_allocator(Some(arena.clone()));
    assert!(map.is_empty() && map.buckets().is_none());
    assert_eq!(map.allocator(), &arena);
}
#[test]
fn exchange_and_move_transfer_the_table_with_its_pairs() {
    let mut first = DoubleMap::<i32, i32>::default();
    let mut second = DoubleMap::<i32, i32>::default();
    first.bind(1, 5).unwrap();
    let table = first.buckets().unwrap().identity();
    first.exchange(&mut second);
    assert!(first.is_empty() && first.buckets().is_none());
    assert_eq!(second.buckets().unwrap().identity(), table);
    first.move_assign(&mut second);
    assert_eq!(first.buckets().unwrap().identity(), table);
    assert_eq!(first.seek1(&1), Some(&5));
    assert!(second.is_empty() && second.buckets().is_none());
}
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(align(64))]
struct WideKey(i32);
#[derive(Default)]
struct WideHash;
impl HashPolicy<WideKey> for WideHash {
    fn hash(&self, key: &WideKey) -> usize {
        key.0 as usize
    }
    fn equal(&self, a: &WideKey, b: &WideKey) -> bool {
        a == b
    }
}
#[test]
fn slots_support_overaligned_keys_and_zero_sized_pairs() {
    let mut wide = DoubleMap::<WideKey, WideKey, WideHash, WideHash>::default();
    wide.bind(WideKey(1), WideKey(2)).unwrap();
    assert_eq!(wide.seek1_ptr(&WideKey(1)).addr() % 64, 0);
    assert_eq!(wide.seek2_ptr(&WideKey(2)).addr() % 64, 0);
    let mut zero = DoubleMap::<(), (), ZeroHash, ZeroHash>::default();
    zero.bind((), ()).unwrap();
    assert_eq!(zero.len(), 1);
    assert!(zero.unbind2(&()));
}

#[derive(Default)]
struct ZeroHash;
impl HashPolicy<()> for ZeroHash {
    fn hash(&self, _key: &()) -> usize {
        0
    }
    fn equal(&self, _first: &(), _second: &()) -> bool {
        true
    }
}
