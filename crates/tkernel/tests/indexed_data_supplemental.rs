//! Indexed values retain address identity and failure atomicity through edits.
use rustycad_tkernel::ncollection::{BaseAllocator, DefaultHashPolicy, IndexedDataMap};
#[test]
fn existing_values_are_stable_and_try_construction_is_lazy() {
    let mut map = IndexedDataMap::<i32, i32>::default();
    let address = map.bound(1, 42) as *const i32;
    assert_eq!(map.try_emplace(1, || panic!("must not construct")), 1);
    assert_eq!(*map.try_emplaced(1, || panic!("must not construct")), 42);
    assert_eq!(map.emplaced(1, || 7) as *const i32, address);
    map.add(2, 99);
    map.swap(1, 2).unwrap();
    map.resize(1000);
    assert_eq!(map.find_from_key(&1).unwrap() as *const i32, address);
    map.remove_from_index(1).unwrap();
    assert_eq!(map.find_index(&1), 1);
    assert_eq!(map.find_from_index(1), Ok(&7));
    assert!(map.items() == map.items().clone());
    assert!(map.iter() == map.iter().clone());
    assert_eq!((&map).into_iter().copied().collect::<Vec<_>>(), [7]);
}
#[test]
fn invalid_edits_preserve_bindings_and_allocator_nodes_are_released() {
    let arena = BaseAllocator::incremental(128);
    let mut map = IndexedDataMap::with_allocator(DefaultHashPolicy, 10, arena.clone());
    assert_eq!(map.allocator(), &arena);
    assert_eq!(map.bucket_count(), 10);
    assert!(map.remove_last().is_err());
    map.add(1, 42);
    map.add(2, 99);
    assert!(map.change_from_index(0).is_err());
    assert!(map.find_from_index(3).is_err());
    assert!(map.change_from_key(&3).is_err());
    assert!(map.find_from_key(&3).is_err());
    assert!(map.substitute(1, 2, 0).is_err());
    assert!(map.swap(1, 3).is_err());
    assert!(map.remove_from_index(0).is_err());
    assert_eq!(map.find_from_key(&1), Ok(&42));
    let mut unchanged = 123;
    assert!(!map.find_copy(&3, &mut unchanged));
    assert_eq!(unchanged, 123);
    map.clear(false);
    arena.reset(true).unwrap();
    assert!(map.is_empty());
}
