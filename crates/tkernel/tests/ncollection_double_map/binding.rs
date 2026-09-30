#[test]
fn NCollection_DoubleMapTest__TryBind_NewKeys() {
    let mut map = NumericMap::default();
    assert!(map.try_bind(10, 1.0));
    assert_eq!(map.extent(), 1);
    assert!(map.is_bound1(&10));
    assert!(map.is_bound2(&1.0));
    assert_eq!(map.find1(&10), Ok(&1.0));
    assert_eq!(map.find2(&1.0), Ok(&10));
    assert!(map.try_bind(20, 2.0));
    assert_eq!(map.extent(), 2);
    assert!(map.is_bound1(&20));
    assert!(map.is_bound2(&2.0));
}

#[test]
fn NCollection_DoubleMapTest__TryBind_ExistingKey1() {
    let mut map = numeric([10]);
    assert_eq!(map.extent(), 1);
    assert!(!map.try_bind(10, 999.0));
    assert_eq!(map.extent(), 1);
    assert_eq!(map.find1(&10), Ok(&1.0));
    assert!(!map.is_bound2(&999.0));
}

#[test]
fn NCollection_DoubleMapTest__TryBind_ExistingKey2() {
    let mut map = numeric([10]);
    assert_eq!(map.extent(), 1);
    assert!(!map.try_bind(999, 1.0));
    assert_eq!(map.extent(), 1);
    assert_eq!(map.find2(&1.0), Ok(&10));
    assert!(!map.is_bound1(&999));
}

#[test]
fn NCollection_DoubleMapTest__TryBind_BothKeysExist() {
    let mut map = numeric([10, 20]);
    assert_eq!(map.extent(), 2);
    assert!(!map.try_bind(10, 2.0));
    assert_eq!(map.extent(), 2);
    assert_eq!(map.find1(&10), Ok(&1.0));
    assert_eq!(map.find2(&2.0), Ok(&20));
}

#[test]
fn NCollection_DoubleMapTest__TryBind_MoveSemantics() {
    let mut map = DoubleMap::<AsciiString, AsciiString>::default();
    assert!(map.try_bind(AsciiString::new("Key1"), AsciiString::new("Value1")));
    assert_eq!(map.extent(), 1);
    assert_eq!(
        map.find1(&AsciiString::new("Key1")),
        Ok(&AsciiString::new("Value1"))
    );
    assert!(!map.try_bind(AsciiString::new("Key1"), AsciiString::new("NewValue")));
    assert_eq!(map.extent(), 1);
    assert_eq!(
        map.find1(&AsciiString::new("Key1")),
        Ok(&AsciiString::new("Value1"))
    );
}

#[test]
fn NCollection_DoubleMapTest__TryBind_VsBindException() {
    let mut map = numeric([10]);
    assert_eq!(
        map.bind(10, 2.0).unwrap_err().exception_type(),
        "Standard_MultiplyDefined"
    );
    assert_eq!(
        map.bind(20, 1.0).unwrap_err().exception_type(),
        "Standard_MultiplyDefined"
    );
    assert!(!map.try_bind(10, 3.0));
    assert!(!map.try_bind(30, 1.0));
    assert_eq!(map.extent(), 1);
}

#[test]
fn NCollection_DoubleMapTest__TryEmplace_NewKey() {
    let mut map = NumericMap::default();
    assert!(map.try_emplace(10, 1.0));
    assert_eq!(map.extent(), 1);
    assert_eq!(map.find1(&10), Ok(&1.0));
}

#[test]
fn NCollection_DoubleMapTest__TryEmplace_ExistingKey() {
    let mut map = numeric([10]);
    assert!(!map.try_emplace(10, 2.0));
    assert!(!map.try_emplace(20, 1.0));
    assert_eq!(map.extent(), 1);
}
