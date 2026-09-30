#[test]
fn TCollection_AsciiStringTest__Copy_BasicCases() {
    let mut string = AsciiString::new("Original");
    string.copy("NewValue");
    text(&string, "NewValue");
    assert_eq!(string.len(), 8);
    let source = AsciiString::new("Another");
    string.copy(&source);
    text(&string, "Another");
    assert_eq!(string.len(), 7);
}
#[test]
fn TCollection_AsciiStringTest__Copy_ShorterAndLonger() {
    let mut string = AsciiString::new("LongString");
    string.copy("Short");
    text(&string, "Short");
    assert_eq!(string.len(), 5);
    string.copy("VeryLongStringHere");
    text(&string, "VeryLongStringHere");
    assert_eq!(string.len(), 18);
}
#[test]
fn TCollection_AsciiStringTest__Copy_EmptyString() {
    let mut string = AsciiString::new("Original");
    string.copy("");
    text(&string, "");
    assert_eq!(string.len(), 0);
    assert!(string.is_empty());
}
#[test]
fn TCollection_AsciiStringTest__Copy_SelfAssignment() {
    let mut string = AsciiString::new("Test");
    string.copy_self(0..string.len());
    text(&string, "Test");
    assert_eq!(string.len(), 4);
}
#[test]
fn TCollection_AsciiStringTest__Copy_MemoryReuse() {
    let mut string = AsciiString::filled(100, b'A');
    string.copy("Short");
    text(&string, "Short");
    assert_eq!(string.len(), 5);
    string.copy("Tiny");
    text(&string, "Tiny");
    assert_eq!(string.len(), 4);
}
#[test]
fn TCollection_AsciiStringTest__SetValue_BasicCases() {
    let mut string = AsciiString::new("Hello");
    string.set_value_char(1, b'h').unwrap();
    text(&string, "hello");
    string.set_value_char(5, b'!').unwrap();
    text(&string, "hell!");
}
#[test]
fn TCollection_AsciiStringTest__SetValue_WithString() {
    let mut string = AsciiString::new("AAAAA");
    string.set_value(2, "XYZ").unwrap();
    text(&string, "AXYZA");
    let replacement = AsciiString::new("12");
    string.set_value(1, &replacement).unwrap();
    text(&string, "12YZA");
}
#[test]
fn TCollection_AsciiStringTest__SetValue_BoundaryConditions() {
    let mut string = AsciiString::new("Test");
    string.set_value_char(1, b'X').unwrap();
    text(&string, "Xest");
    string.set_value_char(4, b'Y').unwrap();
    text(&string, "XesY");
}
#[test]
fn TCollection_AsciiStringTest__SetValue_StringOverflow() {
    let mut string = AsciiString::new("AAAA");
    string.set_value(3, "XYZ").unwrap();
    text(&string, "AAXYZ");
    assert_eq!(string.len(), 5);
}
#[test]
fn TCollection_AsciiStringTest__Remove_BasicCases() {
    let mut string = AsciiString::new("Hello World");
    string.remove(6, 6).unwrap();
    text(&string, "Hello");
    assert_eq!(string.len(), 5);
    let mut second = AsciiString::new("XYZTest");
    second.remove(1, 3).unwrap();
    text(&second, "Test");
    let mut third = AsciiString::new("TestXYZ");
    third.remove(5, 3).unwrap();
    text(&third, "Test");
}
#[test]
fn TCollection_AsciiStringTest__Remove_EntireString() {
    let mut string = AsciiString::new("Test");
    string.remove(1, 4).unwrap();
    text(&string, "");
    assert_eq!(string.len(), 0);
    assert!(string.is_empty());
}
#[test]
fn TCollection_AsciiStringTest__RemoveAll_SingleCharacter() {
    let mut string = AsciiString::new("Mississippi");
    string.remove_all(b's');
    text(&string, "Miiippi");
    string.remove_all(b'i');
    text(&string, "Mpp");
}
#[test]
fn TCollection_AsciiStringTest__RemoveAll_NoOccurrence() {
    let mut string = AsciiString::new("Test");
    string.remove_all(b'X');
    text(&string, "Test");
    assert_eq!(string.len(), 4);
}
#[test]
fn TCollection_AsciiStringTest__RemoveAll_AllCharacters() {
    let mut string = AsciiString::new("AAAA");
    string.remove_all(b'A');
    text(&string, "");
    assert_eq!(string.len(), 0);
    assert!(string.is_empty());
}
#[test]
fn TCollection_AsciiStringTest__Trunc_BasicCases() {
    let mut string = AsciiString::new("Hello World");
    string.truncate(5).unwrap();
    text(&string, "Hello");
    assert_eq!(string.len(), 5);
    string.truncate(0).unwrap();
    text(&string, "");
    assert_eq!(string.len(), 0);
    assert!(string.is_empty());
}
#[test]
fn TCollection_AsciiStringTest__Trunc_NoChange() {
    let mut string = AsciiString::new("Test");
    string.truncate(4).unwrap();
    text(&string, "Test");
    assert_eq!(string.len(), 4);
}
