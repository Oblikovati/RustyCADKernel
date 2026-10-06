//! Native-valid keys need neither formatting nor copying for owned operations.
use rustycad_tkernel::ncollection::{BaseAllocator, HashPolicy, OrderedDataMap};
#[derive(Clone)]
struct OpaqueKey(i32);
#[derive(Clone, Default)]
struct OpaqueHasher;
impl HashPolicy<OpaqueKey> for OpaqueHasher {
    fn hash(&self, key: &OpaqueKey) -> usize {
        key.0 as usize
    }
    fn equal(&self, a: &OpaqueKey, b: &OpaqueKey) -> bool {
        a.0 == b.0
    }
}
type OpaqueMap = OrderedDataMap<OpaqueKey, i32, OpaqueHasher>;
fn populated() -> OpaqueMap {
    let mut map = OpaqueMap::default();
    assert!(map.bind(OpaqueKey(1), 10));
    assert!(map.bind(OpaqueKey(2), 20));
    map
}
#[test]
fn unformatted_keys_support_construction_membership_and_removal() {
    let mut map = OpaqueMap::with_int_buckets(OpaqueHasher, 0, BaseAllocator::common()).unwrap();
    assert!(map.is_empty());
    assert!(map.bind(OpaqueKey(1), 10));
    assert!(map.is_bound(&OpaqueKey(1)));
    assert_eq!(map.seek(&OpaqueKey(1)), Some(&10));
    assert_eq!(map.seek(&OpaqueKey(9)), None);
    *map.change_seek(&OpaqueKey(1)).unwrap() = 11;
    assert_eq!(map.contained(&OpaqueKey(1)).unwrap().0.0, 1);
    assert!(map.unbind(&OpaqueKey(1)));
    assert!(map.is_empty());
}
#[test]
fn unformatted_keys_support_both_mutable_iterator_views() {
    let mut map = populated();
    for value in &mut map {
        *value += 1;
    }
    for (key, value) in map.items_mut() {
        *value += key.0;
    }
    assert_eq!(map.iter().copied().collect::<Vec<_>>(), [12, 23]);
    assert_eq!((&map).into_iter().copied().collect::<Vec<_>>(), [12, 23]);
    assert_eq!(
        map.items()
            .map(|(key, value)| (key.0, *value))
            .collect::<Vec<_>>(),
        [(1, 12), (2, 23)]
    );
    assert!(map.end().next().is_none());
    assert!(map.items_end().next().is_none());
}
#[test]
fn unformatted_keys_support_cursors_and_node_edges() {
    let mut map = populated();
    let mut cursor = map.cursor();
    assert_eq!(cursor.key(&map).unwrap().0, 1);
    *cursor.change_value(&mut map).unwrap() = 15;
    assert_eq!(cursor.post_advance(&map).value(&map), Ok(&15));
    assert_eq!(cursor.value(&map), Ok(&20));
    cursor.reset(&map);
    assert_eq!(cursor.key(&map).unwrap().0, 1);
    *map.first_node_mut().unwrap().change_value() = 16;
    assert_eq!(map.first_node().unwrap().order_next().unwrap().key().0, 2);
    assert_eq!(map.first().unwrap().0, 1);
    assert_eq!(map.last().unwrap().0, 2);
    assert_eq!(map.first_value(), Ok(&16));
    assert_eq!(map.last_value(), Ok(&20));
}
#[test]
fn unformatted_keys_support_copy_move_and_allocator_operations() {
    let mut map = populated();
    let copied = map.clone();
    assert_eq!(copied.extent(), 2);
    let mut output = 0;
    assert!(map.find_copy(&OpaqueKey(2), &mut output));
    assert_eq!(output, 20);
    let mut moved = OpaqueMap::move_from(&mut map);
    assert!(map.is_empty());
    map.assign(&moved).resize_int(200).unwrap();
    assert_eq!(map.extent(), 2);
    map.exchange(&mut moved);
    map.clear_with_allocator(None);
    assert!(map.is_empty());
    assert_eq!(map.allocator(), &BaseAllocator::common());
}
struct MoveOnlyKey(i32);
struct MoveOnlyValue(i32);
struct MoveOnlyHasher;
impl HashPolicy<MoveOnlyKey> for MoveOnlyHasher {
    fn hash(&self, key: &MoveOnlyKey) -> usize {
        key.0 as usize
    }
    fn equal(&self, a: &MoveOnlyKey, b: &MoveOnlyKey) -> bool {
        a.0 == b.0
    }
}
#[test]
fn borrowed_item_overloads_do_not_require_key_clone() {
    let mut map = OrderedDataMap::with_hasher(MoveOnlyHasher, 101);
    assert!(map.bind_value_ref(MoveOnlyKey(1), &10));
    assert_eq!(*map.bound_value_ref(MoveOnlyKey(1), &11), 11);
    assert!(!map.try_bind_value_ref(MoveOnlyKey(1), &12));
    assert_eq!(*map.try_bound_value_ref(MoveOnlyKey(1), &13), 11);
}
#[test]
fn borrowed_key_overloads_do_not_require_item_clone() {
    let mut map = OrderedDataMap::with_hasher(OpaqueHasher, 101);
    assert!(map.bind_key_ref(&OpaqueKey(1), MoveOnlyValue(10)));
    assert_eq!(map.bound_key_ref(&OpaqueKey(1), MoveOnlyValue(11)).0, 11);
    assert!(!map.try_bind_key_ref(&OpaqueKey(1), MoveOnlyValue(12)));
    assert_eq!(
        map.try_bound_key_ref(&OpaqueKey(1), MoveOnlyValue(13)).0,
        11
    );
}
#[test]
fn formatted_missing_key_errors_retain_value_and_expected_domain() {
    let map = OrderedDataMap::<i32, i32>::default();
    let failure = map.find(&99).unwrap_err();
    let message = failure.message();
    assert!(message.contains("99"));
    assert!(message.contains("expected a bound key"));
}

#[test]
fn unformatted_keys_support_generic_items_views() {
    use rustycad_tkernel::ncollection::{ItemsView, ItemsViewMut, KeyValueItems};
    let mut map = populated();
    let view = ItemsView::<_, KeyValueItems>::new(&map);
    assert_eq!((view.size(), view.is_empty()), (2, false));
    assert_eq!(
        view.cbegin()
            .map(|(key, value)| (key.0, *value))
            .collect::<Vec<_>>(),
        [(1, 10), (2, 20)]
    );
    for (key, value) in ItemsViewMut::<_, KeyValueItems>::new(&mut map).begin() {
        *value += key.0;
    }
    assert_eq!(map.iter().copied().collect::<Vec<_>>(), [11, 22]);
}
