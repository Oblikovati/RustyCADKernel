//! Probe-cluster edits and confirmed native displacement return behavior.
use rustycad_tkernel::ncollection::FlatMap;
#[test]
fn emplaced_retains_occt_displaced_key_return_while_added_returns_inserted_key() {
    let mut map = FlatMap::<i32>::default();
    map.add(0);
    map.add(1);
    assert_eq!(map.emplaced(8), &1);
    assert!(map.contains(&8));
    assert_eq!(map.len(), 3);
    let mut added = FlatMap::<i32>::default();
    added.add(0);
    added.add(1);
    assert_eq!(added.added(8), &8);
    let mut tried = FlatMap::<i32>::default();
    tried.add(0);
    tried.add(1);
    assert_eq!(tried.try_emplaced(8), &1);
    assert!(!tried.try_emplace(8));
}
#[test]
fn wraparound_deletion_capacity_reuse_copy_and_exchange_preserve_membership() {
    let mut map = FlatMap::<i32>::default();
    assert_eq!(map.capacity(), 0);
    map.reserve(0);
    assert_eq!(map.capacity(), 8);
    for key in [7, 15, 23, 31] {
        assert!(map.add(key));
    }
    assert!(map.remove(&15));
    for key in [7, 23, 31] {
        assert!(map.contains(&key));
    }
    assert!(map.iter() == map.iter().clone());
    assert_eq!(map.extent(), 3);
    map.reserve(100);
    assert_eq!(map.capacity(), 128);
    assert!(map.remove(&7));
    let mut copy = map.clone();
    assert_eq!(copy.capacity(), 128);
    map.clear(false);
    assert_eq!(map.capacity(), 128);
    assert_eq!(map.clone().capacity(), 0);
    map.exchange(&mut copy);
    assert!(copy.is_empty());
    assert!(map.contains(&23));
    map.clear(true);
    assert_eq!(map.capacity(), 0);
    assert!(!map.remove(&31));
}
