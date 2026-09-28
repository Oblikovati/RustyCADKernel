//! Colliding bidirectional indices, stable references and native real hashes.
use rustycad_tkernel::ncollection::{BaseAllocator, DoubleMap, HashKey, HashPolicy};
#[derive(Clone, Default)]
struct Collision;
impl HashPolicy<i32> for Collision {
    fn hash(&self, _key: &i32) -> usize {
        42
    }
    fn equal(&self, left: &i32, right: &i32) -> bool {
        left == right
    }
}
#[test]
fn collisions_slot_reuse_and_resize_preserve_both_indices() {
    let arena = BaseAllocator::incremental(128);
    let mut map = DoubleMap::with_allocator(Collision, Collision, 10, arena.clone());
    assert_eq!(map.bucket_count(), 10);
    assert_eq!(map.allocator(), &arena);
    map.bind(1, 10).unwrap();
    map.bind(2, 20).unwrap();
    map.bind(3, 30).unwrap();
    let address = map.find1(&1).unwrap() as *const i32;
    assert!(map.are_bound(&1, &10));
    assert!(!map.are_bound(&1, &20));
    assert!(map.unbind2(&20));
    map.bind(4, 40).unwrap();
    assert!(map.are_bound(&4, &40));
    assert!(!map.is_bound1(&2));
    map.resize(1000);
    assert_eq!(map.find1(&1).unwrap() as *const i32, address);
    assert!(map.items() == map.items().clone());
    assert!(map.iter() == map.iter().clone());
    assert!(map.find1(&99).is_err());
    assert!(map.find2(&99).is_err());
    map.clear(false);
    arena.reset(true).unwrap();
    assert!(map.is_empty());
    assert!(!map.are_bound(&1, &10));
}
#[test]
#[cfg(target_pointer_width = "64")]
fn real_hashes_match_captured_linux_libstdcpp_values() {
    let cases = [
        (0.0, 0),
        (-0.0, 0),
        (1.0, 8386164645967068059),
        (2.0, 6369015886390043782),
        (3.0, 10527412025248383462),
        (0.1, 4944624500300936044),
        (1.1, 6614599470348095854),
        (-1.0, 11078049357879903929),
    ];
    for (value, expected) in cases {
        assert_eq!(value.occt_hash(), expected);
    }
}
