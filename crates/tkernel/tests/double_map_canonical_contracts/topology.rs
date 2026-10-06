use rustycad_tkernel::ncollection::{
    BaseAllocator, BaseMapBasicIterator, DoubleMap, DoubleMapNode, HashPolicy,
};
#[test]
fn facade_exposes_actual_dual_chain_nodes_and_native_generic_key2() {
    let (mut map, _events) = super::hash_order::ordered_map();
    map.bind(1, 5).unwrap();
    map.bind(2, 6).unwrap();
    let state = map.observer().snapshot();
    assert_eq!(
        state.second_table - state.first_table,
        (state.bucket_count + 1) * size_of::<usize>()
    );
    let mut cursor = BaseMapBasicIterator::<DoubleMapNode<i32, i32>, false, true>::new(&map);
    unsafe {
        assert_eq!(cursor.key(), &5);
        assert_eq!((*cursor.node()).key1(), &1);
        assert_eq!(cursor.value_ptr(), map.seek1_ptr(&1));
        cursor.advance();
        assert_eq!(cursor.key(), &6);
    }
}
#[test]
fn retained_payload_pointers_survive_link_rewrites_resize_and_neighbor_removal() {
    let (mut map, _events) = super::hash_order::ordered_map();
    map.bind(1, 5).unwrap();
    let first = map.seek2_ptr(&5);
    let second = map.seek1_ptr(&1);
    map.bind(2, 6).unwrap();
    map.bind(3, 7).unwrap();
    map.resize(200);
    assert!(map.unbind2(&6));
    unsafe {
        assert_eq!((*first, *second), (1, 5));
    }
    assert_eq!(map.seek1_ptr(&1), second);
    assert_eq!(map.seek2_ptr(&5), first);
}
#[test]
fn clear_with_same_allocator_releases_both_tables_and_keeps_observer_identity() {
    let arena = BaseAllocator::incremental(512);
    let mut map = DoubleMap::<i32, i32>::with_allocator(
        Default::default(),
        Default::default(),
        1,
        arena.clone(),
    );
    map.bind(1_i32, 5_i32).unwrap();
    let observer = map.observer();
    map.clear_with_allocator(Some(arena.clone()));
    let state = observer.snapshot();
    assert_eq!(
        (state.size, state.first_table, state.second_table),
        (0, 0, 0)
    );
    assert_eq!(state.allocator, arena.identity().addr());
}
#[test]
fn exchange_and_move_keep_observers_attached_to_their_original_containers() {
    let mut first = DoubleMap::<i32, i32>::default();
    let mut second = DoubleMap::<i32, i32>::default();
    first.bind(1, 5).unwrap();
    let a = first.observer();
    let b = second.observer();
    let table = a.snapshot().first_table;
    first.exchange(&mut second);
    assert_eq!(a.snapshot().size, 0);
    assert_eq!(b.snapshot().first_table, table);
    first.move_assign(&mut second);
    assert_eq!(a.snapshot().first_table, table);
    assert_eq!(b.snapshot().size, 0);
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
fn actual_nodes_support_overaligned_keys_and_zero_sized_pairs() {
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
