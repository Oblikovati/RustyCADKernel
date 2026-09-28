//! Exclusive traversal follows links after slot reuse; nodes remain stable across replacement.
use rustycad_tkernel::ncollection::{BaseAllocator, DefaultHashPolicy, OrderedDataMap};
#[test]
fn mutable_iteration_follows_link_order_after_removal_and_slot_reuse() {
    let mut map = OrderedDataMap::<i32, i32>::default();
    map.bind(10, 1);
    map.bind(20, 2);
    map.bind(30, 3);
    let address = map.find(&10).unwrap() as *const i32;
    map.unbind(&20);
    map.bind(40, 4);
    let keys: Vec<i32> = map
        .items_mut()
        .map(|(key, value)| {
            *value *= 10;
            *key
        })
        .collect();
    assert_eq!(keys, [10, 30, 40]);
    assert_eq!(map.iter().copied().collect::<Vec<_>>(), [10, 30, 40]);
    for value in &mut map {
        *value += 1;
    }
    assert_eq!(
        (&map).into_iter().copied().collect::<Vec<_>>(),
        [11, 31, 41]
    );
    assert_eq!(map.bound(10, 99) as *const i32, address);
    assert!(!map.try_emplace(10, || panic!("must retain existing value")));
    assert_eq!(
        *map.try_emplaced(10, || panic!("must retain existing value")),
        99
    );
    assert!(map.items() == map.items().clone());
    assert!(map.iter() == map.iter().clone());
    assert!(map.iter() != map.end());
}
#[test]
fn empty_cursor_errors_and_allocator_nodes_are_released() {
    let arena = BaseAllocator::incremental(128);
    let mut map = OrderedDataMap::with_allocator(DefaultHashPolicy, 10, arena.clone());
    assert_eq!(map.allocator(), &arena);
    assert_eq!(map.bucket_count(), 10);
    let cursor = map.cursor();
    assert!(!cursor.more());
    assert!(cursor.value(&map).is_err());
    map.bind(1, 42);
    assert!(cursor.change_value(&mut map).is_err());
    let mut output = 99;
    assert!(!map.find_copy(&2, &mut output));
    assert_eq!(output, 99);
    map.clear(true);
    arena.reset(true).unwrap();
}
