//! DataMap assertions retain replacement, nonreplacement and hasher-state contracts.
#![allow(non_snake_case)]
use rustycad_tkernel::ncollection::{HashPolicy, Map, map_algo};
use rustycad_tkernel::tcollection::AsciiString;
use std::collections::BTreeSet;

fn integers(values: impl IntoIterator<Item = i32>) -> Map<i32> {
    let mut map = Map::default();
    for value in values {
        map.add(value);
    }
    map
}
fn expect_keys(map: &Map<i32>, values: impl IntoIterator<Item = i32>) {
    let expected: BTreeSet<i32> = values.into_iter().collect();
    assert_eq!(map.extent(), expected.len());
    for value in expected {
        assert!(map.contains(&value));
    }
}
#[derive(Clone, Default)]
struct StatefulHasher {
    salt: i32,
}
impl HashPolicy<i32> for StatefulHasher {
    fn hash(&self, key: &i32) -> usize {
        (key + self.salt) as usize
    }
    fn equal(&self, left: &i32, right: &i32) -> bool {
        left == right
    }
}
#[test]
fn NCollection_MapTest__DefaultConstructor() {
    let map = Map::<i32>::new(101);
    assert!(map.is_empty());
    assert_eq!(map.len(), 0);
    assert_eq!(map.extent(), 0);
    assert_eq!(map.bucket_count(), 101);
}
#[test]
fn NCollection_MapTest__ConstructorWithBuckets() {
    let map = Map::<i32>::new(100);
    assert!(map.is_empty());
    assert_eq!(map.len(), 0);
    assert_eq!(map.extent(), 0);
    assert_eq!(map.bucket_count(), 100);
}
#[test]
fn NCollection_MapTest__AddAndContains() {
    let mut map = Map::default();
    for key in [10, 20, 30] {
        assert!(map.add(key));
    }
    for key in [10, 20] {
        assert!(!map.add(key));
    }
    assert_eq!(map.len(), 3);
    expect_keys(&map, [10, 20, 30]);
    assert!(!map.contains(&40));
}
#[test]
fn NCollection_MapTest__Remove() {
    let mut map = integers([10, 20, 30]);
    assert!(map.remove(&20));
    assert_eq!(map.len(), 2);
    assert!(!map.contains(&20));
    expect_keys(&map, [10, 30]);
    assert!(!map.remove(&40));
    assert_eq!(map.len(), 2);
}
#[test]
fn NCollection_MapTest__Clear() {
    let mut map = integers([10, 20, 30]);
    assert!(!map.is_empty());
    map.clear(true);
    assert!(map.is_empty());
    assert_eq!(map.len(), 0);
    for key in [10, 20, 30] {
        assert!(!map.contains(&key));
    }
}
#[test]
fn NCollection_MapTest__Assignment() {
    let mut first = integers([10, 20, 30]);
    let mut second = Map::default();
    second.assign(&first);
    assert_eq!(first.len(), second.len());
    expect_keys(&second, [10, 20, 30]);
    first.add(40);
    first.remove(&10);
    assert_eq!(first.len(), 3);
    assert_eq!(second.len(), 3);
    assert!(second.contains(&10));
    assert!(!first.contains(&10));
    assert!(first.contains(&40));
    assert!(!second.contains(&40));
}
#[test]
fn NCollection_MapTest__IteratorAccess() {
    let map = integers([10, 20, 30]);
    let found: BTreeSet<i32> = map.iter().copied().collect();
    assert_eq!(found.len(), 3);
    for key in [10, 20, 30] {
        assert!(found.contains(&key));
    }
}
#[test]
fn NCollection_MapTest__Resize() {
    let mut map = Map::<i32>::new(10);
    for key in 0..100 {
        map.add(key);
    }
    assert_eq!(map.len(), 100);
    let keys: Vec<i32> = map.iter().copied().collect();
    map.resize(200);
    assert_eq!(map.len(), 100);
    for key in keys {
        assert!(map.contains(&key));
    }
}
#[test]
fn NCollection_MapTest__ExhaustiveIterator() {
    let map = integers(0..1000);
    assert_eq!(map.len(), 1000);
    let mut count = 0;
    let mut sum = 0;
    for key in &map {
        count += 1;
        sum += key;
    }
    assert_eq!(count, 1000);
    assert_eq!(sum, 1000 * 999 / 2);
}
#[test]
fn NCollection_MapTest__OCC24271_BooleanOperations() {
    let left = integers(1..=10);
    let right = integers(5..=15);
    assert!(!map_algo::contains(&left, &right));
    assert!(!map_algo::contains(&right, &left));
    let mut union = Map::default();
    map_algo::union(&mut union, &left, &right);
    expect_keys(&union, 1..=15);
    let mut intersection = Map::default();
    map_algo::intersection(&mut intersection, &left, &right);
    expect_keys(&intersection, 5..=10);
    assert!(map_algo::contains(&left, &intersection));
    assert!(map_algo::contains(&right, &intersection));
    let mut lr = Map::default();
    map_algo::subtraction(&mut lr, &left, &right);
    expect_keys(&lr, 1..5);
    let mut rl = Map::default();
    map_algo::subtraction(&mut rl, &right, &left);
    expect_keys(&rl, 11..=15);
    let mut difference = Map::default();
    map_algo::difference(&mut difference, &left, &right);
    expect_keys(&difference, (1..5).chain(11..=15));
    let mut swap = Map::default();
    swap.exchange(&mut intersection);
    expect_keys(&swap, 5..=10);
    assert!(intersection.is_empty());
}
#[test]
fn NCollection_MapTest__EmplaceBasic() {
    let mut map = Map::<AsciiString>::default();
    assert!(map.emplace(AsciiString::new("Hello")));
    assert_eq!(map.len(), 1);
    assert!(map.contains(&AsciiString::new("Hello")));
    assert!(!map.emplace(AsciiString::new("Hello")));
    assert_eq!(map.len(), 1);
}
#[test]
fn NCollection_MapTest__EmplacedBasic() {
    let mut map = Map::<AsciiString>::default();
    let first = map.emplaced(AsciiString::new("World"));
    assert_eq!(first, &AsciiString::new("World"));
    let address = first as *const AsciiString;
    assert_eq!(map.len(), 1);
    let second = map.emplaced(AsciiString::new("World"));
    assert_eq!(address, second as *const AsciiString);
}
#[test]
fn NCollection_MapTest__HasherConstructorCopy() {
    let hasher = StatefulHasher { salt: 42 };
    let mut map = Map::with_hasher(hasher.clone(), 10);
    for key in [1, 2, 3] {
        map.add(key);
    }
    assert_eq!(map.len(), 3);
    for key in [1, 2, 3] {
        assert!(map.contains(&key));
    }
    assert_eq!(map.hasher().salt, 42);
}
#[test]
fn NCollection_MapTest__HasherConstructorMove() {
    let mut map = Map::with_hasher(StatefulHasher { salt: 99 }, 10);
    map.add(10);
    map.add(20);
    assert_eq!(map.len(), 2);
    assert_eq!(map.hasher().salt, 99);
}
#[test]
fn NCollection_MapTest__CopyConstructorPreservesHasher() {
    let mut first = Map::with_hasher(StatefulHasher { salt: 123 }, 10);
    first.add(1);
    first.add(2);
    let second = first.clone();
    assert_eq!(second.hasher().salt, 123);
    assert_eq!(second.len(), 2);
    assert!(second.contains(&1));
    assert!(second.contains(&2));
}
#[test]
fn NCollection_MapTest__BeginEndIterator() {
    let map = integers([10, 20, 30]);
    let found: BTreeSet<i32> = map.iter().copied().collect();
    assert_eq!(found.len(), 3);
    for key in [10, 20, 30] {
        assert!(found.contains(&key));
    }
    let shared = &map;
    assert!(map.iter() == shared.iter());
    assert!(map.end() == shared.end());
    assert_eq!(shared.iter().find(|&&key| key == 20), Some(&20));
    assert_eq!(shared.iter().filter(|&&key| key == 10).count(), 1);
}
#[test]
fn NCollection_MapTest__RangeBasedForLoop() {
    let map = integers([100, 200, 300]);
    let mut found = BTreeSet::new();
    for key in &map {
        found.insert(*key);
    }
    assert_eq!(found.len(), 3);
    for key in [100, 200, 300] {
        assert!(found.contains(&key));
    }
}
#[test]
fn NCollection_MapTest__ContainedFound() {
    let map = integers([10, 20, 30]);
    assert_eq!(map.contained(&10), Some(&10));
    assert_eq!(map.contained(&30), Some(&30));
}
#[test]
fn NCollection_MapTest__ContainedNotFound() {
    let map = integers([10]);
    assert_eq!(map.contained(&99), None);
    let empty = Map::<i32>::default();
    assert_eq!(empty.contained(&10), None);
}
#[test]
fn NCollection_MapTest__IteratorEquality() {
    let map = integers([1, 2, 3]);
    assert!(map.iter() == map.iter());
    assert!(map.end() == map.end());
    assert!(map.iter() != map.end());
    let first = map.iter();
    let mut second = map.iter();
    second.next();
    assert!(first != second);
    let mut third = map.iter();
    third.next();
    assert!(second == third);
}
