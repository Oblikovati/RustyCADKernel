//! Bucket transitions and policy ownership beyond the legacy examples.
use rustycad_tkernel::ncollection::{BaseAllocator, DataMap, HashPolicy};
#[derive(Clone)]
struct Salt(usize);
impl HashPolicy<i32> for Salt {
    fn hash(&self, key: &i32) -> usize {
        (*key as usize).wrapping_add(self.0)
    }
    fn equal(&self, first: &i32, second: &i32) -> bool {
        first == second
    }
}
#[test]
fn assignment_keeps_destination_hash_policy_and_allocator() {
    let allocator = BaseAllocator::incremental(128);
    let mut source = DataMap::with_hasher(Salt(42), 1);
    source.bind(1, 7);
    let mut destination = DataMap::with_allocator(Salt(99), 1, allocator.clone());
    destination.assign(&source);
    assert_eq!(destination.hasher().0, 99);
    assert_eq!(destination.allocator(), &allocator);
    assert_eq!(destination.find(&1), Ok(&7));
    destination.clear(false);
    allocator.reset(true).unwrap();
}
#[test]
fn resize_preserves_values_and_try_emplace_does_not_construct_existing_values() {
    let mut map = DataMap::<i32, i32>::default();
    map.bind(1, 42);
    let pointer = map.find(&1).unwrap() as *const i32;
    assert_eq!(map.bucket_count(), 101);
    for i in 2..=103 {
        map.bind(i, i);
    }
    assert_eq!(map.bucket_count(), 1009);
    assert_eq!(map.find(&1).unwrap() as *const i32, pointer);
    assert!(!map.try_emplace(1, || panic!("existing key must not invoke constructor")));
    assert_eq!(map.find(&1), Ok(&42));
    map.resize(2000);
    assert_eq!(map.bucket_count(), 2003);
    map.clear(false);
    assert!(map.is_empty());
    assert_eq!(map.bucket_count(), 2003);
    map.bind(-1, -1);
    assert_eq!(map.find(&-1), Ok(&-1));
}
