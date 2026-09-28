// Copyright (c) 2025 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.

//! DataMap assertions retain replacement, nonreplacement and hasher-state contracts.
#![allow(non_snake_case)]
use rustycad_tkernel::ncollection::{HashPolicy, IndexedMap};
use rustycad_tkernel::tcollection::AsciiString;
use std::collections::BTreeSet;
fn integers(values: impl IntoIterator<Item = i32>) -> IndexedMap<i32> {
    let mut map = IndexedMap::default();
    for value in values {
        map.add(value);
    }
    map
}
fn verify_indexed(map: &IndexedMap<i32>, keys: &[i32]) {
    assert_eq!(map.extent(), keys.len());
    for (index, key) in keys.iter().enumerate() {
        assert!(map.contains(key));
        assert_eq!(map.find_index(key), index + 1);
        assert_eq!(map.find_key(index + 1), Ok(key));
        assert_eq!(&map[index + 1], key);
    }
}
fn verify_iteration(map: &IndexedMap<i32>) {
    let mut found = [false; 3];
    let mut count = 0;
    for key in map.iter() {
        if let Some(index) = [10, 20, 30].iter().position(|item| item == key) {
            found[index] = true;
        }
        count += 1;
    }
    assert_eq!(count, 3);
    assert_eq!(found, [true; 3]);
}
fn fixture() -> Vec<i32> {
    include_str!("fixtures/array1_mt19937_seed1.txt")
        .split_whitespace()
        .map(|value| value.parse().unwrap())
        .collect()
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
#[derive(Clone, Debug, PartialEq)]
struct TestKey {
    id: i32,
    name: String,
}
impl TestKey {
    fn new(id: i32, name: &str) -> Self {
        Self {
            id,
            name: name.to_owned(),
        }
    }
}
#[derive(Clone, Default)]
struct TestKeyHasher;
impl HashPolicy<TestKey> for TestKeyHasher {
    fn hash(&self, key: &TestKey) -> usize {
        // OCCT combines std::hash<int> and std::hash<string>; this fixture
        // captures libstdc++ string hashes for the four upstream keys.
        let name_hash: u64 = match key.name.as_str() {
            "One" => 4974007186610801277,
            "Two" => 18140831775427969181,
            "Three" => 7219768605936779428,
            "Four" => 6500357484398430375,
            _ => unreachable!(),
        };
        let words = [key.id as usize, name_hash as usize];
        let bytes: Vec<u8> = words.iter().flat_map(|word| word.to_ne_bytes()).collect();
        AsciiString::from_raw(&bytes).hash_code()
    }
    fn equal(&self, left: &TestKey, right: &TestKey) -> bool {
        left == right
    }
}

#[test]
fn NCollection_IndexedMapTest__DefaultConstructor() {
    let map = IndexedMap::<i32>::default();
    assert!(map.is_empty());
    assert_eq!(map.extent(), 0);
    assert_eq!(map.len(), 0);
}

#[test]
fn NCollection_IndexedMapTest__BasicAddFind() {
    let mut map = IndexedMap::<i32>::default();
    for (i, key) in [10, 20, 30].into_iter().enumerate() {
        assert_eq!(map.add(key), i + 1);
    }
    verify_indexed(&map, &[10, 20, 30]);
    assert!(!map.contains(&40));
}

#[test]
fn NCollection_IndexedMapTest__DuplicateKey() {
    let mut map = IndexedMap::<i32>::default();
    assert_eq!(map.add(10), 1);
    assert_eq!(map.add(20), 2);
    assert_eq!(map.add(10), 1);
    assert_eq!(map.extent(), 2);
}

#[test]
fn NCollection_IndexedMapTest__RemoveLast() {
    let mut map = integers([10, 20, 30]);
    map.remove_last().unwrap();
    verify_indexed(&map, &[10, 20]);
    assert!(!map.contains(&30));
}

#[test]
fn NCollection_IndexedMapTest__RemoveFromIndex() {
    let mut map = integers([10, 20, 30, 40]);
    map.remove_from_index(2).unwrap();
    verify_indexed(&map, &[10, 40, 30]);
    assert!(!map.contains(&20));
}

#[test]
fn NCollection_IndexedMapTest__RemoveKey() {
    let mut map = integers([10, 20, 30]);
    assert!(map.remove_key(&20));
    assert_eq!(map.extent(), 2);
    assert!(map.contains(&10));
    assert!(!map.contains(&20));
    assert!(map.contains(&30));
    assert!(!map.remove_key(&50));
    assert_eq!(map.extent(), 2);
}

#[test]
fn NCollection_IndexedMapTest__Substitute() {
    let mut map = integers([10, 20, 30]);
    map.substitute(2, 25).unwrap();
    verify_indexed(&map, &[10, 25, 30]);
    assert!(!map.contains(&20));
}

#[test]
fn NCollection_IndexedMapTest__Swap() {
    let mut map = integers([10, 20, 30]);
    map.swap(1, 3).unwrap();
    verify_indexed(&map, &[30, 20, 10]);
}

#[test]
fn NCollection_IndexedMapTest__Clear() {
    let mut map = integers([10, 20, 30]);
    assert_eq!(map.extent(), 3);
    map.clear(true);
    assert!(map.is_empty());
    assert_eq!(map.extent(), 0);
    assert_eq!(map.add(40), 1);
    assert_eq!(map.extent(), 1);
}

#[test]
fn NCollection_IndexedMapTest__CopyConstructor() {
    let mut first = integers([10, 20, 30]);
    let second = first.clone();
    verify_indexed(&second, &[10, 20, 30]);
    first.add(40);
    assert_eq!(first.extent(), 4);
    assert_eq!(second.extent(), 3);
    assert!(!second.contains(&40));
}

#[test]
fn NCollection_IndexedMapTest__AssignmentOperator() {
    let first = integers([10, 20]);
    let mut second = integers([30, 40, 50]);
    second.assign(&first);
    verify_indexed(&second, &[10, 20]);
    for key in [30, 40, 50] {
        assert!(!second.contains(&key));
    }
}

#[test]
fn NCollection_IndexedMapTest__Iterator() {
    let map = integers([10, 20, 30]);
    verify_iteration(&map);
}

#[test]
fn NCollection_IndexedMapTest__StlIterator() {
    let map = integers([10, 20, 30]);
    verify_iteration(&map);
}

#[test]
fn NCollection_IndexedMapTest__StringKeys() {
    let mut map = IndexedMap::<AsciiString>::default();
    for (i, key) in ["First", "Second", "Third"].iter().enumerate() {
        assert_eq!(map.add(AsciiString::new(key)), i + 1);
    }
    for (i, key) in ["First", "Second", "Third"].iter().enumerate() {
        assert_eq!(map.find_index(&AsciiString::new(key)), i + 1);
        assert_eq!(map.find_key(i + 1), Ok(&AsciiString::new(key)));
    }
}

#[test]
fn NCollection_IndexedMapTest__ComplexKeys() {
    let mut map = IndexedMap::<TestKey, TestKeyHasher>::default();
    let keys = [
        TestKey::new(1, "One"),
        TestKey::new(2, "Two"),
        TestKey::new(3, "Three"),
    ];
    for (i, key) in keys.iter().enumerate() {
        assert_eq!(map.add(key.clone()), i + 1);
    }
    for (i, key) in keys.iter().enumerate() {
        assert_eq!(map.find_index(key), i + 1);
        assert_eq!(map.find_key(i + 1).unwrap().id, i as i32 + 1);
    }
    assert!(map.contains(&TestKey::new(1, "One")));
    assert!(!map.contains(&TestKey::new(4, "Four")));
}

#[test]
fn NCollection_IndexedMapTest__Exchange() {
    let mut first = integers([10, 20]);
    let mut second = integers([30, 40, 50]);
    first.exchange(&mut second);
    verify_indexed(&first, &[30, 40, 50]);
    verify_indexed(&second, &[10, 20]);
}

#[test]
fn NCollection_IndexedMapTest__ReSize() {
    let mut map = IndexedMap::<i32>::new(3);
    for key in 1..=100 {
        map.add(key);
    }
    let keys: Vec<i32> = (1..=100).collect();
    verify_indexed(&map, &keys);
    map.resize(200);
    verify_indexed(&map, &keys);
}

#[test]
fn NCollection_IndexedMapTest__STLAlgorithmCompatibility_MinMax() {
    let values = fixture();
    let map = integers(values.iter().copied());
    assert_eq!(map.iter().min(), values.iter().min());
    assert_eq!(map.iter().max(), values.iter().max());
}

#[test]
fn NCollection_IndexedMapTest__STLAlgorithmCompatibility_Find() {
    let values = fixture();
    let map = integers(values.iter().copied());
    let target = values[10];
    let actual = map.iter().find(|&&key| key == target);
    let expected = values.iter().find(|&&key| key == target);
    assert!(actual.is_some());
    assert!(expected.is_some());
    assert_eq!(actual, expected);
}

#[test]
fn NCollection_IndexedMapTest__Added_NewKey() {
    let mut map = IndexedMap::<i32>::default();
    for (index, key) in [10, 20].into_iter().enumerate() {
        assert_eq!(map.added(key), &key);
        assert_eq!(map.extent(), index + 1);
        assert!(map.contains(&key));
        assert_eq!(map.find_index(&key), index + 1);
    }
}

#[test]
fn NCollection_IndexedMapTest__Added_ExistingKey() {
    let mut map = integers([10]);
    assert_eq!(map.extent(), 1);
    assert_eq!(map.added(10), &10);
    assert_eq!(map.extent(), 1);
    assert_eq!(map.find_index(&10), 1);
}

#[test]
fn NCollection_IndexedMapTest__Added_MoveSemantics() {
    let mut map = IndexedMap::<AsciiString>::default();
    assert_eq!(
        map.added(AsciiString::new("First")),
        &AsciiString::new("First")
    );
    assert_eq!(map.extent(), 1);
    assert_eq!(
        map.added(AsciiString::new("First")),
        &AsciiString::new("First")
    );
    assert_eq!(map.extent(), 1);
}

#[test]
fn NCollection_IndexedMapTest__Added_ReferenceValidity() {
    let mut map = IndexedMap::<i32>::default();
    let mut addresses = Vec::new();
    for key in [10, 20, 30] {
        let stored = map.added(key);
        assert_eq!(stored, &key);
        addresses.push(stored as *const i32);
    }
    for (index, address) in addresses.into_iter().enumerate() {
        assert_eq!(address, map.find_key(index + 1).unwrap() as *const i32);
    }
}

#[test]
fn NCollection_IndexedMapTest__Emplace_NewKey() {
    let mut map = IndexedMap::<AsciiString>::default();
    assert_eq!(map.emplace(AsciiString::new("First")), 1);
    assert_eq!(map.extent(), 1);
    assert_eq!(map.find_key(1), Ok(&AsciiString::new("First")));
}

#[test]
fn NCollection_IndexedMapTest__Emplace_ExistingKey() {
    let mut map = IndexedMap::<AsciiString>::default();
    map.add(AsciiString::new("First"));
    assert_eq!(map.emplace(AsciiString::new("First")), 1);
    assert_eq!(map.extent(), 1);
}

#[test]
fn NCollection_IndexedMapTest__Emplaced_NewKey() {
    let mut map = IndexedMap::<AsciiString>::default();
    assert_eq!(
        map.emplaced(AsciiString::new("First")),
        &AsciiString::new("First")
    );
    assert_eq!(map.extent(), 1);
}

#[test]
fn NCollection_IndexedMapTest__HasherConstructorCopy() {
    let hasher = StatefulHasher { salt: 42 };
    let mut map = IndexedMap::with_hasher(hasher.clone(), 10);
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
fn NCollection_IndexedMapTest__HasherConstructorMove() {
    let mut map = IndexedMap::with_hasher(StatefulHasher { salt: 99 }, 10);
    map.add(10);
    map.add(20);
    assert_eq!(map.len(), 2);
    assert_eq!(map.hasher().salt, 99);
}

#[test]
fn NCollection_IndexedMapTest__CopyConstructorPreservesHasher() {
    let mut first = IndexedMap::with_hasher(StatefulHasher { salt: 123 }, 10);
    first.add(1);
    first.add(2);
    let second = first.clone();
    assert_eq!(second.hasher().salt, 123);
    assert_eq!(second.len(), 2);
    assert!(second.contains(&1));
    assert!(second.contains(&2));
}

#[test]
fn NCollection_IndexedMapTest__BeginEndIterator() {
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
fn NCollection_IndexedMapTest__RangeBasedForLoop() {
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
fn NCollection_IndexedMapTest__IndexedItemsIteration() {
    let map = integers([100, 200, 300]);
    let mut key_sum = 0;
    let mut index_sum = 0;
    for (key, index) in map.indexed_items() {
        key_sum += key;
        index_sum += index;
    }
    assert_eq!(key_sum, 600);
    assert_eq!(index_sum, 6);
}

#[test]
fn NCollection_IndexedMapTest__IndexedItemsIteratorEquality() {
    let map = integers([100, 200, 300]);
    assert!(map.indexed_items() == map.indexed_items());
    assert!(map.indexed_end() == map.indexed_end());
    assert!(map.indexed_items() != map.indexed_end());
    let first = map.indexed_items();
    let mut second = map.indexed_items();
    second.next();
    assert!(first != second);
    let mut third = map.indexed_items();
    third.next();
    assert!(second == third);
}
