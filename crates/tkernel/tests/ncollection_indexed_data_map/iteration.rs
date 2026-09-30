#[test]
fn NCollection_IndexedDataMapTest__HasherConstructorCopy() {
    let hasher = StatefulHasher { salt: 42 };
    let mut map = IndexedDataMap::with_hasher(hasher.clone(), 10);
    map.add(1, AsciiString::new("One"));
    map.add(2, AsciiString::new("Two"));
    assert_eq!(map.extent(), 2);
    assert_eq!(map.find_from_index(1), Ok(&AsciiString::new("One")));
    assert_eq!(map.find_from_index(2), Ok(&AsciiString::new("Two")));
    assert_eq!(map.hasher().salt, 42);
}

#[test]
fn NCollection_IndexedDataMapTest__HasherConstructorMove() {
    let mut map = IndexedDataMap::with_hasher(StatefulHasher { salt: 99 }, 10);
    map.add(10, AsciiString::new("Ten"));
    assert_eq!(map.extent(), 1);
    assert_eq!(map.hasher().salt, 99);
}

#[test]
fn NCollection_IndexedDataMapTest__CopyConstructorPreservesHasher() {
    let mut first = IndexedDataMap::with_hasher(StatefulHasher { salt: 123 }, 10);
    first.add(1, AsciiString::new("One"));
    let second = first.clone();
    assert_eq!(second.hasher().salt, 123);
    assert_eq!(second.extent(), 1);
    assert_eq!(second.find_from_index(1), Ok(&AsciiString::new("One")));
}

#[test]
fn NCollection_IndexedDataMapTest__ItemsIteration() {
    let map = strings(&[(1, "One"), (2, "Two"), (3, "Three")]);
    let mut keys = BTreeSet::new();
    let mut values = BTreeSet::new();
    for (key, value) in map.items() {
        keys.insert(*key);
        values.insert(value.as_bytes());
    }
    assert_eq!(keys.len(), 3);
    for key in [1, 2, 3] {
        assert!(keys.contains(&key));
    }
    assert_eq!(values.len(), 3);
    for value in ["One", "Two", "Three"] {
        assert!(values.contains(value.as_bytes()));
    }
}

#[test]
fn NCollection_IndexedDataMapTest__ItemsStructuredBindings() {
    let map = strings(&[(10, "Ten"), (20, "Twenty")]);
    let mut sum = 0;
    for (key, _value) in map.items() {
        sum += key;
    }
    assert_eq!(sum, 30);
}

#[test]
fn NCollection_IndexedDataMapTest__ItemsModifyValue() {
    let mut map = strings(&[(1, "Original")]);
    for (_key, value) in map.items_mut() {
        *value = AsciiString::new("Modified");
    }
    assert_eq!(map.find_from_index(1), Ok(&AsciiString::new("Modified")));
}

#[test]
fn NCollection_IndexedDataMapTest__ConstItemsIteration() {
    let map = strings(&[(1, "One"), (2, "Two")]);
    let shared = &map;
    let mut count = 0;
    for (_key, _value) in shared.items() {
        count += 1;
    }
    assert_eq!(count, 2);
}

#[test]
fn NCollection_IndexedDataMapTest__IndexedItemsIteration() {
    let map = strings(&[(10, "Ten"), (20, "Twenty"), (30, "Thirty")]);
    let mut key_sum = 0;
    let mut index_sum = 0;
    for (key, _value, index) in map.indexed_items() {
        key_sum += key;
        index_sum += index;
    }
    assert_eq!(key_sum, 60);
    assert_eq!(index_sum, 6);
}

#[test]
fn NCollection_IndexedDataMapTest__ConstIndexedItemsIteration() {
    let map = strings(&[(1, "One"), (2, "Two")]);
    let shared = &map;
    let mut sum = 0;
    for (_key, _value, index) in shared.indexed_items() {
        sum += index;
    }
    assert_eq!(sum, 3);
}

#[test]
fn NCollection_IndexedDataMapTest__StandardIteratorContract() {
    let mut map = strings(&[(1, "one"), (2, "two")]);
    let shared = &map;
    assert!(map.iter() == shared.iter());
    assert!(map.end() == shared.end());
    let _shared_contract: &AsciiString = map.iter().next().unwrap();
    let _exclusive_contract: &mut AsciiString = map.iter_mut().next().unwrap();
    for value in &mut map {
        value.assign_cat("!");
    }
    assert_eq!(map.find_from_index(1), Ok(&AsciiString::new("one!")));
    assert_eq!(map.find_from_index(2), Ok(&AsciiString::new("two!")));
}

#[test]
fn NCollection_IndexedDataMapTest__ItemsIteratorEquality() {
    let map = strings(&[(1, "One"), (2, "Two"), (3, "Three")]);
    assert!(map.items() == map.items());
    assert!(map.items_end() == map.items_end());
    assert!(map.items() != map.items_end());
    let first = map.items();
    let mut second = map.items();
    second.next();
    assert!(first != second);
    let mut third = map.items();
    third.next();
    assert!(second == third);
}
