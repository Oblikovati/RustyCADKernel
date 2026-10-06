use super::fixture::{native_row, populated};
use rustycad_tkernel::ncollection::{
    BaseMapBasicIterator, BasicMapIteratorSource, HashPolicy, ListValueNode, Map, MapNode,
};
#[test]
fn facade_owns_actual_map_nodes_with_real_list_value_prefix() {
    let mut map = populated(1);
    assert_eq!(map.buckets(), map.buckets_mut().cast_const());
    let first = unsafe { *map.buckets().add(1) }.cast::<MapNode<i32>>();
    let cursor = BaseMapBasicIterator::<MapNode<i32>, false, true>::new(&map);
    let node = unsafe { &*first };
    let same_key = unsafe { std::ptr::from_ref(node.key()) == cursor.value_ptr() };
    native_row(
        "actual_node",
        format!(
            "actual_node {} {} {}",
            u8::from(first.cast_const() == cursor.node()),
            u8::from(same_key),
            node.key()
        ),
    );
    assert_eq!(unsafe { &*first.cast::<ListValueNode<i32>>() }.value(), &3);
}
#[test]
fn real_map_generic_mutable_specialization_returns_readonly_keys() {
    let mut map = populated(1);
    let mut cursor = BaseMapBasicIterator::<MapNode<i32>, true, true>::new(&mut map);
    let old = unsafe { cursor.advance_post() };
    let same = unsafe { old.value_ptr() == std::ptr::from_ref(old.key()) };
    native_row("key_iterator", unsafe {
        format!(
            "key_iterator {} {} {} {}",
            old.value(),
            old.key(),
            cursor.value(),
            u8::from(same)
        )
    });
}
#[test]
fn duplicate_add_and_rehash_preserve_key_pointer_provenance() {
    let mut map = populated(1);
    let key = std::ptr::from_ref(map.contained(&2).unwrap());
    map.add(2);
    map.add(4);
    map.resize(2000);
    map.remove(&3);
    assert_eq!(unsafe { *key }, 2);
    assert_eq!(key, std::ptr::from_ref(map.contained(&2).unwrap()));
}
#[test]
fn actual_map_source_preserves_empty_table_states() {
    let mut map = Map::<i32>::new(0);
    assert!(map.basic_bucket_slots().is_none());
    assert!(map.basic_first_node().is_null());
    map.add(7);
    let table = map.buckets();
    map.clear(false);
    assert_eq!(map.basic_bucket_slots().unwrap().as_ptr(), table);
    assert!(!BaseMapBasicIterator::<MapNode<i32>, false, true>::new(&map).more());
    map.clear(true);
    assert!(map.basic_bucket_slots().is_none());
}
#[derive(Clone, Debug)]
#[repr(align(256))]
struct AlignedKey(i32);
struct AlignedHash;
impl HashPolicy<AlignedKey> for AlignedHash {
    fn hash(&self, key: &AlignedKey) -> usize {
        key.0 as usize
    }
    fn equal(&self, a: &AlignedKey, b: &AlignedKey) -> bool {
        a.0 == b.0
    }
}
#[test]
fn aligned_keys_keep_addresses_through_growth() {
    let mut map = Map::with_hasher(AlignedHash, 1);
    map.add(AlignedKey(7));
    let key = std::ptr::from_ref(map.contained(&AlignedKey(7)).unwrap());
    assert_eq!(key.addr() % 256, 0);
    map.resize(2000);
    assert_eq!(unsafe { (*key).0 }, 7);
    assert!(map.remove(&AlignedKey(7)));
}
struct ZeroHash;
impl HashPolicy<()> for ZeroHash {
    fn hash(&self, _: &()) -> usize {
        0
    }
    fn equal(&self, _: &(), _: &()) -> bool {
        true
    }
}
#[test]
fn zero_sized_keys_use_real_map_node_storage() {
    let mut map = Map::with_hasher(ZeroHash, 1);
    assert!(map.add(()));
    assert!(!map.add(()));
    assert_eq!(map.iter().count(), 1);
    let cursor = BaseMapBasicIterator::<MapNode<()>, false, true>::new(&map);
    assert_eq!(unsafe { cursor.key() }, &());
    assert!(map.remove(&()));
}
