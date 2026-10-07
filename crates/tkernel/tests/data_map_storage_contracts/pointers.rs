use super::fixture::{keys, populated};
use rustycad_tkernel::ncollection::{DataMap, Map};
#[test]
fn public_bucket_table_chain_matches_the_native_traversal() {
    let map = populated(1);
    let table = map.buckets().unwrap();
    assert_eq!(table.len(), map.bucket_count() + 1);
    let mut observed = String::new();
    // The collision hash places every key in bucket 1, head first.
    for (key, value) in table.chain(1).unwrap() {
        assert_eq!(*value, key * 10);
        observed.push_str(&format!("{key},"));
    }
    assert_eq!(observed, keys(&map));
    assert!(table.chain(0).unwrap().next().is_none());
    let past_end = table.chain(table.len()).err().unwrap();
    assert_eq!(past_end.exception_type(), "Standard_OutOfRange");
}
#[test]
fn rehash_and_other_removals_preserve_values() {
    let mut map = populated(1);
    map.bind(4, 40);
    map.resize(2000);
    map.unbind(&3);
    assert_eq!(map.find(&2), Ok(&20));
    assert_eq!(map.find(&4), Ok(&40));
    assert_eq!(keys(&map), "1,2,4,");
}
#[test]
fn mutable_iterator_lends_disjoint_values_simultaneously() {
    let mut map = populated(1);
    let mut range = map.items_mut();
    let (first_key, first) = range.next().unwrap();
    let (second_key, second) = range.next().unwrap();
    *first += 1;
    *second += 2;
    assert_eq!((*first_key, *first, *second_key, *second), (3, 31, 2, 22));
    assert_eq!(
        range.next().map(|(key, value)| (*key, *value)),
        Some((1, 10))
    );
    assert!(range.next().is_none());
}
#[derive(Debug)]
#[repr(align(256))]
struct AlignedValue(i32);
#[test]
fn aligned_values_and_zero_sized_values_are_stored() {
    let mut aligned = DataMap::<i32, AlignedValue>::default();
    aligned.bind(7, AlignedValue(42));
    assert_eq!(
        std::ptr::from_ref(aligned.find(&7).unwrap()).addr() % 256,
        0
    );
    aligned.resize(2000);
    assert_eq!(aligned.find(&7).unwrap().0, 42);
    let mut zero = DataMap::<i32, ()>::default();
    assert!(zero.bind(0, ()));
    assert!(!zero.bind(0, ()));
    assert_eq!(zero.items().count(), 1);
    assert!(zero.unbind(&0));
}
#[test]
fn key_only_map_keeps_keys_after_native_rehash_and_removal() {
    let mut map = Map::<i32>::default();
    map.add(7);
    map.add(8);
    map.resize(2000);
    map.remove(&8);
    assert_eq!(map.contained(&7), Some(&7));
    assert!(!map.emplace(7));
    assert_eq!(map.iter().copied().collect::<Vec<_>>(), [7]);
}
