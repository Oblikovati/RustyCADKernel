#[test]
fn TCollection_ExtendedStringTest__DefaultConstructor() {
    let string = ExtendedString::default();
    assert_eq!(string.len(), 0);
    assert!(string.is_empty());
}
#[test]
fn TCollection_ExtendedStringTest__ConstructorWithCString() {
    let string = ExtendedString::new("Test String");
    assert_eq!(string.len(), 11);
    assert!(!string.is_empty());
}
#[test]
fn TCollection_ExtendedStringTest__ConstructorWithChar() {
    let string = ExtendedString::from_char(u16::from(b'A'));
    assert_eq!(string.len(), 1);
    assert!(!string.is_empty());
}
#[test]
fn TCollection_ExtendedStringTest__ConstructorWithAsciiString() {
    let ascii = AsciiString::new("ASCII Test");
    let extended = ExtendedString::from_ascii(&ascii);
    assert_eq!(ascii.len(), extended.len());
}
#[test]
fn TCollection_ExtendedStringTest__CopyConstructor() {
    let first = ExtendedString::new("Original");
    let second = first.clone();
    assert_eq!(first.len(), second.len());
    assert!(first.is_equal(&second));
}
#[test]
fn TCollection_ExtendedStringTest__AssignmentOperator() {
    let mut first = ExtendedString::new("First");
    let mut second = ExtendedString::default();
    second.copy(first.as_utf16());
    assert!(first.is_equal(&second));
    first = ExtendedString::new("Changed");
    assert!(!first.is_equal(&second));
}
#[test]
fn TCollection_ExtendedStringTest__Concatenation() {
    let mut first = ExtendedString::new("Hello");
    let second = ExtendedString::new(" World");
    let third = &first + &second;
    assert_eq!(third.len(), 11);
    text(&third, "Hello World");
    first += &second;
    assert!(first.is_equal(&third));
}
#[test]
fn TCollection_ExtendedStringTest__ConversionToFromAscii() {
    let ascii = AsciiString::new("Test String");
    let extended = ExtendedString::from_ascii(&ascii);
    assert_eq!(
        AsciiString::from_extended(&extended, None).as_bytes(),
        ascii.as_bytes()
    );
}
#[test]
fn TCollection_ExtendedStringTest__Comparison() {
    let first = ExtendedString::new("Test");
    let second = ExtendedString::new("Test");
    let third = ExtendedString::new("Different");
    assert!(first.is_equal(&second));
    assert!(first == second);
    assert!(!first.is_different(&second));
    assert!(first == second);
    assert!(!first.is_equal(&third));
    assert!(first != third);
    assert!(first.is_different(&third));
    assert!(first != third);
    let a = ExtendedString::new("A");
    let z = ExtendedString::new("Z");
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
fn TCollection_ExtendedStringTest__UnicodeSupport() {
    let string = ExtendedString::from_utf16(&[0x41, 0xA9, 0x2122, 0x20AC, 0]);
    assert_eq!(string.len(), 4);
    for (i, value) in [0x41, 0xA9, 0x2122, 0x20AC].into_iter().enumerate() {
        assert_eq!(string.value(i as i32 + 1).unwrap(), value);
    }
}
#[test]
fn TCollection_ExtendedStringTest__HashCode() {
    let first = ExtendedString::new("Test");
    let second = ExtendedString::new("Test");
    let third = ExtendedString::new("Different");
    assert_eq!(first.hash_code(), second.hash_code());
    assert_ne!(first.hash_code(), third.hash_code());
}
#[test]
fn TCollection_ExtendedStringTest__Remove() {
    let mut string = ExtendedString::new("Hello World");
    string.remove(6, 6).unwrap();
    text(&string, "Hello");
}
#[test]
fn TCollection_ExtendedStringTest__ToExtString() {
    let string = ExtendedString::new("Test String");
    let ext = string.as_utf16_with_nul();
    for (i, value) in b"Test".iter().enumerate() {
        assert_eq!(ext[i], u16::from(*value));
    }
}
#[test]
fn TCollection_ExtendedStringTest__IsAscii() {
    assert!(ExtendedString::new("Simple ASCII").is_ascii());
    assert!(!ExtendedString::from_utf16(&[65, 0x20AC, 0]).is_ascii());
}
#[test]
fn TCollection_ExtendedStringTest__Cat() {
    let string = ExtendedString::new("Hello");
    text(&string.cat(ExtendedString::new(" World")), "Hello World");
    text(&string.cat(b'!'), "Hello!");
    text(&(&string + b'!'), "Hello!");
}
#[test]
fn TCollection_ExtendedStringTest__AssignCat_IntegerAndReal() {
    let mut integer = ExtendedString::new("Value: ");
    integer.assign_cat(42);
    text(&integer, "Value: 42");
    let mut character = ExtendedString::new("Value: ");
    character += b'!';
    text(&character, "Value: !");
    let mut operator = ExtendedString::new("Value: ");
    operator += -7;
    text(&operator, "Value: -7");
    let mut zero = ExtendedString::new("Value: ");
    zero += 0_i32;
    text(&zero, "Value: 0");
    let mut real = ExtendedString::new("Pi is approximately ");
    real.assign_cat(3.14159);
    assert!(contains(&real, "3.14159"));
}
#[test]
fn TCollection_ExtendedStringTest__Cat_IntegerAndReal() {
    let string = ExtendedString::new("Count: ");
    text(&string.cat(42), "Count: 42");
    text(&(&string + (-100)), "Count: -100");
    text(&string.cat(0_i32), "Count: 0");
    let real = ExtendedString::new("Value: ");
    assert!(contains(&(&real + 3.14), "3.14"));
    text(&string, "Count: ");
}
#[test]
fn TCollection_ExtendedStringTest__ChangeAll() {
    let mut string = ExtendedString::new("Helloo Woorld");
    string.change_all(u16::from(b'o'), u16::from(b'X'));
    text(&string, "HellXX WXXrld");
}
