//! Insertion-reference identity and lazy construction through Robin Hood displacement.
use rustycad_tkernel::ncollection::FlatDataMap;
#[test]
fn displaced_insertions_return_the_new_values_and_keep_old_pairing() {
    let mut map = FlatDataMap::<i32, i32>::default();
    map.bind(0, 10);
    map.bind(1, 11);
    assert_eq!(*map.emplaced(8, || 18), 18);
    assert_eq!(map.find(&1), Ok(&11));
    assert_eq!(
        *map.try_emplaced(8, || panic!("existing key must not construct")),
        18
    );
    assert!(!map.try_emplace(1, || panic!("existing key must not construct")));
    map.reserve(100);
    assert_eq!(map.capacity(), 128);
    assert_eq!(map.find(&8), Ok(&18));
    map[&8] = 88;
    assert_eq!(map.seek(&8), Some(&88));
    assert!(map.items() == map.items().clone());
    assert!(map.iter() == map.iter().clone());
    assert!(map.iter() != map.end());
    let mut empty = FlatDataMap::default();
    map.exchange(&mut empty);
    assert!(map.is_empty());
    assert_eq!(empty.find(&8), Ok(&88));
    assert_eq!((&empty).into_iter().count(), 3);
}
