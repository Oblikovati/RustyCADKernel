#[test]
fn NCollection_DataMapTest__Emplace_NewKey() {
    let mut map = StringMap::default();
    assert!(map.emplace(1, || AsciiString::new("One")));
    assert_eq!(map.len(), 1);
    check(map.find(&1).unwrap(), "One");
    assert!(map.emplace(2, || AsciiString::new("Two")));
    assert_eq!(map.len(), 2);
}
#[test]
fn NCollection_DataMapTest__Emplace_ExistingKey() {
    let mut map = string_map(&[(1, "One")]);
    assert!(!map.emplace(1, || AsciiString::new("New One")));
    assert_eq!(map.len(), 1);
    check(map.find(&1).unwrap(), "New One");
}
#[test]
fn NCollection_DataMapTest__Emplaced_NewKey() {
    let mut map = StringMap::default();
    check(map.emplaced(1, || AsciiString::new("One")), "One");
    assert_eq!(map.len(), 1);
}
#[test]
fn NCollection_DataMapTest__Emplaced_ExistingKey() {
    let mut map = string_map(&[(1, "One")]);
    check(map.emplaced(1, || AsciiString::new("New One")), "New One");
}
#[test]
fn NCollection_DataMapTest__TryEmplace_NewKey() {
    let mut map = StringMap::default();
    assert!(map.try_emplace(1, || AsciiString::new("One")));
    assert_eq!(map.len(), 1);
    check(map.find(&1).unwrap(), "One");
}
#[test]
fn NCollection_DataMapTest__TryEmplace_ExistingKey() {
    let mut map = string_map(&[(1, "One")]);
    assert!(!map.try_emplace(1, || AsciiString::new("New One")));
    assert_eq!(map.len(), 1);
    check(map.find(&1).unwrap(), "One");
}
#[test]
fn NCollection_DataMapTest__TryEmplaced_NewKey() {
    let mut map = StringMap::default();
    check(map.try_emplaced(1, || AsciiString::new("One")), "One");
    assert_eq!(map.len(), 1);
}
#[test]
fn NCollection_DataMapTest__TryEmplaced_ExistingKey() {
    let mut map = string_map(&[(1, "One")]);
    check(map.try_emplaced(1, || AsciiString::new("New One")), "One");
}
