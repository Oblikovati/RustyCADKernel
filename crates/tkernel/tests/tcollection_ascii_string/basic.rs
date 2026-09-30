#[test]
fn TCollection_AsciiStringTest__DefaultConstructor() {
    let string = AsciiString::default();
    assert_eq!(string.len(), 0);
    assert!(string.is_empty());
}
#[test]
fn TCollection_AsciiStringTest__ConstructorWithCString() {
    let string = AsciiString::new("Test String");
    assert_eq!(string.len(), 11);
    assert!(!string.is_empty());
    text(&string, "Test String");
}
#[test]
fn TCollection_AsciiStringTest__ConstructorWithChar() {
    let string = AsciiString::from_char(b'A');
    assert_eq!(string.len(), 1);
    assert!(!string.is_empty());
    text(&string, "A");
}
#[test]
fn TCollection_AsciiStringTest__CopyConstructor() {
    let original = AsciiString::new("Original");
    let copy = original.clone();
    assert_eq!(copy.as_bytes(), original.as_bytes());
    assert_eq!(copy.len(), original.len());
}
#[test]
fn TCollection_AsciiStringTest__AssignmentOperator() {
    let mut first = AsciiString::new("First");
    let mut second = AsciiString::default();
    second.copy(&first);
    assert_eq!(first.as_bytes(), second.as_bytes());
    first.copy("Changed");
    assert_ne!(first.as_bytes(), second.as_bytes());
    text(&second, "First");
}
#[test]
fn TCollection_AsciiStringTest__Concatenation() {
    let mut first = AsciiString::new("Hello");
    let second = AsciiString::new(" World");
    let third = &first + &second;
    text(&third, "Hello World");
    first += &second;
    text(&first, "Hello World");
    first += b'!';
    text(&first, "Hello World!");
    first += " Test";
    text(&first, "Hello World! Test");
}
#[test]
fn TCollection_AsciiStringTest__SubString() {
    text(
        &AsciiString::new("ABCDEFGHIJKLMNOPQRSTUVWXYZ").sub_string(3, 8).unwrap(),
        "CDEFGH",
    );
}
#[test]
fn TCollection_AsciiStringTest__ChangeCase() {
    let original = AsciiString::new("Hello World");
    let mut upper = original.clone();
    upper.upper_case();
    text(&upper, "HELLO WORLD");
    let mut lower = original.clone();
    lower.lower_case();
    text(&lower, "hello world");
    let mut capital = lower.clone();
    capital.capitalize();
    text(&capital, "Hello world");
}
#[test]
fn TCollection_AsciiStringTest__SearchAndLocation() {
    let string = AsciiString::new("This is a test string for testing search functions");
    assert_eq!(string.search("test"), 11);
    assert_eq!(string.search("nonexistent"), -1);
    assert_eq!(string.search_from_end("test"), 27);
    assert_eq!(string.location(AsciiString::new("i"), 1, string.length()).unwrap(), 3);
    assert_eq!(string.location(AsciiString::new("i"), 4, string.length()).unwrap(), 6);
}
#[test]
fn TCollection_AsciiStringTest__RemoveAndInsert() {
    let mut string = AsciiString::new("Hello World");
    string.remove(6, 6).unwrap();
    text(&string, "Hello");
    string.insert(6, " Universe").unwrap();
    text(&string, "Hello Universe");
    string.remove_all(b'e');
    text(&string, "Hllo Univrs");
}
#[test]
fn TCollection_AsciiStringTest__Comparison() {
    let first = AsciiString::new("Test");
    let second = AsciiString::new("Test");
    let third = AsciiString::new("Different");
    assert!(first.is_equal(&second));
    assert!(first == second);
    assert!(!first.is_different(&second));
    assert!(first == second);
    assert!(!first.is_equal(&third));
    assert!(first != third);
    assert!(first.is_different(&third));
    assert!(first != third);
    assert!(first.is_equal("Test"));
    assert!(first == "Test");
    assert!(!first.is_equal("Different"));
    assert!(first != "Different");
    let a = AsciiString::new("A");
    let z = AsciiString::new("Z");
    assert!(a.is_less(&z));
    assert!(a < z);
    assert!(!a.is_greater(&z));
    assert!(a <= z);
    assert!(z.is_greater(&a));
    assert!(z > a);
    assert!(!z.is_less(&a));
    assert!(z >= a);
}
#[test]
fn TCollection_AsciiStringTest__Token() {
    let string = AsciiString::new("This is a test string");
    text(&string.token(" ", 1), "This");
    for (index, expected) in [(2, "is"), (3, "a"), (4, "test"), (5, "string"), (6, "")] {
        text(&string.token(" ", index), expected);
    }
    let string = AsciiString::new("one,two;three:four");
    for (index, expected) in [(1, "one"), (2, "two"), (3, "three"), (4, "four")] {
        text(&string.token(",;:", index), expected);
    }
}
#[test]
fn TCollection_AsciiStringTest__ValueConversion() {
    let integer = AsciiString::new("12345");
    assert_eq!(integer.integer_value().unwrap(), 12345);
    assert!(integer.is_integer_value());
    let real = AsciiString::new("123.456");
    assert_eq!(real.real_value().unwrap(), 123.456);
    assert!(real.is_real_value(false));
    let invalid = AsciiString::new("not a number");
    assert!(!invalid.is_integer_value());
    assert!(!invalid.is_real_value(false));
}
#[test]
fn TCollection_AsciiStringTest__HashCode() {
    let first = AsciiString::new("Test");
    let second = AsciiString::new("Test");
    let third = AsciiString::new("Different");
    assert_eq!(first.hash_code(), second.hash_code());
    assert_ne!(first.hash_code(), third.hash_code());
}
#[test]
fn TCollection_AsciiStringTest__Split() {
    let mut string = AsciiString::new("abcdefghij");
    let remainder = string.split(5).unwrap();
    text(&string, "abcde");
    text(&remainder, "fghij");
}
#[test]
fn TCollection_AsciiStringTest__MemoryAllocation() {
    assert_eq!(AsciiString::new("test").len(), 4);
    for index in 0..=20 {
        assert_eq!(AsciiString::filled(index, b'A').len(), index);
    }
}
#[test]
fn TCollection_AsciiStringTest__LengthConstructor() {
    let source = "This is a very long string";
    let first = AsciiString::with_max_length(source, 4);
    assert_eq!(first.len(), 4);
    text(&first, "This");
    let second = AsciiString::with_max_length(source, 7);
    assert_eq!(second.len(), 7);
    text(&second, "This is");
    let third = AsciiString::with_max_length(source, 100);
    assert_eq!(third.len(), 26);
    text(&third, source);
}
#[test]
fn TCollection_AsciiStringTest__ExtendedStringConversion() {
    let extended = ExtendedString::new("Hello World");
    let ascii = AsciiString::from_extended(&extended, None);
    assert_eq!(ascii.len(), extended.len());
    text(&ascii, "Hello World");
}
#[test]
fn TCollection_AsciiStringTest__NumericalConstructors() {
    text(&AsciiString::from_integer(42), "42");
    assert!(
        AsciiString::from_real(3.14)
            .as_bytes()
            .windows(4)
            .any(|s| s == b"3.14")
    );
}
#[test]
fn TCollection_AsciiStringTest__FillerConstructor() {
    let string = AsciiString::filled(5, b'*');
    assert_eq!(string.len(), 5);
    text(&string, "*****");
}
#[test]
fn TCollection_AsciiStringTest__ConcatenationConstructors() {
    let base = AsciiString::new("Hello");
    text(&base.cat(b'!'), "Hello!");
    text(&base.cat(" World"), "Hello World");
    text(&base.cat(AsciiString::new(" Universe")), "Hello Universe");
}
#[test]
fn TCollection_AsciiStringTest__EdgeCases() {
    let first = AsciiString::default();
    let second = AsciiString::new("");
    assert!(first.is_equal(&second));
    assert_eq!(first.len(), 0);
    assert!(first.is_empty());
    let null = AsciiString::from_char(0);
    assert_eq!(null.len(), 0);
    assert!(null.is_empty());
}
#[test]
fn TCollection_AsciiStringTest__LargeStrings() {
    let string = AsciiString::filled(1000, b'X');
    assert_eq!(string.len(), 1000);
    assert_eq!(string.value(1).unwrap(), b'X');
    assert_eq!(string.value(1000).unwrap(), b'X');
}
#[test]
fn TCollection_AsciiStringTest__PaddingSafety() {
    for length in 1..=16 {
        let string = AsciiString::filled(length, b'A');
        assert_eq!(string.len(), length);
        assert_eq!(string.as_bytes_with_nul()[length], 0);
        assert!(
            string.as_bytes_with_nul()[..length]
                .iter()
                .all(|&byte| byte == b'A')
        );
    }
}
