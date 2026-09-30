#[test]
fn TCollection_AsciiStringTest__Insert_BasicCases() {
    let mut string = AsciiString::new("HelloWorld");
    string.insert(1, "Start").unwrap();
    text(&string, "StartHelloWorld");
    let mut second = AsciiString::new("AC");
    second.insert(2, "B").unwrap();
    text(&second, "ABC");
    let mut third = AsciiString::new("Hello");
    third.insert(6, " World").unwrap();
    text(&third, "Hello World");
}
#[test]
fn TCollection_AsciiStringTest__Insert_EmptyStrings() {
    let mut string = AsciiString::default();
    string.insert(1, "First").unwrap();
    text(&string, "First");
    assert_eq!(string.len(), 5);
    let mut second = AsciiString::new("Test");
    second.insert(3, "").unwrap();
    text(&second, "Test");
    assert_eq!(second.len(), 4);
}
#[test]
fn TCollection_AsciiStringTest__Insert_SelfReference() {
    let mut string = AsciiString::new("XYZ");
    string.insert_self(1, 0..string.len()).unwrap();
    text(&string, "XYZXYZ");
    assert_eq!(string.len(), 6);
    let mut second = AsciiString::new("ABCD");
    second.insert_self(2, 1..second.len()).unwrap();
    text(&second, "ABCDBCD");
    assert_eq!(second.len(), 7);
}
#[test]
fn TCollection_AsciiStringTest__Insert_WithCharacter() {
    let mut string = AsciiString::new("Hllo");
    string.insert_char(2, b'e').unwrap();
    text(&string, "Hello");
    assert_eq!(string.len(), 5);
    let mut second = AsciiString::new("ello");
    second.insert_char(1, b'H').unwrap();
    text(&second, "Hello");
}
#[test]
fn TCollection_AsciiStringTest__Insert_BoundaryConditions() {
    let original = AsciiString::new("Test");
    let mut first = original.clone();
    first.insert(1, ">").unwrap();
    text(&first, ">Test");
    let mut second = original.clone();
    second.insert(5, "<").unwrap();
    text(&second, "Test<");
}
#[test]
fn TCollection_AsciiStringTest__Insert_OverlapLeftSide() {
    let mut string = AsciiString::new("ABCDEFGH");
    string.insert_self(4, 0..3).unwrap();
    text(&string, "ABCABCDEFGH");
    assert_eq!(string.len(), 11);
}
#[test]
fn TCollection_AsciiStringTest__Insert_OverlapRightSide() {
    let mut string = AsciiString::new("ABCDEFGH");
    string.insert_self(2, 3..8).unwrap();
    text(&string, "ADEFGHBCDEFGH");
    assert_eq!(string.len(), 13);
}
#[test]
fn TCollection_AsciiStringTest__Insert_OverlapInsideShiftWindow() {
    let mut string = AsciiString::new("ABCDEFGH");
    string.insert_self(2, 2..5).unwrap();
    text(&string, "ACDEBCDEFGH");
    assert_eq!(string.len(), 11);
}
#[test]
fn TCollection_AsciiStringTest__Insert_OverlapStartsBeforeShiftWindow() {
    let mut string = AsciiString::new("ABCDEFGH");
    string.insert_self(4, 1..5).unwrap();
    text(&string, "ABCBCDEDEFGH");
    assert_eq!(string.len(), 12);
}
#[test]
fn TCollection_AsciiStringTest__Insert_OverlapAtInsertionPoint() {
    let mut string = AsciiString::new("ABCDEFGH");
    string.insert_self(4, 3..6).unwrap();
    text(&string, "ABCDEFDEFGH");
    assert_eq!(string.len(), 11);
}
#[test]
fn TCollection_AsciiStringTest__Insert_OverlapComplex1() {
    let mut string = AsciiString::new("123456789");
    string.insert_self(1, 3..6).unwrap();
    text(&string, "456123456789");
    assert_eq!(string.len(), 12);
}
#[test]
fn TCollection_AsciiStringTest__Insert_OverlapComplex2() {
    let mut string = AsciiString::new("XYZABC");
    string.insert_self(4, 0..2).unwrap();
    text(&string, "XYZXYABC");
    assert_eq!(string.len(), 8);
}
#[test]
fn TCollection_AsciiStringTest__Insert_OverlapSpansEntireShiftWindow() {
    let mut string = AsciiString::new("ABCDEFGHIJK");
    string.insert_self(5, 1..8).unwrap();
    text(&string, "ABCDBCDEFGHEFGHIJK");
    assert_eq!(string.len(), 18);
}
#[test]
fn TCollection_AsciiStringTest__Insert_OverlapSingleChar() {
    let mut string = AsciiString::new("ABC");
    string.insert_self(1, 1..2).unwrap();
    text(&string, "BABC");
    assert_eq!(string.len(), 4);
}
#[test]
fn TCollection_AsciiStringTest__Insert_OverlapEndToBeginning() {
    let mut string = AsciiString::new("HELLO");
    string.insert_self(1, 3..5).unwrap();
    text(&string, "LOHELLO");
    assert_eq!(string.len(), 7);
}
#[test]
fn TCollection_AsciiStringTest__Insert_OverlapLargeString() {
    let mut string = AsciiString::filled(50, b'A');
    string.set_value(25, "MARKER").unwrap();
    string.insert_self(10, 20..30).unwrap();
    assert_eq!(string.len(), 60);
    assert!(contains(&string, "MARKER"));
}
