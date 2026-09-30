#[test]
fn NCollection_OrderedMapTest__DefaultConstructor() {
    let map = OrderedMap::<i32>::default();
    assert!(map.is_empty());
    assert_eq!(map.len(), 0);
    assert_eq!(map.extent(), 0);
}

#[test]
fn NCollection_OrderedMapTest__ConstructorWithBuckets() {
    let map = OrderedMap::<i32>::new(100);
    assert!(map.is_empty());
    assert_eq!(map.len(), 0);
    assert_eq!(map.bucket_count(), 100);
}

#[test]
fn NCollection_OrderedMapTest__AddAndContainsWithOrder() {
    let mut map = OrderedMap::<i32>::default();
    for key in [10, 20, 30] {
        assert!(map.add(key));
    }
    assert_eq!(map.len(), 3);
    for key in [10, 20, 30] {
        assert!(map.contains(&key));
    }
    assert!(!map.contains(&40));
    assert_eq!(collect_keys(&map), [10, 20, 30]);
}

#[test]
fn NCollection_OrderedMapTest__AddDuplicate() {
    let mut map = OrderedMap::<i32>::default();
    for key in [10, 20, 30] {
        assert!(map.add(key));
    }
    for key in [10, 20] {
        assert!(!map.add(key));
    }
    assert_eq!(map.len(), 3);
    assert_eq!(collect_keys(&map), [10, 20, 30]);
}

#[test]
fn NCollection_OrderedMapTest__AddedMethod() {
    let mut map = OrderedMap::<i32>::default();
    let first = map.added(10);
    assert_eq!(*first, 10);
    let address = first as *const i32;
    assert_eq!(*map.added(20), 20);
    let third = map.added(10);
    assert_eq!(*third, 10);
    assert_eq!(third as *const i32, address);
    assert_eq!(map.len(), 2);
}

#[test]
fn NCollection_OrderedMapTest__RemoveFirst() {
    let mut map = integers([10, 20, 30]);
    assert!(map.remove(&10));
    assert_eq!(map.len(), 2);
    assert!(!map.contains(&10));
    assert_eq!(collect_keys(&map), [20, 30]);
}

#[test]
fn NCollection_OrderedMapTest__RemoveMiddle() {
    let mut map = integers([10, 20, 30]);
    assert!(map.remove(&20));
    assert_eq!(map.len(), 2);
    assert!(!map.contains(&20));
    assert_eq!(collect_keys(&map), [10, 30]);
}

#[test]
fn NCollection_OrderedMapTest__RemoveLast() {
    let mut map = integers([10, 20, 30]);
    assert!(map.remove(&30));
    assert_eq!(map.len(), 2);
    assert!(!map.contains(&30));
    assert_eq!(collect_keys(&map), [10, 20]);
}

#[test]
fn NCollection_OrderedMapTest__RemoveAll() {
    let mut map = integers([10, 20, 30]);
    for key in [20, 10, 30] {
        assert!(map.remove(&key));
    }
    assert!(map.is_empty());
    assert_eq!(map.len(), 0);
    assert!(!map.remove(&10));
}

#[test]
fn NCollection_OrderedMapTest__RemoveNonExistent() {
    let mut map = integers([10]);
    assert!(!map.remove(&99));
    assert_eq!(map.len(), 1);
}

#[test]
fn NCollection_OrderedMapTest__ClearAndReAdd() {
    let mut map = integers([10, 20, 30]);
    map.clear(false);
    assert!(map.is_empty());
    for key in [30, 10, 20] {
        map.add(key);
    }
    assert_eq!(collect_keys(&map), [30, 10, 20]);
}

#[test]
fn NCollection_OrderedMapTest__CopyPreservesOrder() {
    let mut first = integers([30, 10, 20]);
    let second = first.clone();
    assert_eq!(first.len(), second.len());
    assert_eq!(collect_keys(&first), collect_keys(&second));
    first.remove(&10);
    assert_eq!(second.len(), 3);
    assert!(second.contains(&10));
}

#[test]
fn NCollection_OrderedMapTest__AssignPreservesOrder() {
    let first = integers([30, 10, 20]);
    let mut second = integers([99]);
    second.assign(&first);
    assert_eq!(collect_keys(&first), collect_keys(&second));
    assert!(!second.contains(&99));
}

#[test]
fn NCollection_OrderedMapTest__MoveConstructor() {
    let mut first = integers([10, 20, 30]);
    let second = OrderedMap::move_from(&mut first);
    assert_eq!(second.len(), 3);
    assert_eq!(collect_keys(&second), [10, 20, 30]);
    assert!(first.is_empty());
}

#[test]
fn NCollection_OrderedMapTest__MoveAssignment() {
    let mut first = integers([10, 20, 30]);
    let mut second = OrderedMap::default();
    second.move_assign(&mut first);
    assert_eq!(second.len(), 3);
    assert_eq!(collect_keys(&second), [10, 20, 30]);
}

#[test]
fn NCollection_OrderedMapTest__Exchange() {
    let mut first = integers([10, 20]);
    let mut second = integers([30, 40, 50]);
    first.exchange(&mut second);
    assert_eq!(first.len(), 3);
    assert_eq!(second.len(), 2);
    assert_eq!(collect_keys(&first), [30, 40, 50]);
    assert_eq!(collect_keys(&second), [10, 20]);
}

#[test]
fn NCollection_OrderedMapTest__ReSizePreservesOrder() {
    let mut map = OrderedMap::<i32>::new(10);
    for key in 0..100 {
        map.add(key);
    }
    assert_eq!(map.len(), 100);
    let before = collect_keys(&map);
    map.resize(200);
    assert_eq!(map.len(), 100);
    assert_eq!(collect_keys(&map), before);
    for key in 0..100 {
        assert!(map.contains(&key));
    }
}

#[test]
fn NCollection_OrderedMapTest__FirstLast() {
    let mut map = integers([30, 10, 20]);
    assert_eq!(map.first(), Ok(&30));
    assert_eq!(map.last(), Ok(&20));
    map.remove(&30);
    assert_eq!(map.first(), Ok(&10));
    assert_eq!(map.last(), Ok(&20));
    map.remove(&20);
    assert_eq!(map.first(), Ok(&10));
    assert_eq!(map.last(), Ok(&10));
}

#[test]
fn NCollection_OrderedMapTest__FirstLastEmpty() {
    let map = OrderedMap::<i32>::default();
    assert_eq!(
        map.first().unwrap_err().exception_type(),
        "Standard_NoSuchObject"
    );
    assert_eq!(
        map.last().unwrap_err().exception_type(),
        "Standard_NoSuchObject"
    );
}

#[test]
fn NCollection_OrderedMapTest__FirstLastSingleElement() {
    let map = integers([42]);
    assert_eq!(map.first(), Ok(&42));
    assert_eq!(map.last(), Ok(&42));
}
