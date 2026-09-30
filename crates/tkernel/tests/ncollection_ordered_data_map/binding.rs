#[test]
fn NCollection_OrderedDataMapTest__FirstLastKey() {
    let map = strings(&[(30, "Thirty"), (10, "Ten"), (20, "Twenty")]);
    assert_eq!(map.first(), Ok(&30));
    assert_eq!(map.last(), Ok(&20));
}

#[test]
fn NCollection_OrderedDataMapTest__FirstLastValue() {
    let mut map = strings(&[(30, "Thirty"), (10, "Ten"), (20, "Twenty")]);
    assert_eq!(map.first_value(), Ok(&AsciiString::new("Thirty")));
    assert_eq!(map.last_value(), Ok(&AsciiString::new("Twenty")));
    *map.change_first_value().unwrap() = AsciiString::new("THIRTY");
    *map.change_last_value().unwrap() = AsciiString::new("TWENTY");
    assert_eq!(map.first_value(), Ok(&AsciiString::new("THIRTY")));
    assert_eq!(map.last_value(), Ok(&AsciiString::new("TWENTY")));
    assert_eq!(map.find(&30), Ok(&AsciiString::new("THIRTY")));
    assert_eq!(map.find(&20), Ok(&AsciiString::new("TWENTY")));
}

#[test]
fn NCollection_OrderedDataMapTest__FirstLastEmpty() {
    let mut map = StringMap::default();
    assert_eq!(
        map.first().unwrap_err().exception_type(),
        "Standard_NoSuchObject"
    );
    assert_eq!(
        map.last().unwrap_err().exception_type(),
        "Standard_NoSuchObject"
    );
    assert_eq!(
        map.first_value().unwrap_err().exception_type(),
        "Standard_NoSuchObject"
    );
    assert_eq!(
        map.last_value().unwrap_err().exception_type(),
        "Standard_NoSuchObject"
    );
    assert_eq!(
        map.change_first_value().unwrap_err().exception_type(),
        "Standard_NoSuchObject"
    );
    assert_eq!(
        map.change_last_value().unwrap_err().exception_type(),
        "Standard_NoSuchObject"
    );
}

#[test]
fn NCollection_OrderedDataMapTest__FirstLastAfterRemove() {
    let mut map = strings(&[(1, "One"), (2, "Two"), (3, "Three")]);
    map.unbind(&1);
    assert_eq!(map.first(), Ok(&2));
    assert_eq!(map.first_value(), Ok(&AsciiString::new("Two")));
    map.unbind(&3);
    assert_eq!(map.last(), Ok(&2));
    assert_eq!(map.last_value(), Ok(&AsciiString::new("Two")));
    assert_eq!(map.first(), map.last());
}

#[test]
fn NCollection_OrderedDataMapTest__TryBind_NewKey() {
    let mut map = StringMap::default();
    assert!(map.try_bind(1, AsciiString::new("One")));
    assert_eq!(map.find(&1), Ok(&AsciiString::new("One")));
    assert_eq!(map.len(), 1);
}

#[test]
fn NCollection_OrderedDataMapTest__TryBind_ExistingKey() {
    let mut map = StringMap::default();
    map.bind(1, AsciiString::new("One"));
    assert!(!map.try_bind(1, AsciiString::new("New One")));
    assert_eq!(map.find(&1), Ok(&AsciiString::new("One")));
    assert_eq!(map.len(), 1);
}

#[test]
fn NCollection_OrderedDataMapTest__TryBound_NewKey() {
    let mut map = StringMap::default();
    assert_eq!(
        map.try_bound(1, AsciiString::new("One")),
        &AsciiString::new("One")
    );
    assert_eq!(map.len(), 1);
    *map.change_find(&1).unwrap() = AsciiString::new("Modified One");
    assert_eq!(map.find(&1), Ok(&AsciiString::new("Modified One")));
}

#[test]
fn NCollection_OrderedDataMapTest__TryBound_ExistingKey() {
    let mut map = StringMap::default();
    map.bind(1, AsciiString::new("One"));
    assert_eq!(
        map.try_bound(1, AsciiString::new("New One")),
        &AsciiString::new("One")
    );
    assert_eq!(map.len(), 1);
}

#[test]
fn NCollection_OrderedDataMapTest__Emplace_NewKey() {
    let mut map = StringMap::default();
    assert!(map.emplace(1, || AsciiString::new("One")));
    assert_eq!(map.find(&1), Ok(&AsciiString::new("One")));
    assert_eq!(map.len(), 1);
}

#[test]
fn NCollection_OrderedDataMapTest__Emplace_ExistingKey() {
    let mut map = StringMap::default();
    map.bind(1, AsciiString::new("One"));
    assert!(!map.emplace(1, || AsciiString::new("New One")));
    assert_eq!(map.find(&1), Ok(&AsciiString::new("New One")));
    assert_eq!(map.len(), 1);
}

#[test]
fn NCollection_OrderedDataMapTest__TryEmplace_NewKey() {
    let mut map = StringMap::default();
    assert!(map.try_emplace(1, || AsciiString::new("One")));
    assert_eq!(map.find(&1), Ok(&AsciiString::new("One")));
    assert_eq!(map.len(), 1);
}

#[test]
fn NCollection_OrderedDataMapTest__TryEmplace_ExistingKey() {
    let mut map = StringMap::default();
    map.bind(1, AsciiString::new("One"));
    assert!(!map.try_emplace(1, || AsciiString::new("New One")));
    assert_eq!(map.find(&1), Ok(&AsciiString::new("One")));
    assert_eq!(map.len(), 1);
}

#[test]
fn NCollection_OrderedDataMapTest__TryBindPreservesOrder() {
    let mut map = StringMap::default();
    for (key, value) in [(30, "Thirty"), (10, "Ten"), (20, "Twenty"), (10, "TEN")] {
        map.try_bind(key, AsciiString::new(value));
    }
    assert_eq!(collect_keys(&map), [30, 10, 20]);
    assert_eq!(map.find(&10), Ok(&AsciiString::new("Ten")));
}

#[test]
fn NCollection_OrderedDataMapTest__Emplaced() {
    let mut map = StringMap::default();
    assert_eq!(
        map.emplaced(1, || AsciiString::new("One")),
        &AsciiString::new("One")
    );
    assert_eq!(map.len(), 1);
}

#[test]
fn NCollection_OrderedDataMapTest__TryEmplaced() {
    let mut map = StringMap::default();
    assert_eq!(
        map.try_emplaced(1, || AsciiString::new("One")),
        &AsciiString::new("One")
    );
    assert_eq!(
        map.try_emplaced(1, || AsciiString::new("New One")),
        &AsciiString::new("One")
    );
}
