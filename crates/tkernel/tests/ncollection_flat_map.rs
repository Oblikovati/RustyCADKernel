//! Robin Hood set behavior from OCCT's complete FlatMap suite.
#![allow(non_snake_case)]
use rustycad_tkernel::ncollection::{FlatMap, HashPolicy};
use rustycad_tkernel::tcollection::AsciiString;
use std::collections::BTreeSet;
fn integers(values: impl IntoIterator<Item = i32>) -> FlatMap<i32> {
    let mut map = FlatMap::default();
    for value in values {
        map.add(value);
    }
    map
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
#[derive(Clone, Default)]
struct ConstantHasher;
impl HashPolicy<i32> for ConstantHasher {
    fn hash(&self, _key: &i32) -> usize {
        0
    }
    fn equal(&self, left: &i32, right: &i32) -> bool {
        left == right
    }
}

#[test]
fn NCollection_FlatMapTest__DefaultConstructor() {
    let map = FlatMap::<i32>::default();
    assert!(map.is_empty());
    assert_eq!(map.len(), 0);
}

#[test]
fn NCollection_FlatMapTest__AddAndContains() {
    let mut map = FlatMap::<i32>::default();
    for key in [1, 2, 3] {
        assert!(map.add(key));
    }
    assert!(!map.add(1));
    assert_eq!(map.len(), 3);
    for key in [1, 2, 3] {
        assert!(map.contains(&key));
    }
    assert!(!map.contains(&4));
}

#[test]
fn NCollection_FlatMapTest__Remove() {
    let mut map = integers([1, 2, 3]);
    assert!(map.remove(&2));
    assert_eq!(map.len(), 2);
    assert!(!map.contains(&2));
    assert!(!map.remove(&4));
    assert_eq!(map.len(), 2);
}

#[test]
fn NCollection_FlatMapTest__Clear() {
    let mut map = integers([1, 2]);
    map.clear(false);
    assert!(map.is_empty());
}

#[test]
fn NCollection_FlatMapTest__CopyConstructor() {
    let first = integers([1, 2, 3]);
    let second = first.clone();
    assert_eq!(first.len(), second.len());
    for key in [1, 2, 3] {
        assert!(second.contains(&key));
    }
}

#[test]
fn NCollection_FlatMapTest__MoveConstructor() {
    let mut first = integers([1, 2]);
    let second = FlatMap::move_from(&mut first);
    assert_eq!(second.len(), 2);
    assert!(first.is_empty());
}

#[test]
fn NCollection_FlatMapTest__Iterator() {
    let map = integers([10, 20, 30]);
    let found: BTreeSet<i32> = map.iter().copied().collect();
    assert_eq!(found.len(), 3);
    for key in [10, 20, 30] {
        assert!(found.contains(&key));
    }
}

#[test]
fn NCollection_FlatMapTest__STLIterators() {
    let map = integers([10, 20, 30]);
    let mut sum = 0;
    for key in &map {
        sum += key;
    }
    assert_eq!(sum, 60);
    let shared = &map;
    let found = shared.iter().find(|&&key| key == 20);
    assert!(found.is_some());
    assert_eq!(found, Some(&20));
}

#[test]
fn NCollection_FlatMapTest__LargeDataSet() {
    let map = integers(0..10000);
    assert_eq!(map.len(), 10000);
    for key in 0..10000 {
        assert!(map.contains(&key), "Key {key} not found");
    }
}

#[test]
fn NCollection_FlatMapTest__LongProbeSequence() {
    let mut map = FlatMap::<i32, ConstantHasher>::default();
    for key in 0..400 {
        assert!(map.add(key));
    }
    assert_eq!(map.len(), 400);
    for key in 0..400 {
        assert!(map.contains(&key), "Key {key} not found");
    }
}

#[test]
fn NCollection_FlatMapTest__StringKeys() {
    let mut map = FlatMap::<AsciiString>::default();
    for key in ["Hello", "World", "Test"] {
        map.add(AsciiString::new(key));
    }
    assert_eq!(map.len(), 3);
    for key in ["Hello", "World", "Test"] {
        assert!(map.contains(&AsciiString::new(key)));
    }
    assert!(!map.contains(&AsciiString::new("NotFound")));
}

#[test]
fn NCollection_FlatMapTest__AddedNewKey() {
    let mut map = FlatMap::<AsciiString>::default();
    assert_eq!(
        map.added(AsciiString::new("Hello")),
        &AsciiString::new("Hello")
    );
    assert_eq!(map.len(), 1);
}

#[test]
fn NCollection_FlatMapTest__AddedExistingKey() {
    let mut map = FlatMap::<AsciiString>::default();
    map.add(AsciiString::new("Hello"));
    assert_eq!(
        map.added(AsciiString::new("Hello")),
        &AsciiString::new("Hello")
    );
    assert_eq!(map.len(), 1);
}

#[test]
fn NCollection_FlatMapTest__EmplaceNewKey() {
    let mut map = FlatMap::<AsciiString>::default();
    assert!(map.emplace(AsciiString::new("Hello")));
    assert_eq!(map.len(), 1);
    assert!(map.contains(&AsciiString::new("Hello")));
}

#[test]
fn NCollection_FlatMapTest__EmplaceExistingKey() {
    let mut map = FlatMap::<AsciiString>::default();
    map.add(AsciiString::new("Hello"));
    assert!(!map.emplace(AsciiString::new("Hello")));
    assert_eq!(map.len(), 1);
    assert!(map.contains(&AsciiString::new("Hello")));
}

#[test]
fn NCollection_FlatMapTest__EmplacedNewKey() {
    let mut map = FlatMap::<AsciiString>::default();
    assert_eq!(
        map.emplaced(AsciiString::new("World")),
        &AsciiString::new("World")
    );
    assert_eq!(map.len(), 1);
}

#[test]
fn NCollection_FlatMapTest__EmplacedExistingKey() {
    let mut map = FlatMap::<AsciiString>::default();
    map.add(AsciiString::new("World"));
    assert_eq!(
        map.emplaced(AsciiString::new("World")),
        &AsciiString::new("World")
    );
    assert_eq!(map.len(), 1);
}

#[test]
fn NCollection_FlatMapTest__SeekFound() {
    let map = integers([10, 20, 30]);
    assert_eq!(map.seek(&10), Some(&10));
    assert_eq!(map.seek(&30), Some(&30));
}

#[test]
fn NCollection_FlatMapTest__SeekNotFound() {
    let map = integers([10]);
    assert_eq!(map.seek(&99), None);
    let empty = FlatMap::<i32>::default();
    assert_eq!(empty.seek(&10), None);
}

#[test]
fn NCollection_FlatMapTest__ChangeSeekModify() {
    let mut map = FlatMap::<AsciiString>::default();
    map.add(AsciiString::new("Hello"));
    map.add(AsciiString::new("World"));
    assert_eq!(
        map.change_seek(&AsciiString::new("Hello")),
        Some(&mut AsciiString::new("Hello"))
    );
}

#[test]
fn NCollection_FlatMapTest__ChangeSeekNotFound() {
    let mut map = integers([10]);
    assert_eq!(map.change_seek(&99), None);
}

#[test]
fn NCollection_FlatMapTest__HasherConstructorCopy() {
    let hasher = StatefulHasher { salt: 42 };
    let mut map = FlatMap::with_hasher(hasher.clone(), 10);
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
fn NCollection_FlatMapTest__HasherConstructorMove() {
    let mut map = FlatMap::with_hasher(StatefulHasher { salt: 99 }, 10);
    map.add(10);
    map.add(20);
    assert_eq!(map.len(), 2);
    assert_eq!(map.hasher().salt, 99);
}

#[test]
fn NCollection_FlatMapTest__CopyConstructorPreservesHasher() {
    let mut first = FlatMap::with_hasher(StatefulHasher { salt: 123 }, 10);
    first.add(1);
    first.add(2);
    let second = first.clone();
    assert_eq!(second.hasher().salt, 123);
    assert_eq!(second.len(), 2);
    assert!(second.contains(&1));
    assert!(second.contains(&2));
}

#[test]
fn NCollection_FlatMapTest__IteratorEquality() {
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
