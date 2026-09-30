#[test]
fn TCollection_ExtendedStringTest__StringView_Constructor() {
    let view = units("Hello World");
    let string = ExtendedString::from_raw(&view);
    assert_eq!(string.len(), 11);
    text(&string, "Hello World");
}
#[test]
fn TCollection_ExtendedStringTest__StringView_Assignment() {
    let mut string = ExtendedString::new("Original");
    let view = units("New Value");
    string.copy(&view);
    assert_eq!(string.len(), 9);
    text(&string, "New Value");
}
#[test]
fn TCollection_ExtendedStringTest__StringView_AssignCat() {
    let mut string = ExtendedString::new("Hello");
    let view = units(" World");
    string.assign_cat(view.as_slice());
    assert_eq!(string.len(), 11);
    text(&string, "Hello World");
}
#[test]
fn TCollection_ExtendedStringTest__StringView_Conversion() {
    let string = ExtendedString::new("Test");
    let view = string.as_utf16();
    assert_eq!(view.len(), 4);
    assert_eq!(view[0], 84);
    assert_eq!(view[1], 101);
}
#[test]
fn TCollection_ExtendedStringTest__StringView_EmptyConstructor() {
    assert!(ExtendedString::from_raw(&[]).is_empty());
}
#[test]
fn TCollection_ExtendedStringTest__FillerConstructor_NegativeLength_Throws() {
    assert_eq!(
        ExtendedString::filled(-5, 88).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
}
#[test]
fn TCollection_ExtendedStringTest__FillerConstructor_ZeroLength() {
    let string = ExtendedString::filled(0, 88).unwrap();
    assert!(string.is_empty());
    assert_eq!(string.len(), 0);
}
#[test]
fn TCollection_ExtendedStringTest__AssignCat_SelfAppend() {
    let mut string = ExtendedString::new("Hello");
    string.assign_cat_self(0..string.len());
    assert_eq!(string.len(), 10);
    text(&string, "HelloHello");
}
#[test]
fn TCollection_ExtendedStringTest__AssignCat_SelfAppendMultiple() {
    let mut string = ExtendedString::new("AB");
    string.assign_cat_self(0..string.len());
    string.assign_cat_self(0..string.len());
    assert_eq!(string.len(), 8);
    text(&string, "ABABABAB");
}
#[test]
fn TCollection_ExtendedStringTest__AssignCat_PointerIntoSelf() {
    let mut string = ExtendedString::new("HelloWorld");
    string.assign_cat_self(5..10);
    assert_eq!(string.len(), 15);
    text(&string, "HelloWorldWorld");
}
#[test]
fn TCollection_ExtendedStringTest__AssignCat_PointerIntoSelfBeginning() {
    let mut string = ExtendedString::new("Hello");
    string.assign_cat_self(0..3);
    assert_eq!(string.len(), 8);
    text(&string, "HelloHel");
}
#[test]
fn TCollection_ExtendedStringTest__Prepend_PointerIntoSelf() {
    let mut string = ExtendedString::new("HelloWorld");
    string.prepend_self(5..10);
    assert_eq!(string.len(), 15);
    text(&string, "WorldHelloWorld");
}
#[test]
fn TCollection_ExtendedStringTest__Prepend_PointerIntoSelfBeginning() {
    let mut string = ExtendedString::new("Hello");
    string.prepend_self(0..3);
    assert_eq!(string.len(), 8);
    text(&string, "HelHello");
}
#[test]
fn TCollection_ExtendedStringTest__OCC22744_NonAsciiCharIsNotAsciiAndCanBeUsedAsMapKey() {
    let mut string = ExtendedString::default();
    string.insert(1, 0x0F00_u16).unwrap();
    assert!(!string.is_ascii());
    let mut map = DataMap::<ExtendedString, i32>::default();
    map.bind(string, 0);
    assert_eq!(map.len(), 1);
}
