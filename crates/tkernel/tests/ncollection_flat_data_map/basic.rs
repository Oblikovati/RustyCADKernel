#[test]
fn NCollection_FlatDataMapTest__DefaultConstructor() {
    let map = StringMap::default();
    assert!(map.is_empty());
    assert_eq!(map.len(), 0);
    assert_eq!(map.extent(), 0);
}

#[test]
fn NCollection_FlatDataMapTest__BindingAndAccess() {
    let mut map = StringMap::default();
    for (key, value) in [(1, "One"), (2, "Two"), (3, "Three")] {
        assert!(map.bind(key, AsciiString::new(value)));
    }
    assert!(!map.is_empty());
    assert_eq!(map.len(), 3);
    for (key, value) in [(1, "One"), (2, "Two"), (3, "Three")] {
        assert_eq!(map.find(&key), Ok(&AsciiString::new(value)));
        assert!(map.is_bound(&key));
    }
    assert!(!map.is_bound(&4));
}

#[test]
fn NCollection_FlatDataMapTest__SeekMethod() {
    let map = strings(&[(1, "One")]);
    assert_eq!(map.seek(&1), Some(&AsciiString::new("One")));
    assert_eq!(map.seek(&2), None);
}

#[test]
fn NCollection_FlatDataMapTest__ChangeSeekMethod() {
    let mut map = strings(&[(1, "One")]);
    *map.change_seek(&1).unwrap() = AsciiString::new("Modified One");
    assert_eq!(map.find(&1), Ok(&AsciiString::new("Modified One")));
}

#[test]
fn NCollection_FlatDataMapTest__Rebind() {
    let mut map = StringMap::default();
    assert!(map.bind(1, AsciiString::new("One")));
    assert_eq!(map.find(&1), Ok(&AsciiString::new("One")));
    assert!(!map.bind(1, AsciiString::new("New One")));
    assert_eq!(map.find(&1), Ok(&AsciiString::new("New One")));
    assert_eq!(map.len(), 1);
}

#[test]
fn NCollection_FlatDataMapTest__UnBind() {
    let mut map = strings(&[(1, "One"), (2, "Two"), (3, "Three")]);
    assert!(map.unbind(&2));
    assert_eq!(map.len(), 2);
    assert!(!map.is_bound(&2));
    assert!(!map.unbind(&4));
    assert_eq!(map.len(), 2);
}

#[test]
fn NCollection_FlatDataMapTest__Clear() {
    let mut map = strings(&[(1, "One"), (2, "Two")]);
    map.clear(false);
    assert!(map.is_empty());
    assert_eq!(map.len(), 0);
    assert!(!map.is_bound(&1));
    assert!(!map.is_bound(&2));
    assert!(map.capacity() > 0);
}

#[test]
fn NCollection_FlatDataMapTest__ClearWithRelease() {
    let mut map = strings(&[(1, "One"), (2, "Two")]);
    map.clear(true);
    assert!(map.is_empty());
    assert_eq!(map.capacity(), 0);
}

#[test]
fn NCollection_FlatDataMapTest__CopyConstructor() {
    let mut first = strings(&[(1, "One"), (2, "Two")]);
    let second = first.clone();
    assert_eq!(first.len(), second.len());
    for key in [1, 2] {
        assert_eq!(first.find(&key), second.find(&key));
    }
    first.bind(1, AsciiString::new("Modified One"));
    assert_eq!(first.find(&1), Ok(&AsciiString::new("Modified One")));
    assert_eq!(second.find(&1), Ok(&AsciiString::new("One")));
}

#[test]
fn NCollection_FlatDataMapTest__MoveConstructor() {
    let mut first = strings(&[(1, "One"), (2, "Two")]);
    let second = StringMap::move_from(&mut first);
    assert_eq!(second.len(), 2);
    assert_eq!(second.find(&1), Ok(&AsciiString::new("One")));
    assert_eq!(second.find(&2), Ok(&AsciiString::new("Two")));
    assert!(first.is_empty());
}

#[test]
fn NCollection_FlatDataMapTest__Assignment() {
    let first = strings(&[(1, "One"), (2, "Two")]);
    let mut second = StringMap::default();
    second.assign(&first);
    assert_eq!(first.len(), second.len());
    for key in [1, 2] {
        assert_eq!(first.find(&key), second.find(&key));
    }
}

#[test]
fn NCollection_FlatDataMapTest__MoveAssignment() {
    let mut first = strings(&[(1, "One"), (2, "Two")]);
    let mut second = StringMap::default();
    second.move_assign(&mut first);
    assert_eq!(second.len(), 2);
    assert!(first.is_empty());
}

#[test]
fn NCollection_FlatDataMapTest__Find_NonExisting() {
    let mut map = strings(&[(1, "One")]);
    assert_eq!(
        map.find(&2).unwrap_err().exception_type(),
        "Standard_NoSuchObject"
    );
    assert_eq!(
        map.change_find(&2).unwrap_err().exception_type(),
        "Standard_NoSuchObject"
    );
}

#[test]
fn NCollection_FlatDataMapTest__STLIterators() {
    let mut map = FlatDataMap::<i32, i32>::default();
    for key in [1, 2, 3] {
        map.bind(key, key * 10);
    }
    for value in &mut map {
        *value /= 10;
    }
    let shared = &map;
    let found = shared.iter().find(|&&value| value == 2);
    assert!(found.is_some());
    assert_eq!(found, Some(&2));
}

#[test]
fn NCollection_FlatDataMapTest__LargeDataSet() {
    let mut map = FlatDataMap::<i32, i32>::default();
    for key in 0..10000 {
        map.bind(key, key * 2);
    }
    assert_eq!(map.len(), 10000);
    for key in 0..10000 {
        assert!(map.is_bound(&key));
        assert_eq!(map.find(&key), Ok(&(key * 2)));
    }
    let mut count = 0;
    for (key, value) in map.items() {
        assert_eq!(*key * 2, *value);
        count += 1;
    }
    assert_eq!(count, 10000);
}

#[test]
fn NCollection_FlatDataMapTest__LongProbeSequence() {
    let mut map = FlatDataMap::<i32, i32, ConstantHasher>::default();
    for key in 0..400 {
        assert!(map.bind(key, key * 3));
    }
    assert_eq!(map.len(), 400);
    for key in 0..400 {
        assert!(map.is_bound(&key));
        assert_eq!(map.find(&key), Ok(&(key * 3)));
    }
}

#[test]
fn NCollection_FlatDataMapTest__UnBindAndRebind() {
    let mut map = FlatDataMap::<i32, i32>::default();
    for key in 0..100 {
        map.bind(key, key);
    }
    for key in (0..100).step_by(2) {
        assert!(map.unbind(&key));
    }
    assert_eq!(map.len(), 50);
    for key in (0..100).step_by(2) {
        map.bind(key, key * 10);
    }
    assert_eq!(map.len(), 100);
    for key in 0..100 {
        let expected = if key % 2 == 0 { key * 10 } else { key };
        assert_eq!(map.find(&key), Ok(&expected));
    }
}

#[test]
fn NCollection_FlatDataMapTest__IntegerKeyIntegerValue() {
    let mut map = FlatDataMap::<i32, i32>::default();
    for key in [10, 20, 30] {
        map.bind(key, key * 10);
    }
    for key in [10, 20, 30] {
        assert_eq!(map[&key], key * 10);
    }
}
