#[test]
fn TCollection_AsciiStringTest__StringView_Constructor() {
    let view = &b"Hello World"[..];
    let string = AsciiString::from_view(view);
    text(&string, "Hello World");
    assert_eq!(string.len(), 11);
}
#[test]
fn TCollection_AsciiStringTest__StringView_AssignmentAndComparison() {
    let mut string = AsciiString::default();
    let view = &b"Test"[..];
    string.copy(view);
    text(&string, "Test");
    assert!(string.is_equal(view));
    assert!(!string.is_different(view));
}
#[test]
fn TCollection_AsciiStringTest__StringView_CatAndAssignCat() {
    let mut string = AsciiString::new("Hello");
    let view = &b" World"[..];
    text(&string.cat(view), "Hello World");
    string.assign_cat(view);
    text(&string, "Hello World");
}
#[test]
fn TCollection_AsciiStringTest__StringView_Insert() {
    let mut string = AsciiString::new("HloWorld");
    let view = &b"el"[..];
    string.insert(2, view).unwrap();
    text(&string, "HelloWorld");
}
#[test]
fn TCollection_AsciiStringTest__MoveConstructor() {
    let mut first = AsciiString::new("Original String");
    let second = AsciiString::move_from(&mut first);
    text(&second, "Original String");
    assert_eq!(second.len(), 15);
    assert!(first.is_empty() || first.length() >= 0);
}
#[test]
fn TCollection_AsciiStringTest__MoveAssignment() {
    let mut first = AsciiString::new("First");
    let mut second = AsciiString::new("Second");
    second.move_assign(&mut first);
    text(&second, "First");
    assert_eq!(second.len(), 5);
}
#[test]
fn TCollection_AsciiStringTest__TemplateLiteral_Constructor() {
    let string = AsciiString::new("Literal");
    text(&string, "Literal");
    assert_eq!(string.len(), 7);
}
#[test]
fn TCollection_AsciiStringTest__TemplateLiteral_Assignment() {
    let mut string = AsciiString::default();
    string.copy("Assigned");
    text(&string, "Assigned");
    assert_eq!(string.len(), 8);
}
#[test]
fn TCollection_AsciiStringTest__StressTest_MultipleOperations() {
    let mut string = AsciiString::default();
    for _ in 0..10 {
        string.assign_cat("A");
    }
    assert_eq!(string.len(), 10);
    for _ in 0..5 {
        string.insert(6, "B").unwrap();
    }
    assert_eq!(string.len(), 15);
    string.remove_all(b'B');
    assert_eq!(string.len(), 10);
    text(&string, "AAAAAAAAAA");
}
#[test]
fn TCollection_AsciiStringTest__StressTest_LargeOperations() {
    let mut large = AsciiString::filled(1000, b'X');
    assert_eq!(large.len(), 1000);
    let mut copy = AsciiString::default();
    copy.copy(&large);
    assert_eq!(copy.len(), 1000);
    large.assign_cat(&copy);
    assert_eq!(large.len(), 2000);
}
#[test]
fn TCollection_AsciiStringTest__EdgeCase_NullCharacterHandling() {
    let string = AsciiString::from_char(0);
    assert!(string.is_empty());
    assert_eq!(string.len(), 0);
}
#[test]
fn TCollection_AsciiStringTest__EdgeCase_ConsecutiveOperations() {
    let mut string = AsciiString::new("Test");
    string.copy("A");
    string.copy("BB");
    string.copy("CCC");
    text(&string, "CCC");
    string.assign_cat("D");
    string.assign_cat("E");
    string.assign_cat("F");
    text(&string, "CCCDEF");
}
