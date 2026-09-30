#[test]
fn NCollection_OrderedDataMapTest__ItemsStructuredBindings() {
    let map = strings(&[(30, "Thirty"), (10, "Ten"), (20, "Twenty")]);
    let mut keys = Vec::new();
    let mut values = Vec::new();
    for (key, value) in map.items() {
        keys.push(*key);
        values.push(value.as_bytes());
    }
    assert_eq!(keys, [30, 10, 20]);
    assert_eq!(
        values,
        ["Thirty".as_bytes(), "Ten".as_bytes(), "Twenty".as_bytes()]
    );
}

#[test]
fn NCollection_OrderedDataMapTest__ItemsModifyValue() {
    let mut map = strings(&[(1, "Original")]);
    for (_key, value) in map.items_mut() {
        *value = AsciiString::new("Modified");
    }
    assert_eq!(map.find(&1), Ok(&AsciiString::new("Modified")));
}

#[test]
fn NCollection_OrderedDataMapTest__ConstItemsIteration() {
    let map = strings(&[(1, "One"), (2, "Two")]);
    let shared = &map;
    let mut count = 0;
    for (_key, _value) in shared.items() {
        count += 1;
    }
    assert_eq!(count, 2);
}

#[test]
fn NCollection_OrderedDataMapTest__ItemsIteratorEquality() {
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
fn NCollection_OrderedDataMapTest__NativeIterator() {
    let mut map = strings(&[(10, "Ten"), (20, "Twenty"), (30, "Thirty")]);
    let mut cursor = map.cursor();
    assert!(cursor.more());
    assert_eq!(cursor.key(&map), Ok(&10));
    assert_eq!(cursor.value(&map), Ok(&AsciiString::new("Ten")));
    cursor.advance(&map);
    assert!(cursor.more());
    assert_eq!(cursor.key(&map), Ok(&20));
    assert_eq!(cursor.value(&map), Ok(&AsciiString::new("Twenty")));
    *cursor.change_value(&mut map).unwrap() = AsciiString::new("TWENTY");
    assert_eq!(map.find(&20), Ok(&AsciiString::new("TWENTY")));
    cursor.advance(&map);
    assert!(cursor.more());
    assert_eq!(cursor.key(&map), Ok(&30));
    cursor.advance(&map);
    assert!(!cursor.more());
}

#[test]
fn NCollection_OrderedDataMapTest__UnBindAndReAdd() {
    let mut map = strings(&[(10, "Ten"), (20, "Twenty"), (30, "Thirty")]);
    map.unbind(&20);
    map.bind(40, AsciiString::new("Forty"));
    map.bind(20, AsciiString::new("TWENTY"));
    assert_eq!(collect_keys(&map), [10, 30, 40, 20]);
    assert_eq!(map.find(&20), Ok(&AsciiString::new("TWENTY")));
}

#[test]
fn NCollection_OrderedDataMapTest__LargeMapOrderPreservation() {
    let mut map = OrderedDataMap::<i32, i32>::default();
    for key in 0..1000 {
        map.bind(key, key * 2);
    }
    assert_eq!(map.len(), 1000);
    let mut index = 0;
    for (key, value) in map.items() {
        assert_eq!(*key, index);
        assert_eq!(*value, index * 2);
        index += 1;
    }
    assert_eq!(index, 1000);
}

#[test]
fn NCollection_OrderedDataMapTest__STLBeginEnd() {
    let mut map = OrderedDataMap::<i32, i32>::default();
    for key in [30, 10, 20] {
        map.bind(key, key * 10);
    }
    let expected = [300, 100, 200];
    assert_eq!(map.iter().copied().collect::<Vec<_>>(), expected);
    let shared = &map;
    assert_eq!(shared.iter().copied().collect::<Vec<_>>(), expected);
    let found = map.iter().find(|&&value| value == 200);
    assert!(found.is_some());
    assert_eq!(found, Some(&200));
    assert!(map.iter().copied().eq(expected));
}

#[test]
fn NCollection_OrderedDataMapTest__CustomStatefulHasher() {
    let mut map = OrderedDataMap::with_hasher(StatefulHasher { salt: 42 }, 10);
    for (key, value) in [(30, "Thirty"), (10, "Ten"), (20, "Twenty")] {
        map.bind(key, AsciiString::new(value));
    }
    assert_eq!(map.len(), 3);
    assert_eq!(map.hasher().salt, 42);
    assert_eq!(map.find(&10), Ok(&AsciiString::new("Ten")));
    assert_eq!(collect_keys(&map), [30, 10, 20]);
}

#[test]
fn NCollection_OrderedDataMapTest__AutoResize() {
    let mut map = OrderedDataMap::<i32, i32>::new(2);
    let mut expected = Vec::new();
    for key in 0..50 {
        map.bind(key, key * 10);
        expected.push(key);
    }
    assert_eq!(map.len(), 50);
    assert_eq!(collect_keys(&map), expected);
}
