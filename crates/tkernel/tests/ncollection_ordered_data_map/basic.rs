#[test]
fn NCollection_OrderedDataMapTest__DefaultConstructor() {
    let map = StringMap::default();
    assert!(map.is_empty());
    assert_eq!(map.len(), 0);
    assert_eq!(map.extent(), 0);
}

#[test]
fn NCollection_OrderedDataMapTest__FindNonExisting() {
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
fn NCollection_OrderedDataMapTest__BindAndFind() {
    let map = strings(&[(1, "One"), (2, "Two"), (3, "Three")]);
    assert!(!map.is_empty());
    assert_eq!(map.len(), 3);
    for (key, value) in [(1, "One"), (2, "Two"), (3, "Three")] {
        assert_eq!(map.find(&key), Ok(&AsciiString::new(value)));
        assert!(map.is_bound(&key));
    }
    assert!(!map.is_bound(&4));
}

#[test]
fn NCollection_OrderedDataMapTest__BindPreservesOrder() {
    let map = strings(&[(30, "Thirty"), (10, "Ten"), (20, "Twenty")]);
    assert_eq!(collect_keys(&map), [30, 10, 20]);
}

#[test]
fn NCollection_OrderedDataMapTest__BindOverwritesValuePreservesOrder() {
    let mut map = strings(&[(1, "One"), (2, "Two"), (3, "Three")]);
    map.bind(2, AsciiString::new("TWO"));
    assert_eq!(map.len(), 3);
    assert_eq!(map.find(&2), Ok(&AsciiString::new("TWO")));
    assert_eq!(collect_keys(&map), [1, 2, 3]);
}

#[test]
fn NCollection_OrderedDataMapTest__ChangeFind() {
    let mut map = strings(&[(1, "One")]);
    assert_eq!(map.find(&1), Ok(&AsciiString::new("One")));
    *map.change_find(&1).unwrap() = AsciiString::new("Modified One");
    assert_eq!(map.find(&1), Ok(&AsciiString::new("Modified One")));
}

#[test]
fn NCollection_OrderedDataMapTest__OperatorParens() {
    let mut map = strings(&[(1, "One"), (2, "Two")]);
    let shared = &map;
    assert_eq!(shared[&1], AsciiString::new("One"));
    map[&1] = AsciiString::new("Modified");
    assert_eq!(map.find(&1), Ok(&AsciiString::new("Modified")));
}

#[test]
fn NCollection_OrderedDataMapTest__SeekAndChangeSeek() {
    let mut map = strings(&[(1, "One")]);
    assert_eq!(map.seek(&1), Some(&AsciiString::new("One")));
    assert_eq!(map.seek(&99), None);
    *map.change_seek(&1).unwrap() = AsciiString::new("Modified");
    assert_eq!(map.find(&1), Ok(&AsciiString::new("Modified")));
    assert_eq!(map.change_seek(&99), None);
}

#[test]
fn NCollection_OrderedDataMapTest__FindWithCopy() {
    let map = strings(&[(1, "One")]);
    let mut value = AsciiString::default();
    assert!(map.find_copy(&1, &mut value));
    assert_eq!(value, AsciiString::new("One"));
    assert!(!map.find_copy(&99, &mut value));
}

#[test]
fn NCollection_OrderedDataMapTest__ContainedConst_Found() {
    let map = strings(&[(1, "One"), (2, "Two")]);
    let shared = &map;
    let result = shared.contained(&2);
    assert!(result.is_some());
    let (key, value) = result.unwrap();
    assert_eq!(*key, 2);
    assert_eq!(value, &AsciiString::new("Two"));
}

#[test]
fn NCollection_OrderedDataMapTest__ContainedConst_NotFound() {
    let map = strings(&[(1, "One")]);
    let shared = &map;
    assert!(shared.contained(&99).is_none());
}

#[test]
fn NCollection_OrderedDataMapTest__ContainedMutable() {
    let mut map = strings(&[(1, "One"), (2, "Two")]);
    let result = map.contained_mut(&2);
    assert!(result.is_some());
    let (key, value) = result.unwrap();
    assert_eq!(*key, 2);
    assert_eq!(*value, AsciiString::new("Two"));
    *value = AsciiString::new("Modified Two");
    assert_eq!(map.find(&2), Ok(&AsciiString::new("Modified Two")));
}

#[test]
fn NCollection_OrderedDataMapTest__ContainedEmpty() {
    let mut map = StringMap::default();
    let shared = &map;
    assert!(shared.contained(&1).is_none());
    assert!(map.contained_mut(&1).is_none());
}

#[test]
fn NCollection_OrderedDataMapTest__UnBindOrder() {
    let mut map = strings(&[(1, "One"), (2, "Two"), (3, "Three"), (4, "Four")]);
    assert!(map.unbind(&2));
    assert_eq!(map.len(), 3);
    assert!(!map.is_bound(&2));
    assert_eq!(collect_keys(&map), [1, 3, 4]);
    assert!(map.unbind(&1));
    assert_eq!(collect_keys(&map), [3, 4]);
    assert!(map.unbind(&4));
    assert_eq!(collect_keys(&map), [3]);
    assert!(!map.unbind(&99));
}

#[test]
fn NCollection_OrderedDataMapTest__Clear() {
    let mut map = strings(&[(1, "One"), (2, "Two")]);
    map.clear(false);
    assert!(map.is_empty());
    assert_eq!(map.len(), 0);
    assert!(!map.is_bound(&1));
    assert!(!map.is_bound(&2));
}

#[test]
fn NCollection_OrderedDataMapTest__ClearAndReAdd() {
    let mut map = strings(&[(1, "One"), (2, "Two")]);
    map.clear(false);
    map.bind(2, AsciiString::new("Two"));
    map.bind(1, AsciiString::new("One"));
    assert_eq!(collect_keys(&map), [2, 1]);
}

#[test]
fn NCollection_OrderedDataMapTest__CopyPreservesOrder() {
    let mut first = strings(&[(30, "Thirty"), (10, "Ten"), (20, "Twenty")]);
    let second = first.clone();
    assert_eq!(first.len(), second.len());
    assert_eq!(collect_keys(&first), collect_keys(&second));
    assert_eq!(second.find(&10), Ok(&AsciiString::new("Ten")));
    *first.change_find(&10).unwrap() = AsciiString::new("Modified Ten");
    assert_eq!(second.find(&10), Ok(&AsciiString::new("Ten")));
}

#[test]
fn NCollection_OrderedDataMapTest__AssignPreservesOrder() {
    let first = strings(&[(30, "Thirty"), (10, "Ten"), (20, "Twenty")]);
    let mut second = strings(&[(99, "Ninety-Nine")]);
    second.assign(&first);
    assert_eq!(collect_keys(&first), collect_keys(&second));
    assert!(!second.is_bound(&99));
}

#[test]
fn NCollection_OrderedDataMapTest__MoveConstructor() {
    let mut first = strings(&[(10, "Ten"), (20, "Twenty"), (30, "Thirty")]);
    let second = StringMap::move_from(&mut first);
    assert_eq!(second.len(), 3);
    assert_eq!(collect_keys(&second), [10, 20, 30]);
    assert_eq!(second.find(&20), Ok(&AsciiString::new("Twenty")));
    assert!(first.is_empty());
}

#[test]
fn NCollection_OrderedDataMapTest__MoveAssignment() {
    let mut first = strings(&[(10, "Ten"), (20, "Twenty"), (30, "Thirty")]);
    let mut second = StringMap::default();
    second.move_assign(&mut first);
    assert_eq!(second.len(), 3);
    assert_eq!(collect_keys(&second), [10, 20, 30]);
}

#[test]
fn NCollection_OrderedDataMapTest__Exchange() {
    let mut first = strings(&[(10, "Ten"), (20, "Twenty")]);
    let mut second = strings(&[(30, "Thirty"), (40, "Forty"), (50, "Fifty")]);
    first.exchange(&mut second);
    assert_eq!(first.len(), 3);
    assert_eq!(second.len(), 2);
    assert_eq!(collect_keys(&first), [30, 40, 50]);
    assert_eq!(collect_keys(&second), [10, 20]);
    assert_eq!(first.find(&30), Ok(&AsciiString::new("Thirty")));
    assert_eq!(second.find(&10), Ok(&AsciiString::new("Ten")));
}

#[test]
fn NCollection_OrderedDataMapTest__ReSizePreservesOrder() {
    let mut map = OrderedDataMap::<i32, i32>::new(10);
    for key in 0..100 {
        map.bind(key, key * 2);
    }
    assert_eq!(map.len(), 100);
    let before = collect_keys(&map);
    map.resize(200);
    assert_eq!(map.len(), 100);
    assert_eq!(collect_keys(&map), before);
    for key in 0..100 {
        assert_eq!(map.find(&key), Ok(&(key * 2)));
    }
}
