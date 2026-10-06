//! Native FlatMap iterator and pointer contracts, including non-Clone keys/policies.
use rustycad_tkernel::ncollection::{FlatMap, FlatMapIter, HashPolicy};

#[derive(Debug)]
struct Key {
    identity: i32,
    label: &'static str,
}
struct IdentityPolicy;
impl HashPolicy<Key> for IdentityPolicy {
    fn hash(&self, key: &Key) -> usize {
        key.identity as usize
    }
    fn equal(&self, left: &Key, right: &Key) -> bool {
        left.identity == right.identity
    }
}
fn key(identity: i32, label: &'static str) -> Key {
    Key { identity, label }
}

#[test]
fn raw_queries_return_null_for_unbound_and_same_slot_for_bound_key() {
    let mut map = FlatMap::with_hasher(IdentityPolicy, 0);
    let lookup = key(7, "lookup");
    assert!(map.seek_ptr(&lookup).is_null());
    assert!(map.change_seek_ptr(&lookup).is_null());
    map.add(key(7, "stored"));
    let pointer = map.seek_ptr(&lookup);
    assert_eq!(pointer, map.change_seek_ptr(&lookup).cast_const());
    assert_eq!(pointer, std::ptr::from_ref(map.contained(&lookup).unwrap()));
    map.change_seek(&lookup).unwrap().label = "changed";
    assert_eq!(map.seek_ptr(&lookup), pointer);
    assert_eq!(map.seek(&lookup).unwrap().label, "changed");
}

#[test]
fn empty_reserved_and_cleared_queries_still_return_null() {
    let mut map = FlatMap::<i32>::new(20);
    assert!(map.seek_ptr(&7).is_null());
    map.add(7);
    map.clear(false);
    assert!(map.capacity() > 0);
    assert!(map.change_seek_ptr(&7).is_null());
    map.clear(true);
    assert!(map.seek_ptr(&7).is_null());
}

#[test]
fn iterator_pointer_copy_assignment_and_postfix_keep_slot_identity() {
    let mut map = FlatMap::with_hasher(IdentityPolicy, 0);
    map.add(key(7, "stored"));
    let source = map.iter();
    let mut cursor = FlatMapIter::default();
    let address = &mut cursor as *mut _;
    assert_eq!(cursor.assign_from(&source) as *mut _, address);
    assert!(cursor.is_equal(&source));
    assert_eq!(cursor.key_ptr().unwrap(), map.seek_ptr(&key(7, "lookup")));
    assert!(cursor.advance_post().is_equal(&source));
    assert!(!cursor.more());
    assert!(cursor.is_equal(&map.end()));
}

#[test]
fn iterator_prefix_self_assignment_and_exhaustion_match_native() {
    let mut map = FlatMap::<i32>::default();
    map.add(7);
    let mut cursor = map.iter();
    let address = &mut cursor as *mut _;
    assert_eq!(cursor.assign_self() as *mut _, address);
    assert_eq!(cursor.advance_returning() as *mut _, address);
    assert!(cursor.is_equal(&FlatMapIter::default()));
    assert!(cursor.key_ptr().is_err());
    let old = cursor.advance_post();
    assert!(cursor.is_equal(&old));
}

#[test]
fn exhausted_cursor_assignment_can_rebind_to_another_container() {
    let mut first = FlatMap::<i32>::default();
    let mut second = FlatMap::<i32>::default();
    first.add(7);
    second.add(8);
    let mut cursor = first.end();
    let source = second.iter();
    assert!(!cursor.is_equal(&source));
    cursor.assign_from(&source);
    assert_eq!(cursor.key(), Ok(&8));
    assert!(!cursor.is_equal(&first.iter()));
}

#[test]
fn container_self_assignment_does_not_require_cloning_key_or_hasher() {
    let mut map = FlatMap::with_hasher(IdentityPolicy, 0);
    map.add(key(7, "stored"));
    let pointer = map.seek_ptr(&key(7, "lookup"));
    let address = &mut map as *mut _;
    assert_eq!(map.assign_self() as *mut _, address);
    assert_eq!(map.seek_ptr(&key(7, "lookup")), pointer);
    assert_eq!(map.len(), 1);
}
