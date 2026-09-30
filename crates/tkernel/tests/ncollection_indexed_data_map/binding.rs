#[test]
fn NCollection_IndexedDataMapTest__TryBound_NewKey() {
    let mut map = NumericMap::default();
    for (i, key) in [10, 20].into_iter().enumerate() {
        let value = key as f64 / 10.0;
        assert_eq!(*map.try_bound(key, value), value);
        assert_eq!(map.find_from_key(&key), Ok(&value));
        assert_eq!(map.extent(), i + 1);
        assert_eq!(map.find_index(&key), i + 1);
    }
    *map.change_from_key(&10).unwrap() = 1.5;
    assert_eq!(map.find_from_key(&10), Ok(&1.5));
}

#[test]
fn NCollection_IndexedDataMapTest__TryBound_ExistingKey() {
    let mut map = numeric([10]);
    assert_eq!(map.find_from_key(&10), Ok(&1.0));
    assert_eq!(*map.try_bound(10, 999.0), 1.0);
    assert_eq!(map.find_from_key(&10), Ok(&1.0));
    assert_eq!(map.extent(), 1);
    assert_eq!(map.find_index(&10), 1);
    *map.change_from_key(&10).unwrap() = 1.5;
    assert_eq!(map.find_from_key(&10), Ok(&1.5));
}

#[test]
fn NCollection_IndexedDataMapTest__TryBound_MoveSemantics() {
    let mut map = IndexedDataMap::<AsciiString, f64>::default();
    assert_eq!(*map.try_bound(AsciiString::new("Key1"), 1.0), 1.0);
    assert_eq!(map.extent(), 1);
    assert_eq!(*map.try_bound(AsciiString::new("Key1"), 999.0), 1.0);
    assert_eq!(map.extent(), 1);
}

#[test]
fn NCollection_IndexedDataMapTest__Bound_NewKey() {
    let mut map = NumericMap::default();
    for (i, key) in [10, 20].into_iter().enumerate() {
        let value = key as f64 / 10.0;
        assert_eq!(*map.bound(key, value), value);
        assert_eq!(map.find_from_key(&key), Ok(&value));
        assert_eq!(map.extent(), i + 1);
        assert_eq!(map.find_index(&key), i + 1);
    }
    *map.change_from_key(&10).unwrap() = 1.5;
    assert_eq!(map.find_from_key(&10), Ok(&1.5));
}

#[test]
fn NCollection_IndexedDataMapTest__Bound_ExistingKey() {
    let mut map = numeric([10]);
    assert_eq!(map.find_from_key(&10), Ok(&1.0));
    assert_eq!(*map.bound(10, 999.0), 999.0);
    assert_eq!(map.find_from_key(&10), Ok(&999.0));
    assert_eq!(map.extent(), 1);
    assert_eq!(map.find_index(&10), 1);
    *map.change_from_key(&10).unwrap() = 1.5;
    assert_eq!(map.find_from_key(&10), Ok(&1.5));
}

#[test]
fn NCollection_IndexedDataMapTest__Bound_MoveSemantics() {
    let mut map = IndexedDataMap::<AsciiString, f64>::default();
    assert_eq!(*map.bound(AsciiString::new("Key1"), 1.0), 1.0);
    assert_eq!(map.extent(), 1);
    assert_eq!(*map.bound(AsciiString::new("Key1"), 999.0), 999.0);
    assert_eq!(map.extent(), 1);
}

#[test]
fn NCollection_IndexedDataMapTest__TryBound_ReferenceValidity() {
    let mut map = NumericMap::default();
    let mut addresses = Vec::new();
    for key in [10, 20, 30] {
        let value = map.try_bound(key, key as f64 / 10.0);
        assert_eq!(*value, key as f64 / 10.0);
        addresses.push(value as *const f64);
    }
    for (i, address) in addresses.into_iter().enumerate() {
        assert_eq!(address, map.change_from_index(i + 1).unwrap() as *const f64);
    }
}

#[test]
fn NCollection_IndexedDataMapTest__Bind_NewKey() {
    let mut map = NumericMap::default();
    for (i, key) in [10, 20].into_iter().enumerate() {
        let value = key as f64 / 10.0;
        assert!(map.bind(key, value));
        assert_eq!(map.find_from_key(&key), Ok(&value));
        assert_eq!(map.extent(), i + 1);
        assert_eq!(map.find_index(&key), i + 1);
    }
}

#[test]
fn NCollection_IndexedDataMapTest__Bind_ExistingKey() {
    let mut map = NumericMap::default();
    map.bind(10, 1.0);
    assert_eq!(map.find_from_key(&10), Ok(&1.0));
    assert!(!map.bind(10, 999.0));
    assert_eq!(map.find_from_key(&10), Ok(&999.0));
    assert_eq!(map.extent(), 1);
    assert_eq!(map.find_index(&10), 1);
}

#[test]
fn NCollection_IndexedDataMapTest__Bind_MoveSemantics() {
    let mut map = IndexedDataMap::<AsciiString, f64>::default();
    assert!(map.bind(AsciiString::new("Key1"), 1.0));
    assert_eq!(map.extent(), 1);
    assert_eq!(map.find_from_key(&AsciiString::new("Key1")), Ok(&1.0));
    assert!(!map.bind(AsciiString::new("Key1"), 999.0));
    assert_eq!(map.extent(), 1);
    assert_eq!(map.find_from_key(&AsciiString::new("Key1")), Ok(&999.0));
}

#[test]
fn NCollection_IndexedDataMapTest__Bind_Bound_VsAdd_Behavior() {
    let mut map = numeric([10]);
    assert_eq!(map.find_from_key(&10), Ok(&1.0));
    map.add(10, 999.0);
    assert_eq!(map.find_from_key(&10), Ok(&1.0));
    map.bind(10, 2.0);
    assert_eq!(map.find_from_key(&10), Ok(&2.0));
    map.bound(10, 3.0);
    assert_eq!(map.find_from_key(&10), Ok(&3.0));
    map.try_bound(10, 999.0);
    assert_eq!(map.find_from_key(&10), Ok(&3.0));
}

#[test]
fn NCollection_IndexedDataMapTest__Emplace_NewKey() {
    let mut map = IndexedDataMap::<i32, AsciiString>::default();
    assert_eq!(map.emplace(1, || AsciiString::new("One")), 1);
    assert_eq!(map.find_from_key(&1), Ok(&AsciiString::new("One")));
    assert_eq!(map.extent(), 1);
}

#[test]
fn NCollection_IndexedDataMapTest__Emplace_ExistingKey() {
    let mut map = IndexedDataMap::<i32, AsciiString>::default();
    map.add(1, AsciiString::new("One"));
    assert_eq!(map.emplace(1, || AsciiString::new("New One")), 1);
    assert_eq!(map.find_from_key(&1), Ok(&AsciiString::new("New One")));
    assert_eq!(map.extent(), 1);
}

#[test]
fn NCollection_IndexedDataMapTest__Emplaced_NewKey() {
    let mut map = IndexedDataMap::<i32, AsciiString>::default();
    assert_eq!(
        map.emplaced(1, || AsciiString::new("One")),
        &AsciiString::new("One")
    );
    assert_eq!(map.extent(), 1);
}

#[test]
fn NCollection_IndexedDataMapTest__Emplaced_ExistingKey() {
    let mut map = IndexedDataMap::<i32, AsciiString>::default();
    map.add(1, AsciiString::new("One"));
    assert_eq!(
        map.emplaced(1, || AsciiString::new("New One")),
        &AsciiString::new("New One")
    );
}

#[test]
fn NCollection_IndexedDataMapTest__TryEmplace_NewKey() {
    let mut map = IndexedDataMap::<i32, AsciiString>::default();
    assert_eq!(map.try_emplace(1, || AsciiString::new("One")), 1);
    assert_eq!(map.find_from_key(&1), Ok(&AsciiString::new("One")));
    assert_eq!(map.extent(), 1);
}

#[test]
fn NCollection_IndexedDataMapTest__TryEmplace_ExistingKey() {
    let mut map = IndexedDataMap::<i32, AsciiString>::default();
    map.add(1, AsciiString::new("One"));
    assert_eq!(map.try_emplace(1, || AsciiString::new("New One")), 1);
    assert_eq!(map.find_from_key(&1), Ok(&AsciiString::new("One")));
    assert_eq!(map.extent(), 1);
}

#[test]
fn NCollection_IndexedDataMapTest__TryEmplaced_NewKey() {
    let mut map = IndexedDataMap::<i32, AsciiString>::default();
    assert_eq!(
        map.try_emplaced(1, || AsciiString::new("One")),
        &AsciiString::new("One")
    );
    assert_eq!(map.extent(), 1);
}

#[test]
fn NCollection_IndexedDataMapTest__TryEmplaced_ExistingKey() {
    let mut map = IndexedDataMap::<i32, AsciiString>::default();
    map.add(1, AsciiString::new("One"));
    assert_eq!(
        map.try_emplaced(1, || AsciiString::new("New One")),
        &AsciiString::new("One")
    );
}
