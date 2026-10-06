use super::fixture::{keys, populated};
use rustycad_tkernel::ncollection::{DataMap, DataMapNode, Map};
#[test]
fn actual_bucket_chain_contains_real_native_nodes() {
    let mut map = populated(1);
    assert_eq!(map.buckets(), map.buckets_mut().cast_const());
    let mut pointer = unsafe { *map.buckets().add(1) };
    let mut observed = String::new();
    while !pointer.is_null() {
        let node = unsafe { &*pointer.cast::<DataMapNode<i32, i32>>() };
        assert_eq!(*node.value(), node.key() * 10);
        observed.push_str(&format!("{},", node.key()));
        pointer = node.link().next();
    }
    assert_eq!(observed, keys(&map));
}
#[test]
fn rehash_and_other_node_removal_preserve_item_addresses() {
    let mut map = populated(1);
    let pointer = std::ptr::from_ref(map.find(&2).unwrap());
    map.bind(4, 40);
    map.resize(2000);
    map.unbind(&3);
    assert_eq!(unsafe { *pointer }, 20);
    assert_eq!(pointer, std::ptr::from_ref(map.find(&2).unwrap()));
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
fn aligned_values_and_zero_sized_values_use_real_nodes() {
    let mut aligned = DataMap::<i32, AlignedValue>::default();
    aligned.bind(7, AlignedValue(42));
    let pointer = std::ptr::from_ref(aligned.find(&7).unwrap());
    assert_eq!(pointer.addr() % 256, 0);
    aligned.resize(2000);
    assert_eq!(unsafe { (*pointer).0 }, 42);
    let mut zero = DataMap::<i32, ()>::default();
    assert!(zero.bind(0, ()));
    assert!(!zero.bind(0, ()));
    assert_eq!(zero.items().count(), 1);
    assert!(zero.unbind(&0));
}
#[test]
fn wrapped_map_projects_stable_keys_after_native_rehash() {
    let mut map = Map::<i32>::default();
    map.add(7);
    let pointer = std::ptr::from_ref(map.contained(&7).unwrap());
    map.add(8);
    map.resize(2000);
    map.remove(&8);
    assert_eq!(unsafe { *pointer }, 7);
    assert!(!map.emplace(7));
    assert_eq!(map.iter().copied().collect::<Vec<_>>(), [7]);
}
