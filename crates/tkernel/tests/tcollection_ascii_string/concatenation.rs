#[test]
fn TCollection_AsciiStringTest__AssignCat_BasicCases() {
    let mut string = AsciiString::new("Hello");
    string.assign_cat(" World");
    text(&string, "Hello World");
    assert_eq!(string.len(), 11);
    string.assign_cat(AsciiString::new("!"));
    text(&string, "Hello World!");
    assert_eq!(string.len(), 12);
    string.assign_cat(b'?');
    text(&string, "Hello World!?");
    assert_eq!(string.len(), 13);
}
#[test]
fn TCollection_AsciiStringTest__AssignCat_EmptyStrings() {
    let mut string = AsciiString::default();
    string.assign_cat("First");
    text(&string, "First");
    assert_eq!(string.len(), 5);
    string.assign_cat("");
    text(&string, "First");
    assert_eq!(string.len(), 5);
    let mut second = AsciiString::new("Test");
    second.assign_cat(0_u8);
    text(&second, "Test");
    assert_eq!(second.len(), 4);
}
#[test]
fn TCollection_AsciiStringTest__AssignCat_SelfReference() {
    let mut string = AsciiString::new("ABC");
    string.assign_cat_self(0..string.len());
    text(&string, "ABCABC");
    assert_eq!(string.len(), 6);
    let mut second = AsciiString::new("Hello");
    second.assign_cat_self(1..second.len());
    text(&second, "Helloello");
    assert_eq!(second.len(), 9);
}
fn contains(string: &AsciiString, pattern: &str) -> bool {
    string
        .as_bytes()
        .windows(pattern.len())
        .any(|part| part == pattern.as_bytes())
}
#[test]
fn TCollection_AsciiStringTest__AssignCat_IntegerAndReal() {
    let mut string = AsciiString::new("Value: ");
    string.assign_cat(42);
    text(&string, "Value: 42");
    let mut second = AsciiString::new("Pi is approximately ");
    second.assign_cat(3.14159);
    assert!(contains(&second, "3.14159"));
}
#[test]
fn TCollection_AsciiStringTest__AssignCat_ExtendedStringAndWideChar() {
    let mut target = AsciiString::new("Value:");
    let source = ExtendedString::new(" OK");
    target += &source;
    text(&target, "Value: OK");
    let non_ascii = ExtendedString::from_utf16(&[32, 0x20AC, 0]);
    let mut replaced = AsciiString::new("Value");
    replaced.assign_cat(AsciiString::from_extended(&non_ascii, Some(b'?')));
    text(&replaced, "Value ?");
    let mut wide = AsciiString::new("Hello");
    wide.assign_cat_wide(&[32, 87, 111, 114, 108, 100, 0]);
    text(&wide, "Hello World");
}
#[test]
fn TCollection_AsciiStringTest__AssignCat_LargeStrings() {
    let mut string = AsciiString::filled(100, b'A');
    let suffix = AsciiString::filled(100, b'B');
    string.assign_cat(&suffix);
    assert_eq!(string.len(), 200);
    assert!(string.as_bytes()[..100].iter().all(|&byte| byte == b'A'));
    assert!(string.as_bytes()[100..200].iter().all(|&byte| byte == b'B'));
}
#[test]
fn TCollection_AsciiStringTest__Cat_BasicCases() {
    let string = AsciiString::new("Hello");
    text(&string.cat(" World"), "Hello World");
    text(&string, "Hello");
    text(&string.cat(AsciiString::new("!")), "Hello!");
    text(&string.cat(b'?'), "Hello?");
}
#[test]
fn TCollection_AsciiStringTest__Cat_IntegerAndReal() {
    let string = AsciiString::new("Count: ");
    text(&string.cat(42), "Count: 42");
    text(&string.cat(-100), "Count: -100");
    assert!(contains(&AsciiString::new("Value: ").cat(3.14), "3.14"));
    text(&string.cat(0_i32), "Count: 0");
}
#[test]
fn TCollection_AsciiStringTest__Cat_ExtendedStringAndWideChar() {
    let string = AsciiString::new("Value:");
    text(&(&string + &ExtendedString::new(" OK")), "Value: OK");
    let accent = ExtendedString::from_utf16(&[32, 0xE9, 0]);
    let utf8 = AsciiString::new("Cafe").cat(&accent);
    assert_eq!(utf8.len(), 7);
    assert_eq!(utf8.as_bytes()[4], b' ');
    assert_eq!(utf8.as_bytes()[5], 0xC3);
    assert_eq!(utf8.as_bytes()[6], 0xA9);
    text(&string.cat_wide(&[32, 87, 105, 100, 101, 0]), "Value: Wide");
    text(&string, "Value:");
}
#[test]
fn TCollection_AsciiStringTest__Cat_EmptyStrings() {
    text(&AsciiString::default().cat("First"), "First");
    text(&AsciiString::new("Test").cat(""), "Test");
}
#[test]
fn TCollection_AsciiStringTest__Cat_ChainedOperations() {
    text(&AsciiString::new("A").cat("B").cat("C").cat("D"), "ABCD");
}
