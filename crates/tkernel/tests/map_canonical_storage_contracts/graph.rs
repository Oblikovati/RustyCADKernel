use super::fixture::{native_row, populated, table_of};
use rustycad_tkernel::ncollection::{HashPolicy, Map};
#[test]
fn bucket_table_head_is_the_first_cursor_entry() {
    let map = populated(1);
    let table = map.buckets().unwrap();
    // The collision hash places every key in bucket 1; its head is the native first node.
    let (head, unit) = table.chain(1).unwrap().next().unwrap();
    let cursor = map.cursor();
    let same_entry = std::ptr::eq(head, cursor.key().unwrap());
    let same_key = std::ptr::eq(cursor.key().unwrap(), cursor.value().unwrap());
    native_row(
        "actual_node",
        format!(
            "actual_node {} {} {head}",
            u8::from(same_entry),
            u8::from(same_key)
        ),
    );
    assert_eq!(unit, &());
}
#[test]
fn key_only_cursor_postfix_advance_returns_readonly_keys() {
    let map = populated(1);
    let mut cursor = map.cursor();
    let old = cursor.advance_post();
    let same = std::ptr::eq(old.value().unwrap(), old.key().unwrap());
    native_row(
        "key_iterator",
        format!(
            "key_iterator {} {} {} {}",
            old.value().unwrap(),
            old.key().unwrap(),
            cursor.value().unwrap(),
            u8::from(same)
        ),
    );
}
#[test]
fn duplicate_add_and_rehash_keep_the_stored_key() {
    let mut map = populated(1);
    map.add(2);
    map.add(4);
    map.resize(2000);
    map.remove(&3);
    assert_eq!(map.contained(&2), Some(&2));
    assert_eq!(map.len(), 3);
}
#[test]
fn bucket_table_preserves_empty_table_states() {
    let mut map = Map::<i32>::new(0);
    assert!(table_of(&map).is_none());
    map.add(7);
    let table = table_of(&map);
    map.clear(false);
    assert_eq!(table_of(&map), table);
    assert!(!map.cursor().more());
    map.clear(true);
    assert!(table_of(&map).is_none());
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
fn aligned_keys_stay_aligned_through_growth() {
    let mut map = Map::with_hasher(AlignedHash, 1);
    map.add(AlignedKey(7));
    let key = std::ptr::from_ref(map.contained(&AlignedKey(7)).unwrap());
    assert_eq!(key.addr() % 256, 0);
    map.resize(2000);
    assert_eq!(map.contained(&AlignedKey(7)).unwrap().0, 7);
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
fn zero_sized_keys_are_stored_once() {
    let mut map = Map::with_hasher(ZeroHash, 1);
    assert!(map.add(()));
    assert!(!map.add(()));
    assert_eq!(map.iter().count(), 1);
    assert_eq!(map.cursor().key(), Ok(&()));
    assert!(map.remove(&()));
}
