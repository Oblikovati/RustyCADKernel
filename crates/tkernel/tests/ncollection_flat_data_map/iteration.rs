#[test]
fn NCollection_FlatDataMapTest__HasherConstructorCopy() {
    let hasher = StatefulHasher { salt: 42 };
    let mut map = FlatDataMap::with_hasher(hasher.clone(), 10);
    map.bind(1, AsciiString::new("One"));
    map.bind(2, AsciiString::new("Two"));
    assert_eq!(map.extent(), 2);
    assert_eq!(map.find(&1), Ok(&AsciiString::new("One")));
    assert_eq!(map.find(&2), Ok(&AsciiString::new("Two")));
    assert_eq!(map.hasher().salt, 42);
}

#[test]
fn NCollection_FlatDataMapTest__HasherConstructorMove() {
    let mut map = FlatDataMap::with_hasher(StatefulHasher { salt: 99 }, 10);
    map.bind(10, AsciiString::new("Ten"));
    assert_eq!(map.extent(), 1);
    assert_eq!(map.hasher().salt, 99);
}

#[test]
fn NCollection_FlatDataMapTest__CopyConstructorPreservesHasher() {
    let mut first = FlatDataMap::with_hasher(StatefulHasher { salt: 123 }, 10);
    first.bind(1, AsciiString::new("One"));
    let second = first.clone();
    assert_eq!(second.hasher().salt, 123);
    assert_eq!(second.extent(), 1);
    assert_eq!(second.find(&1), Ok(&AsciiString::new("One")));
}

#[test]
fn NCollection_FlatDataMapTest__ItemsIteration() {
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
fn NCollection_FlatDataMapTest__ItemsStructuredBindings() {
    let map = strings(&[(10, "Ten"), (20, "Twenty")]);
    let mut sum = 0;
    for (key, _value) in map.items() {
        sum += key;
    }
    assert_eq!(sum, 30);
}

#[test]
fn NCollection_FlatDataMapTest__ItemsModifyValue() {
    let mut map = strings(&[(1, "Original")]);
    for (_key, value) in map.items_mut() {
        *value = AsciiString::new("Modified");
    }
    assert_eq!(map.find(&1), Ok(&AsciiString::new("Modified")));
}

#[test]
fn NCollection_FlatDataMapTest__ConstItemsIteration() {
    let map = strings(&[(1, "One"), (2, "Two")]);
    let shared = &map;
    let mut count = 0;
    for (_key, _value) in shared.items() {
        count += 1;
    }
    assert_eq!(count, 2);
}

#[test]
fn NCollection_FlatDataMapTest__ItemsIteratorEquality() {
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

#[test]
fn NCollection_FlatDataMapTest__IteratorAccess() {
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
