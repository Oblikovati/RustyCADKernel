#[test]
fn NCollection_OrderedMapTest__STLBeginEnd() {
    let map = integers([10, 20, 30]);
    let expected = [10, 20, 30];
    assert_eq!(collect_keys(&map), expected);
    let shared = &map;
    assert_eq!(collect_keys(shared), expected);
    let found = shared.iter().find(|&&key| key == 20);
    assert!(found.is_some());
    assert_eq!(found, Some(&20));
    assert!(shared.iter().copied().eq(expected));
}

#[test]
fn NCollection_OrderedMapTest__RangeBasedFor() {
    let map = integers([100, 200, 300]);
    let mut order = Vec::new();
    for key in &map {
        order.push(*key);
    }
    assert_eq!(order, [100, 200, 300]);
}

#[test]
fn NCollection_OrderedMapTest__IteratorEquality() {
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

#[test]
fn NCollection_OrderedMapTest__LargeMapOrderPreservation() {
    let map = integers((0..1000).map(|i| i * 3));
    assert_eq!(map.len(), 1000);
    let mut index = 0;
    for key in &map {
        assert_eq!(*key, index * 3);
        index += 1;
    }
    assert_eq!(index, 1000);
}

#[test]
fn NCollection_OrderedMapTest__CustomStatefulHasher() {
    let mut map = OrderedMap::with_hasher(StatefulHasher { salt: 42 }, 10);
    for key in [30, 10, 20] {
        map.add(key);
    }
    assert_eq!(map.len(), 3);
    for key in [10, 20, 30] {
        assert!(map.contains(&key));
    }
    assert_eq!(map.hasher().salt, 42);
    assert_eq!(collect_keys(&map), [30, 10, 20]);
    let copy = map.clone();
    assert_eq!(copy.hasher().salt, 42);
    assert_eq!(collect_keys(&copy), [30, 10, 20]);
}

#[test]
fn NCollection_OrderedMapTest__EmplaceBasic() {
    let mut map = OrderedMap::<AsciiString>::default();
    assert!(map.emplace(AsciiString::new("Hello")));
    assert_eq!(map.len(), 1);
    assert!(map.contains(&AsciiString::new("Hello")));
    assert!(!map.emplace(AsciiString::new("Hello")));
    assert_eq!(map.len(), 1);
}

#[test]
fn NCollection_OrderedMapTest__TryEmplaceBasic() {
    let mut map = OrderedMap::<AsciiString>::default();
    assert!(map.try_emplace(AsciiString::new("Hello")));
    assert_eq!(map.len(), 1);
    assert!(map.contains(&AsciiString::new("Hello")));
    assert!(!map.try_emplace(AsciiString::new("Hello")));
    assert_eq!(map.len(), 1);
}

#[test]
fn NCollection_OrderedMapTest__EmplacedBasic() {
    let mut map = OrderedMap::<AsciiString>::default();
    let first = map.emplaced(AsciiString::new("World"));
    assert_eq!(first, &AsciiString::new("World"));
    let address = first as *const AsciiString;
    assert_eq!(map.len(), 1);
    let second = map.emplaced(AsciiString::new("World"));
    assert_eq!(address, second as *const AsciiString);
}

#[test]
fn NCollection_OrderedMapTest__TryEmplacedBasic() {
    let mut map = OrderedMap::<AsciiString>::default();
    let first = map.try_emplaced(AsciiString::new("World"));
    assert_eq!(first, &AsciiString::new("World"));
    let address = first as *const AsciiString;
    assert_eq!(map.len(), 1);
    let second = map.try_emplaced(AsciiString::new("World"));
    assert_eq!(address, second as *const AsciiString);
}

#[test]
fn NCollection_OrderedMapTest__TryEmplacePreservesOrder() {
    let mut map = OrderedMap::<i32>::default();
    for key in [30, 10, 20, 10] {
        map.try_emplace(key);
    }
    assert_eq!(collect_keys(&map), [30, 10, 20]);
}

#[test]
fn NCollection_OrderedMapTest__ContainedFound() {
    let map = integers([10, 20, 30]);
    assert_eq!(map.contained(&20), Some(&20));
}

#[test]
fn NCollection_OrderedMapTest__ContainedNotFound() {
    let map = integers([10]);
    assert_eq!(map.contained(&99), None);
}

#[test]
fn NCollection_OrderedMapTest__ContainedEmpty() {
    let map = OrderedMap::<i32>::default();
    assert_eq!(map.contained(&1), None);
}

#[test]
fn NCollection_OrderedMapTest__EmplacePreservesOrder() {
    let mut map = OrderedMap::<AsciiString>::default();
    for key in ["Charlie", "Alpha", "Bravo"] {
        map.emplace(AsciiString::new(key));
    }
    let order: Vec<AsciiString> = map.iter().cloned().collect();
    assert_eq!(order.len(), 3);
    for (value, expected) in order.iter().zip(["Charlie", "Alpha", "Bravo"]) {
        assert_eq!(value, &AsciiString::new(expected));
    }
}

#[test]
fn NCollection_OrderedMapTest__NativeIterator() {
    let map = integers([10, 20, 30]);
    let mut iter = map.iter();
    assert!(iter.more());
    assert_eq!(iter.key(), Ok(&10));
    assert_eq!(iter.value(), Ok(&10));
    iter.advance();
    assert!(iter.more());
    assert_eq!(iter.key(), Ok(&20));
    iter.advance();
    assert!(iter.more());
    assert_eq!(iter.key(), Ok(&30));
    iter.advance();
    assert!(!iter.more());
}

#[test]
fn NCollection_OrderedMapTest__IteratorInitialize() {
    let map = integers([10, 20]);
    let mut iter = OrderedMapIter::default();
    assert!(!iter.more());
    iter.initialize(&map);
    assert!(iter.more());
    assert_eq!(iter.key(), Ok(&10));
}

#[test]
fn NCollection_OrderedMapTest__RemoveAndReAdd() {
    let mut map = integers([10, 20, 30]);
    map.remove(&20);
    map.add(40);
    map.add(20);
    assert_eq!(collect_keys(&map), [10, 30, 40, 20]);
}

#[test]
fn NCollection_OrderedMapTest__AutoResize() {
    let mut map = OrderedMap::<i32>::new(2);
    let mut expected = Vec::new();
    for key in 0..50 {
        map.add(key);
        expected.push(key);
    }
    assert_eq!(map.len(), 50);
    assert_eq!(collect_keys(&map), expected);
}
