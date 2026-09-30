#[test]
fn NCollection_DataMapTest__IntegerKeys() {
    let map = StringMap::default();
    assert!(map.is_empty());
    assert_eq!(map.len(), 0);
    assert_eq!(map.extent(), 0);
}
#[test]
fn NCollection_DataMapTest__BindingAndAccess() {
    let map = string_map(&[(1, "One"), (2, "Two"), (3, "Three")]);
    assert!(!map.is_empty());
    assert_eq!(map.len(), 3);
    for (key, value) in [(1, "One"), (2, "Two"), (3, "Three")] {
        check(map.find(&key).unwrap(), value);
        assert!(map.is_bound(&key));
    }
    assert!(!map.is_bound(&4));
}
#[test]
fn NCollection_DataMapTest__ChangeFind() {
    let mut map = string_map(&[(1, "One")]);
    check(map.find(&1).unwrap(), "One");
    map.change_find(&1).unwrap().copy("Modified One");
    check(map.find(&1).unwrap(), "Modified One");
}
#[test]
fn NCollection_DataMapTest__Rebind() {
    let mut map = string_map(&[(1, "One")]);
    check(map.find(&1).unwrap(), "One");
    map.bind(1, AsciiString::new("New One"));
    check(map.find(&1).unwrap(), "New One");
    assert_eq!(map.len(), 1);
}
#[test]
fn NCollection_DataMapTest__UnBind() {
    let mut map = string_map(&[(1, "One"), (2, "Two"), (3, "Three")]);
    assert!(map.unbind(&2));
    assert_eq!(map.len(), 2);
    assert!(!map.is_bound(&2));
    assert!(!map.unbind(&4));
    assert_eq!(map.len(), 2);
}
#[test]
fn NCollection_DataMapTest__Clear() {
    let mut map = string_map(&[(1, "One"), (2, "Two")]);
    map.clear(true);
    assert!(map.is_empty());
    assert_eq!(map.len(), 0);
    assert!(!map.is_bound(&1));
    assert!(!map.is_bound(&2));
}
#[test]
fn NCollection_DataMapTest__Assignment() {
    let mut first = string_map(&[(1, "One"), (2, "Two")]);
    let mut second = StringMap::default();
    second.assign(&first);
    assert_eq!(first.len(), second.len());
    assert_eq!(first.find(&1).unwrap(), second.find(&1).unwrap());
    assert_eq!(first.find(&2).unwrap(), second.find(&2).unwrap());
    first.change_find(&1).unwrap().copy("Modified One");
    check(first.find(&1).unwrap(), "Modified One");
    check(second.find(&1).unwrap(), "One");
}
#[test]
fn NCollection_DataMapTest__Find_NonExisting() {
    let mut map = string_map(&[(1, "One")]);
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
fn NCollection_DataMapTest__TryBind_NewKey() {
    let mut map = StringMap::default();
    assert!(map.try_bind(1, AsciiString::new("One")));
    assert_eq!(map.len(), 1);
    check(map.find(&1).unwrap(), "One");
    assert!(map.try_bind(2, AsciiString::new("Two")));
    assert_eq!(map.len(), 2);
    check(map.find(&2).unwrap(), "Two");
}
#[test]
fn NCollection_DataMapTest__TryBind_ExistingKey() {
    let mut map = string_map(&[(1, "One")]);
    check(map.find(&1).unwrap(), "One");
    assert!(!map.try_bind(1, AsciiString::new("New One")));
    assert_eq!(map.len(), 1);
    check(map.find(&1).unwrap(), "One");
}
#[test]
fn NCollection_DataMapTest__TryBind_MoveSemantics() {
    let mut map = StringMap::default();
    let key = 1;
    assert!(map.try_bind(key, AsciiString::new("One")));
    check(map.find(&1).unwrap(), "One");
    let value = AsciiString::new("Two");
    assert!(map.try_bind(2, value));
    check(map.find(&2).unwrap(), "Two");
    assert!(map.try_bind(3, AsciiString::new("Three")));
    check(map.find(&3).unwrap(), "Three");
}
#[test]
fn NCollection_DataMapTest__TryBound_NewKey() {
    let mut map = StringMap::default();
    let reference = map.try_bound(1, AsciiString::new("One"));
    check(reference, "One");
    let address = reference as *const AsciiString;
    check(map.find(&1).unwrap(), "One");
    assert_eq!(map.len(), 1);
    assert_eq!(map.change_find(&1).unwrap() as *const AsciiString, address);
    map.change_find(&1).unwrap().copy("Modified One");
    check(map.find(&1).unwrap(), "Modified One");
}
#[test]
fn NCollection_DataMapTest__TryBound_ExistingKey() {
    let mut map = string_map(&[(1, "One")]);
    let reference = map.try_bound(1, AsciiString::new("New One"));
    check(reference, "One");
    let address = reference as *const AsciiString;
    check(map.find(&1).unwrap(), "One");
    assert_eq!(map.len(), 1);
    assert_eq!(map.change_find(&1).unwrap() as *const AsciiString, address);
    map.change_find(&1).unwrap().copy("Modified One");
    check(map.find(&1).unwrap(), "Modified One");
}
#[test]
fn NCollection_DataMapTest__TryBound_MoveSemantics() {
    let mut map = StringMap::default();
    check(map.try_bound(1, AsciiString::new("One")), "One");
    check(map.try_bound(2, AsciiString::new("Two")), "Two");
    check(map.try_bound(1, AsciiString::new("New One")), "One");
}
#[test]
fn NCollection_DataMapTest__HasherConstructorCopy() {
    let hasher = StatefulHasher { salt: 42 };
    let mut map = DataMap::with_hasher(hasher.clone(), 10);
    map.bind(1, AsciiString::new("One"));
    map.bind(2, AsciiString::new("Two"));
    assert_eq!(map.len(), 2);
    check(map.find(&1).unwrap(), "One");
    check(map.find(&2).unwrap(), "Two");
    assert_eq!(map.hasher().salt, 42);
}
#[test]
fn NCollection_DataMapTest__HasherConstructorMove() {
    let mut map = DataMap::with_hasher(StatefulHasher { salt: 99 }, 10);
    map.bind(10, AsciiString::new("Ten"));
    assert_eq!(map.len(), 1);
    assert_eq!(map.hasher().salt, 99);
}
#[test]
fn NCollection_DataMapTest__CopyConstructorPreservesHasher() {
    let mut first = DataMap::with_hasher(StatefulHasher { salt: 123 }, 10);
    first.bind(1, AsciiString::new("One"));
    let second = first.clone();
    assert_eq!(second.hasher().salt, 123);
    assert_eq!(second.len(), 1);
    check(second.find(&1).unwrap(), "One");
}
